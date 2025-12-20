/**
 * Optimization Phase - Enhanced
 *
 * Applies quality optimizations to improve layout:
 * - Overlap removal (hierarchy-aware, bottom-up)
 * - Containment enforcement (elastic padding)
 * - Crossing minimization (layer-by-layer swapping)
 * - Spacing optimization (balanced whitespace)
 * - Label placement (collision-free)
 *
 * Migrated from legacy algorithms/optimizer.ts
 */

import type {
  LayoutPhase,
  LayoutContext,
  LayoutNode,
  OptimizationPhase,
  Point,
  Rect,
} from "../core/types";
import {
  MIN_PARENT_PADDING,
  SAFETY_MARGIN,
  LABEL_HEIGHT_BUFFER,
} from "../constants";

// ============================================================================
// PHASE DEFINITION
// ============================================================================

export function createOptimizationPhase(): LayoutPhase {
  return {
    name: "optimization",
    description: "Apply layout optimizations",
    dependencies: ["edge-routing"],
    execute: async (context: LayoutContext): Promise<LayoutContext> => {
      if (!context.options.optimization?.enabled) {
        return context;
      }

      let optimizedContext = context;
      const phases = context.options.optimization.phases || [];

      for (const phaseName of phases) {
        optimizedContext = await applyOptimizationPhase(phaseName, optimizedContext);

        // Check if we've reached target quality
        if (
          context.options.quality.earlyExit &&
          optimizedContext.qualityScore.score >= getTargetScore(context.options.quality.targetGrade)
        ) {
          break;
        }
      }

      // CRITICAL: Overlap removal now respects parent bounds during separation
      // This prevents nodes from being pushed outside parents
      optimizedContext = applyOverlapRemoval(optimizedContext);

      // CRITICAL: Global overlap removal catches cross-hierarchy overlaps
      // Note: This may still push nodes outside parents, so we'll fix it
      optimizedContext = applyGlobalOverlapRemoval(optimizedContext);

      // CRITICAL: Force fix explicitly handles stubborn overlaps
      // Note: This may still push nodes outside parents, so we'll fix it
      optimizedContext = forceFixRemainingOverlaps(optimizedContext);

      // CRITICAL: Final localized overlap removal to clean up after force fix
      // This respects parent bounds, so should be safe
      optimizedContext = applyOverlapRemoval(optimizedContext);

      // CRITICAL: ALWAYS run containment enforcement as the FINAL authoritative step
      // This ensures that even if global/force-fix pushed nodes out, they are pulled back in (or parent grows)
      // Since overlap removal now respects bounds, this should mostly just resize parents if needed
      optimizedContext = applyContainmentEnforcement(optimizedContext);

      return optimizedContext;
    },
  };
}

async function applyOptimizationPhase(
  phase: OptimizationPhase,
  context: LayoutContext
): Promise<LayoutContext> {
  switch (phase) {
    case "overlap-removal":
      return applyOverlapRemoval(context);
    case "crossing-minimization":
      return applyCrossingMinimization(context);
    case "spacing-optimization":
      return applySpacingOptimization(context);
    case "edge-routing":
      // Edge routing is handled by its own phase
      return context;
    case "label-placement":
      return applyLabelPlacement(context);
    case "containment-enforcement":
      return applyContainmentEnforcement(context);
    default:
      return context;
  }
}

// ============================================================================
// OVERLAP REMOVAL (HIERARCHY-AWARE, BOTTOM-UP)
// ============================================================================

export function applyOverlapRemoval(context: LayoutContext): LayoutContext {
  const nodes = new Map(context.nodes);
  const padding = context.options.spacing.nodePadding;

  // Debug: Check for overlaps before removal
  if (context.options.debug?.enabled) {
    const beforeOverlaps = detectAllOverlaps(Array.from(nodes.values()));
    if (beforeOverlaps.length > 0) {
      console.log(`[OverlapRemoval] Found ${beforeOverlaps.length} overlaps before removal:`);
      beforeOverlaps.slice(0, 5).forEach(([a, b]) => {
        console.log(`  - ${a.id} overlaps ${b.id}`);
      });
    }
  }

  // Build hierarchy for bottom-up processing
  // CRITICAL: Sort root nodes for deterministic processing order
  const rootNodes = Array.from(nodes.values())
    .filter((n) => !n.parent && n.visible)
    .sort((a, b) => a.id.localeCompare(b.id));

  // Process in bottom-up order (children before parents)
  for (const root of rootNodes) {
    processNodeBottomUp(root, nodes, padding);
  }

  // Fix overlaps among root nodes - CRITICAL: These cause the persistent overlaps
  if (rootNodes.length > 1) {
    const rootNodesList = rootNodes.map((n) => nodes.get(n.id)!);

    // Run overlap removal multiple times for root nodes with increased iterations
    let rootIterations = 0;
    const maxRootIterations = 3;
    let previousOverlaps = Infinity;

    while (rootIterations < maxRootIterations) {
      rootIterations++;

      // Use more aggressive parameters for root nodes
      const adjustedRoots = removeOverlapsAmongSiblings(rootNodesList, Math.max(padding, 60), nodes);

      // Count remaining overlaps
      let currentOverlaps = 0;
      for (let i = 0; i < adjustedRoots.length; i++) {
        const a = adjustedRoots[i];
        for (let j = i + 1; j < adjustedRoots.length; j++) {
          const b = adjustedRoots[j];
          const aRight = a.bbox.x + a.bbox.width;
          const aBottom = a.bbox.y + a.bbox.height;
          const bRight = b.bbox.x + b.bbox.width;
          const bBottom = b.bbox.y + b.bbox.height;

          if (a.bbox.x < bRight && aRight > b.bbox.x &&
            a.bbox.y < bBottom && aBottom > b.bbox.y) {
            currentOverlaps++;
          }
        }
      }

      // Update nodes with adjusted positions
      for (const adjusted of adjustedRoots) {
        nodes.set(adjusted.id, adjusted);
        // Update the list for next iteration
        const idx = rootNodesList.findIndex((n) => n.id === adjusted.id);
        if (idx >= 0) {
          rootNodesList[idx] = adjusted;
        }
      }

      // If no improvement or no overlaps, stop
      if (currentOverlaps === 0 || currentOverlaps >= previousOverlaps) {
        break;
      }
      previousOverlaps = currentOverlaps;
    }
  }

  // Debug: Check for overlaps after removal
  if (context.options.debug?.enabled) {
    const afterOverlaps = detectAllOverlaps(Array.from(nodes.values()));
    if (afterOverlaps.length > 0) {
      console.log(`[OverlapRemoval] WARNING: ${afterOverlaps.length} overlaps remain after removal:`);
      afterOverlaps.forEach(([a, b]) => {
        console.log(`  - ${a.id} (${a.bbox.x.toFixed(1)},${a.bbox.y.toFixed(1)} ${a.bbox.width}x${a.bbox.height}) overlaps ${b.id} (${b.bbox.x.toFixed(1)},${b.bbox.y.toFixed(1)} ${b.bbox.width}x${b.bbox.height})`);
      });
    } else {
      console.log(`[OverlapRemoval] ✓ All overlaps removed`);
    }
  }

  return { ...context, nodes };
}

// Helper to detect all overlaps (for debugging)
function detectAllOverlaps(nodes: LayoutNode[]): Array<[LayoutNode, LayoutNode]> {
  const overlaps: Array<[LayoutNode, LayoutNode]> = [];
  const visibleNodes = nodes.filter((n) => n.visible);

  for (let i = 0; i < visibleNodes.length; i++) {
    const a = visibleNodes[i];
    for (let j = i + 1; j < visibleNodes.length; j++) {
      const b = visibleNodes[j];

      // Skip parent-child (containment handles that)
      if (a.parent?.id === b.id || b.parent?.id === a.id) continue;

      // Check overlap using same logic as quality evaluator
      const aRight = a.bbox.x + a.bbox.width;
      const aBottom = a.bbox.y + a.bbox.height;
      const bRight = b.bbox.x + b.bbox.width;
      const bBottom = b.bbox.y + b.bbox.height;

      const overlapsX = a.bbox.x < bRight && aRight > b.bbox.x;
      const overlapsY = a.bbox.y < bBottom && aBottom > b.bbox.y;

      if (overlapsX && overlapsY) {
        overlaps.push([a, b]);
      }
    }
  }

  return overlaps;
}

function processNodeBottomUp(
  node: LayoutNode,
  nodes: Map<string, LayoutNode>,
  padding: number
): void {
  // First, process all children recursively
  for (const child of node.children) {
    if (child.visible) {
      processNodeBottomUp(child, nodes, padding);
    }
  }

  // Then fix overlaps among this node's children
  // CRITICAL: Sort children once for deterministic processing
  const visibleChildren = node.children
    .filter((c) => c.visible)
    .sort((a, b) => a.id.localeCompare(b.id));
  if (visibleChildren.length > 1 && !node.collapsed) {
    const childNodes = visibleChildren.map((c) => nodes.get(c.id)!).filter(Boolean);
    
    // CRITICAL: Use more aggressive overlap removal for children within parent
    // Increase iterations and padding to ensure no overlaps remain
    // Pass nodes map so overlap removal can respect parent bounds
    const childPadding = Math.max(padding, 60); // Ensure minimum spacing
    const adjusted = removeOverlapsAmongSiblings(childNodes, childPadding, nodes);
    
      // Update nodes with adjusted positions
      // Note: removeOverlapsAmongSiblings now respects parent bounds internally,
      // but we still do a final clamp pass to ensure 100% containment
      const parent = nodes.get(node.id);
      if (parent) {
        const effectivePadding = Math.max(80, padding * 1.5);
        const minX = parent.bbox.x + effectivePadding;
        const minY = parent.bbox.y + effectivePadding + 30;
        const maxX = parent.bbox.x + parent.bbox.width - effectivePadding;
        const maxY = parent.bbox.y + parent.bbox.height - effectivePadding;

        for (const adjustedNode of adjusted) {
          // Final safety clamp (should already be within bounds, but ensure)
          let clampedX = Math.max(minX, Math.min(maxX - adjustedNode.bbox.width, adjustedNode.bbox.x));
          let clampedY = Math.max(minY, Math.min(maxY - adjustedNode.bbox.height, adjustedNode.bbox.y));

          // Ensure child doesn't extend beyond parent
          if (clampedX + adjustedNode.bbox.width > maxX) {
            clampedX = maxX - adjustedNode.bbox.width;
          }
          if (clampedY + adjustedNode.bbox.height > maxY) {
            clampedY = maxY - adjustedNode.bbox.height;
          }

          const clampedNode = updateNodePosition(adjustedNode, clampedX, clampedY);
          nodes.set(clampedNode.id, clampedNode);
        }
      } else {
        // No parent, just update normally
        for (const adjustedNode of adjusted) {
          nodes.set(adjustedNode.id, adjustedNode);
        }
      }

    // Resize parent to fit adjusted children
    resizeParentToFitChildren(node, nodes, padding);
  }
}

function removeOverlapsAmongSiblings(
  siblings: LayoutNode[],
  padding: number,
  allNodes?: Map<string, LayoutNode>
): LayoutNode[] {
  if (siblings.length <= 1) return siblings;

  const adjusted = [...siblings];
  let hasOverlap = true;
  let iterations = 0;

  // Get parent bounds if siblings have a common parent
  const getParentBounds = (node: LayoutNode): { minX: number; minY: number; maxX: number; maxY: number; padding: number } | null => {
    if (!node.parent || !allNodes) return null;
    const parent = allNodes.get(node.parent.id);
    if (!parent) return null;
    const effectivePadding = Math.max(80, padding * 1.5);
    return {
      minX: parent.bbox.x + effectivePadding,
      minY: parent.bbox.y + effectivePadding + 30, // Label height
      maxX: parent.bbox.x + parent.bbox.width - effectivePadding,
      maxY: parent.bbox.y + parent.bbox.height - effectivePadding,
      padding: effectivePadding
    };
  };

  // Check if all siblings share the same parent
  const commonParent = siblings[0]?.parent;
  const allShareParent = commonParent && siblings.every(s => s.parent?.id === commonParent.id);
  const parentBounds = allShareParent && siblings[0] ? getParentBounds(siblings[0]) : null;

  // Helper to clamp position within parent bounds
  const clampToParent = (node: LayoutNode, x: number, y: number): { x: number; y: number } => {
    if (!parentBounds) return { x, y };
    const clampedX = Math.max(
      parentBounds.minX,
      Math.min(parentBounds.maxX - node.bbox.width, x)
    );
    const clampedY = Math.max(
      parentBounds.minY,
      Math.min(parentBounds.maxY - node.bbox.height, y)
    );
    return { x: clampedX, y: clampedY };
  };

  // More iterations for larger groups - increased for better overlap resolution
  const maxIterations = siblings.length > 5 ? 35 : 20;
  while (hasOverlap && iterations < maxIterations) {
    hasOverlap = false;
    iterations++;

    for (let i = 0; i < adjusted.length; i++) {
      const a = adjusted[i];
      const aRight = a.bbox.x + a.bbox.width + padding;
      const aBottom = a.bbox.y + a.bbox.height + padding;

      for (let j = i + 1; j < adjusted.length; j++) {
        const b = adjusted[j];
        const bRight = b.bbox.x + b.bbox.width + padding;
        const bBottom = b.bbox.y + b.bbox.height + padding;

        // Check for overlap - use strict check (no padding) to match quality evaluator
        const aRightStrict = a.bbox.x + a.bbox.width;
        const aBottomStrict = a.bbox.y + a.bbox.height;
        const bRightStrict = b.bbox.x + b.bbox.width;
        const bBottomStrict = b.bbox.y + b.bbox.height;

        const overlapX = a.bbox.x < bRightStrict && aRightStrict > b.bbox.x;
        const overlapY = a.bbox.y < bBottomStrict && aBottomStrict > b.bbox.y;

        if (overlapX && overlapY) {
          hasOverlap = true;

          // Calculate overlap amounts
          const overlapWidth = Math.min(aRight - b.bbox.x, bRight - a.bbox.x);
          const overlapHeight = Math.min(aBottom - b.bbox.y, bBottom - a.bbox.y);

          // Push apart in the direction of least overlap, but respect parent bounds
          if (overlapWidth < overlapHeight) { // Separate horizontally
            const minSeparation = padding + 5;
            const centerX = (a.bbox.x + a.bbox.width / 2 + b.bbox.x + b.bbox.width / 2) / 2;

            // Move both away from center
            let newAX = centerX - minSeparation / 2 - a.bbox.width;
            let newBX = centerX + minSeparation / 2;

            // Clamp to parent bounds if applicable
            const clampedA = clampToParent(a, newAX, a.bbox.y);
            const clampedB = clampToParent(b, newBX, b.bbox.y);

            adjusted[i] = updateNodePosition(a, clampedA.x, clampedA.y);
            adjusted[j] = updateNodePosition(b, clampedB.x, clampedB.y);
          } else {
            // Separate vertically
            const minSeparation = padding + 5;
            const centerY = (a.bbox.y + a.bbox.height / 2 + b.bbox.y + b.bbox.height / 2) / 2;

            let newAY = centerY - minSeparation / 2 - a.bbox.height;
            let newBY = centerY + minSeparation / 2;

            // Clamp to parent bounds if applicable
            const clampedA = clampToParent(a, a.bbox.x, newAY);
            const clampedB = clampToParent(b, b.bbox.x, newBY);

            adjusted[i] = updateNodePosition(a, clampedA.x, clampedA.y);
            adjusted[j] = updateNodePosition(b, clampedB.x, clampedB.y);
          }

        }
      }
    }
  }

  // Final pass: verify no overlaps remain and fix any that do
  // This handles cases where parent constraints prevented full separation
  // CRITICAL: Still respect parent bounds during final pass
  let finalHasOverlap = true;
  let finalIterations = 0;
  while (finalHasOverlap && finalIterations < 10) {
    finalHasOverlap = false;
    finalIterations++;

    for (let i = 0; i < adjusted.length; i++) {
      const a = adjusted[i];

      for (let j = i + 1; j < adjusted.length; j++) {
        const b = adjusted[j];

        // Use strict overlap check
        const aRightStrict = a.bbox.x + a.bbox.width;
        const aBottomStrict = a.bbox.y + a.bbox.height;
        const bRightStrict = b.bbox.x + b.bbox.width;
        const bBottomStrict = b.bbox.y + b.bbox.height;

        const overlapX = a.bbox.x < bRightStrict && aRightStrict > b.bbox.x;
        const overlapY = a.bbox.y < bBottomStrict && aBottomStrict > b.bbox.y;

        if (overlapX && overlapY) {
          finalHasOverlap = true;
          // Force separation - move b to the right/bottom of a with minimum spacing
          const separation = padding + 10;
          let newBX = aRightStrict + separation - b.bbox.width;
          let newBY = aBottomStrict + separation - b.bbox.height;

          // Use whichever direction gives more space, but clamp to parent bounds
          if (Math.abs(newBX - b.bbox.x) < Math.abs(newBY - b.bbox.y)) {
            const clamped = clampToParent(b, newBX, b.bbox.y);
            adjusted[j] = updateNodePosition(b, clamped.x, clamped.y);
          } else {
            const clamped = clampToParent(b, b.bbox.x, newBY);
            adjusted[j] = updateNodePosition(b, clamped.x, clamped.y);
          }
        }
      }
    }
  }

  return adjusted;
}

/**
 * Global overlap removal - checks ALL nodes for overlaps, not just siblings
 * This catches overlaps between nodes in different parent groups
 */
export function applyGlobalOverlapRemoval(context: LayoutContext): LayoutContext {
  const nodes = new Map(context.nodes);
  const padding = context.options.spacing.nodePadding;
  // CRITICAL: Sort visible nodes for deterministic processing order
  const visibleNodes = Array.from(nodes.values())
    .filter((n) => n.visible)
    .sort((a, b) => a.id.localeCompare(b.id));

  if (visibleNodes.length <= 1) return context;

  let hasOverlap = true;
  let iterations = 0;
  const maxIterations = 30; // Increased for root-level nodes

  while (hasOverlap && iterations < maxIterations) {
    hasOverlap = false;
    iterations++;

    for (let i = 0; i < visibleNodes.length; i++) {
      const a = nodes.get(visibleNodes[i].id);
      if (!a) continue;

      const aRightStrict = a.bbox.x + a.bbox.width;
      const aBottomStrict = a.bbox.y + a.bbox.height;

      for (let j = i + 1; j < visibleNodes.length; j++) {
        const b = nodes.get(visibleNodes[j].id);
        if (!b) continue;

        // Skip if they're parent-child (containment handles that)
        if (a.parent?.id === b.id || b.parent?.id === a.id) continue;

        // Check if they're siblings (same parent) - these should be handled by sibling removal
        // But we'll still check them here as a safety net
        const areSiblings = a.parent?.id === b.parent?.id && a.parent;

        // Check for cross-hierarchy overlaps (root overlapping with child of different system)
        const aIsRoot = !a.parent;
        const bIsRoot = !b.parent;
        const crossHierarchy = (aIsRoot && !bIsRoot) || (!aIsRoot && bIsRoot);

        const bRightStrict = b.bbox.x + b.bbox.width;
        const bBottomStrict = b.bbox.y + b.bbox.height;

        // Strict overlap check to match quality evaluator
        const overlapX = a.bbox.x < bRightStrict && aRightStrict > b.bbox.x;
        const overlapY = a.bbox.y < bBottomStrict && aBottomStrict > b.bbox.y;

        if (overlapX && overlapY) {
          hasOverlap = true;

          // Calculate overlap amounts
          const overlapWidth = Math.min(aRightStrict - b.bbox.x, bRightStrict - a.bbox.x);
          const overlapHeight = Math.min(aBottomStrict - b.bbox.y, bBottomStrict - a.bbox.y);

          // Reduced from padding + 30
          const minSeparation = padding + 10;

          // Handle different overlap scenarios
          if (aIsRoot && bIsRoot) {
            // Both are root nodes - move b away from a with aggressive separation
            const pushDist = Math.max(minSeparation, Math.max(overlapWidth, overlapHeight) * 1.5); // Reduced from 4.0
            if (overlapWidth < overlapHeight) {
              // Separate horizontally
              const newBX = aRightStrict + pushDist - b.bbox.width;
              nodes.set(b.id, updateNodePosition(b, newBX, b.bbox.y));
            } else {
              // Separate vertically
              const newBY = aBottomStrict + pushDist - b.bbox.height;
              nodes.set(b.id, updateNodePosition(b, b.bbox.x, newBY));
            }
          } else if (crossHierarchy) {
            // Cross-hierarchy: root overlapping with child of different system
            // Move the root node (no parent constraints)
            const pushDist = Math.max(minSeparation, Math.max(overlapWidth, overlapHeight) * 1.5); // Reduced from 5.0

            if (aIsRoot) {
              // a is root, b is child - move a away
              if (overlapWidth < overlapHeight) {
                const newAY = bBottomStrict + pushDist - a.bbox.height;
                nodes.set(a.id, updateNodePosition(a, a.bbox.x, newAY));
              } else {
                const newAX = bRightStrict + pushDist - a.bbox.width;
                nodes.set(a.id, updateNodePosition(a, newAX, a.bbox.y));
              }
            } else {
              // b is root, a is child - move b away
              if (overlapWidth < overlapHeight) {
                const newBY = aBottomStrict + pushDist - b.bbox.height;
                nodes.set(b.id, updateNodePosition(b, b.bbox.x, newBY));
              } else {
                const newBX = aRightStrict + pushDist - b.bbox.width;
                nodes.set(b.id, updateNodePosition(b, newBX, b.bbox.y));
              }
            }
          } else if (areSiblings) {
            // Siblings - should have been handled, but fix anyway
            // CRITICAL: Respect parent bounds when moving siblings
            const pushDist = Math.max(minSeparation, Math.max(overlapWidth, overlapHeight) * 1.2);
            const effectivePadding = Math.max(80, padding * 1.5);
            
            if (overlapWidth < overlapHeight) {
              let newBY = aBottomStrict + pushDist - b.bbox.height;
              // Check parent bounds
              if (a.parent) {
                const parent = nodes.get(a.parent.id);
                if (parent) {
                  const minY = parent.bbox.y + effectivePadding + 30;
                  const maxY = parent.bbox.y + parent.bbox.height - effectivePadding;
                  newBY = Math.max(minY, Math.min(maxY - b.bbox.height, newBY));
                }
              }
              nodes.set(b.id, updateNodePosition(b, b.bbox.x, newBY));
            } else {
              let newBX = aRightStrict + pushDist - b.bbox.width;
              if (a.parent) {
                const parent = nodes.get(a.parent.id);
                if (parent) {
                  const minX = parent.bbox.x + effectivePadding;
                  const maxX = parent.bbox.x + parent.bbox.width - effectivePadding;
                  newBX = Math.max(minX, Math.min(maxX - b.bbox.width, newBX));
                }
              }
              nodes.set(b.id, updateNodePosition(b, newBX, b.bbox.y));
            }
          } else {
            // Both have parents - try to move b, respecting parent constraints
            const effectivePadding = Math.max(80, padding * 1.5);
            
            if (overlapWidth < overlapHeight) {
              const pushDist = Math.max(minSeparation, overlapWidth * 2.5);
              let newBX = aRightStrict + pushDist - b.bbox.width;

              if (b.parent) {
                const parent = nodes.get(b.parent.id);
                if (parent) {
                  const minX = parent.bbox.x + effectivePadding;
                  const maxX = parent.bbox.x + parent.bbox.width - effectivePadding;
                  newBX = Math.max(minX, Math.min(maxX - b.bbox.width, newBX));
                }
              }
              nodes.set(b.id, updateNodePosition(b, newBX, b.bbox.y));
            } else {
              const pushDist = Math.max(minSeparation, overlapHeight * 2.5);
              let newBY = aBottomStrict + pushDist - b.bbox.height;

              if (b.parent) {
                const parent = nodes.get(b.parent.id);
                if (parent) {
                  const minY = parent.bbox.y + effectivePadding + 30;
                  const maxY = parent.bbox.y + parent.bbox.height - effectivePadding;
                  newBY = Math.max(minY, Math.min(maxY - b.bbox.height, newBY));
                }
              }
              nodes.set(b.id, updateNodePosition(b, b.bbox.x, newBY));
            }
          }
        }
      }
    }
  }

  return { ...context, nodes };
}

/**
 * Force fix any remaining overlaps - last resort aggressive approach
 * This runs after all other optimizations to catch stubborn overlaps
 * Specifically handles root-level node overlaps
 */
export function forceFixRemainingOverlaps(context: LayoutContext): LayoutContext {
  const nodes = new Map(context.nodes);
  // CRITICAL: Sort visible nodes for deterministic processing order
  const visibleNodes = Array.from(nodes.values())
    .filter((n) => n.visible)
    .sort((a, b) => a.id.localeCompare(b.id));
  const padding = context.options.spacing.nodePadding;

  if (visibleNodes.length <= 1) return context;

  // Detect all overlaps, prioritizing root-level nodes
  const overlaps: Array<[LayoutNode, LayoutNode]> = [];
  const rootOverlaps: Array<[LayoutNode, LayoutNode]> = [];

  for (let i = 0; i < visibleNodes.length; i++) {
    const a = visibleNodes[i];
    for (let j = i + 1; j < visibleNodes.length; j++) {
      const b = visibleNodes[j];

      // Skip parent-child
      if (a.parent?.id === b.id || b.parent?.id === a.id) continue;

      const aRight = a.bbox.x + a.bbox.width;
      const aBottom = a.bbox.y + a.bbox.height;
      const bRight = b.bbox.x + b.bbox.width;
      const bBottom = b.bbox.y + b.bbox.height;

      const overlapsX = a.bbox.x < bRight && aRight > b.bbox.x;
      const overlapsY = a.bbox.y < bBottom && aBottom > b.bbox.y;

      if (overlapsX && overlapsY) {
        const pair: [LayoutNode, LayoutNode] = [a, b];
        // Prioritize root-level overlaps and cross-hierarchy overlaps
        // (root node overlapping with child of another system)
        const aIsRoot = !a.parent;
        const bIsRoot = !b.parent;
        const crossHierarchy = (aIsRoot && !bIsRoot) || (!aIsRoot && bIsRoot);

        if (aIsRoot && bIsRoot) {
          rootOverlaps.push(pair);
        } else if (crossHierarchy) {
          // Cross-hierarchy overlaps: root overlapping with child of another system
          // These are critical and should be fixed aggressively
          rootOverlaps.push(pair);
        } else {
          overlaps.push(pair);
        }
      }
    }
  }

  // Fix root-level and cross-hierarchy overlaps first
  for (const [a, b] of rootOverlaps) {
    const aNode = nodes.get(a.id);
    const bNode = nodes.get(b.id);
    if (!aNode || !bNode) continue;

    const aIsRoot = !a.parent;
    const bIsRoot = !b.parent;
    const crossHierarchy = (aIsRoot && !bIsRoot) || (!aIsRoot && bIsRoot);

    // Debug logging for specific problematic nodes
    // Note: IDs are like c4_Customer, c4_ECommerce.WebApp, c4_ECommerce.ProductDB
    const isProblematic =
      a.id.includes('Customer') || a.id.includes('WebApp') || a.id.includes('ProductDB') ||
      a.id.includes('OrderDB') || a.id.includes('UserDB') ||
      b.id.includes('Customer') || b.id.includes('WebApp') || b.id.includes('ProductDB') ||
      b.id.includes('OrderDB') || b.id.includes('UserDB');

    if (isProblematic && typeof process !== 'undefined' && process.env?.LAYOUT_DEBUG) {
      console.log(`[ForceFix] Fixing ${crossHierarchy ? 'cross-hierarchy' : 'root'} overlap: ${a.id} ↔ ${b.id}`);
      console.log(`  ${a.id}: root=${aIsRoot}, parent=${a.parent?.id || 'none'}, at (${aNode.bbox.x.toFixed(1)}, ${aNode.bbox.y.toFixed(1)}) ${aNode.bbox.width}x${aNode.bbox.height}`);
      console.log(`  ${b.id}: root=${bIsRoot}, parent=${b.parent?.id || 'none'}, at (${bNode.bbox.x.toFixed(1)}, ${bNode.bbox.y.toFixed(1)}) ${bNode.bbox.width}x${bNode.bbox.height}`);
    }

    const aRight = aNode.bbox.x + aNode.bbox.width;
    const aBottom = aNode.bbox.y + aNode.bbox.height;

    // Aggressive separation - reduced to minimize empty space
    const minSeparation = crossHierarchy ? padding + 20 : padding + 10;

    // Calculate overlap
    const overlapWidth = Math.min(aRight - bNode.bbox.x, (bNode.bbox.x + bNode.bbox.width) - aNode.bbox.x);
    const overlapHeight = Math.min(aBottom - bNode.bbox.y, (bNode.bbox.y + bNode.bbox.height) - aNode.bbox.y);

    // For cross-hierarchy: prefer moving the root node (easier, no parent constraints)
    // For root-root: move b away from a
    if (crossHierarchy && aIsRoot) {
      // a is root, b is child - move a away from b
      let newAX = aNode.bbox.x;
      let newAY = aNode.bbox.y;

      if (overlapWidth < overlapHeight) {
        // Move a vertically away (down)
        // Correct: newAY = bBottom + separation
        newAY = bNode.bbox.y + bNode.bbox.height + minSeparation;
      } else {
        // Move a horizontally away (right)
        // Correct: newAX = bRight + separation
        newAX = bNode.bbox.x + bNode.bbox.width + minSeparation;
      }

      const updatedA = updateNodePosition(aNode, newAX, newAY);
      nodes.set(aNode.id, updatedA);
    } else if (crossHierarchy && bIsRoot) {
      // b is root, a is child - move b away from a
      let newBX = bNode.bbox.x;
      let newBY = bNode.bbox.y;

      if (overlapWidth < overlapHeight) {
        // Move b vertically away (down)
        newBY = aNode.bbox.y + aNode.bbox.height + minSeparation;
      } else {
        // Move b horizontally away (right)
        newBX = aNode.bbox.x + aNode.bbox.width + minSeparation;
      }

      const updatedB = updateNodePosition(bNode, newBX, newBY);
      nodes.set(bNode.id, updatedB);
    } else {
      // Both are root - move b away from a (no parent constraints)
      if (overlapWidth < overlapHeight) {
        // Separate vertically - move b below a
        const newBY = aBottom + minSeparation;
        const updatedB = updateNodePosition(bNode, bNode.bbox.x, newBY);
        nodes.set(bNode.id, updatedB);
      } else {
        // Separate horizontally - move b to the right of a
        const newBX = aRight + minSeparation;
        const updatedB = updateNodePosition(bNode, newBX, bNode.bbox.y);
        nodes.set(bNode.id, updatedB);
      }
    }
  }

  // Fix other overlaps
  for (const [a, b] of overlaps) {
    const aNode = nodes.get(a.id);
    const bNode = nodes.get(b.id);
    if (!aNode || !bNode) continue;

    const aRight = aNode.bbox.x + aNode.bbox.width;
    const aBottom = aNode.bbox.y + aNode.bbox.height;

    const minSeparation = padding + 30;
    const effectivePadding = Math.max(80, padding * 1.5);

    // Try to move b to the right of a
    let newBX = aRight + minSeparation - bNode.bbox.width;
    let newBY = bNode.bbox.y;

    // If b has a parent, check constraints and clamp
    if (bNode.parent) {
      const parent = nodes.get(bNode.parent.id);
      if (parent) {
        const minX = parent.bbox.x + effectivePadding;
        const minY = parent.bbox.y + effectivePadding + 30;
        const maxX = parent.bbox.x + parent.bbox.width - effectivePadding;
        const maxY = parent.bbox.y + parent.bbox.height - effectivePadding;

        // If horizontal move doesn't work, try vertical
        if (newBX > maxX - bNode.bbox.width) {
          newBX = bNode.bbox.x;
          newBY = aBottom + minSeparation - bNode.bbox.height;

          if (newBY > maxY - bNode.bbox.height) {
            // Both directions constrained - move to edge of parent
            newBX = Math.min(maxX - bNode.bbox.width, aRight + minSeparation - bNode.bbox.width);
            newBY = Math.min(maxY - bNode.bbox.height, aBottom + minSeparation - bNode.bbox.height);
          }
        }

        // Ensure within parent bounds (strict clamping)
        newBX = Math.max(minX, Math.min(newBX, maxX - bNode.bbox.width));
        newBY = Math.max(minY, Math.min(newBY, maxY - bNode.bbox.height));
      }
    }

    // Apply the fix
    nodes.set(bNode.id, updateNodePosition(bNode, newBX, newBY));
  }

  return { ...context, nodes };
}

function resizeParentToFitChildren(
  parent: LayoutNode,
  nodes: Map<string, LayoutNode>,
  padding: number
): void {
  const currentParent = nodes.get(parent.id);
  if (!currentParent || parent.children.length === 0) return;

  // Calculate bounding box of all children
  // CRITICAL: Sort children for deterministic processing
  const sortedChildren = [...parent.children].sort((a, b) => a.id.localeCompare(b.id));
  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;

  for (const child of sortedChildren) {
    const childNode = nodes.get(child.id);
    if (childNode && childNode.visible) {
      minX = Math.min(minX, childNode.bbox.x);
      minY = Math.min(minY, childNode.bbox.y);
      maxX = Math.max(maxX, childNode.bbox.x + childNode.bbox.width);
      maxY = Math.max(maxY, childNode.bbox.y + childNode.bbox.height);
    }
  }

  if (minX === Infinity) return;

  // Calculate overflows
  const expandedPadding = parent.children.length > 3 ? padding * 1.5 : padding * 1.2;
  const extraMargin = 40; // Safety margin

  const leftOverflow = Math.max(0, currentParent.bbox.x - minX);
  const topOverflow = Math.max(0, currentParent.bbox.y - minY);
  const rightOverflow = Math.max(0, maxX - (currentParent.bbox.x + currentParent.bbox.width));
  const bottomOverflow = Math.max(0, maxY - (currentParent.bbox.y + currentParent.bbox.height));

  const needsResize =
    leftOverflow > 0 || topOverflow > 0 || rightOverflow > 0 || bottomOverflow > 0;

  if (needsResize) {
    const newX = Math.min(currentParent.bbox.x, minX - expandedPadding - extraMargin);
    const newY = Math.min(currentParent.bbox.y, minY - expandedPadding - extraMargin);
    const newWidth =
      currentParent.bbox.width +
      leftOverflow +
      rightOverflow +
      expandedPadding +
      extraMargin;
    const newHeight =
      currentParent.bbox.height +
      topOverflow +
      bottomOverflow +
      expandedPadding +
      extraMargin;

    nodes.set(parent.id, {
      ...currentParent,
      bbox: { x: newX, y: newY, width: newWidth, height: newHeight },
    });
  }
}

// ============================================================================
// CONTAINMENT ENFORCEMENT
// ============================================================================

/**
 * Diagnostic: Count and report containment violations
 */
function diagnoseContainmentViolations(context: LayoutContext): Array<{
  childId: string;
  parentId: string;
  violation: string;
  details: string;
}> {
  const violations: Array<{
    childId: string;
    parentId: string;
    violation: string;
    details: string;
  }> = [];
  const nodes = context.nodes;

  for (const node of nodes.values()) {
    if (!node.parent || !node.visible) continue;

    const parent = nodes.get(node.parent.id);
    if (!parent) continue;

    const effectivePadding = Math.max(MIN_PARENT_PADDING, context.options.spacing.parentChildPadding);
    const minX = parent.bbox.x + effectivePadding;
    const minY = parent.bbox.y + effectivePadding + LABEL_HEIGHT_BUFFER;

    const childRight = node.bbox.x + node.bbox.width;
    const childBottom = node.bbox.y + node.bbox.height;
    const parentRight = parent.bbox.x + parent.bbox.width;
    const parentBottom = parent.bbox.y + parent.bbox.height;

    const issues: string[] = [];
    if (node.bbox.x < minX) issues.push(`left: ${(minX - node.bbox.x).toFixed(1)}px outside`);
    if (node.bbox.y < minY) issues.push(`top: ${(minY - node.bbox.y).toFixed(1)}px outside`);
    if (childRight > parentRight - effectivePadding)
      issues.push(`right: ${(childRight - (parentRight - effectivePadding)).toFixed(1)}px outside`);
    if (childBottom > parentBottom - effectivePadding)
      issues.push(`bottom: ${(childBottom - (parentBottom - effectivePadding)).toFixed(1)}px outside`);

    if (issues.length > 0) {
      violations.push({
        childId: node.id,
        parentId: parent.id,
        violation: issues.join(", "),
        details: `Child at (${node.bbox.x.toFixed(1)}, ${node.bbox.y.toFixed(1)}) size ${node.bbox.width}x${node.bbox.height}, Parent at (${parent.bbox.x.toFixed(1)}, ${parent.bbox.y.toFixed(1)}) size ${parent.bbox.width}x${parent.bbox.height}`,
      });
    }
  }

  return violations;
}

/**
 * SIMPLIFIED APPROACH: Fix containment in 3 clear steps
 * Step 1: Move children inside parents (simple clamp)
 * Step 2: Resize parents to fit children if needed
 * Step 3: Verify and repeat if needed
 */
function applyContainmentEnforcement(context: LayoutContext): LayoutContext {
  const nodes = new Map(context.nodes);

  const initialViolations = diagnoseContainmentViolations({ ...context, nodes });
  if (initialViolations.length > 0) {
    console.log(`[Containment] Found ${initialViolations.length} violations before fixing`);
    if (context.options.debug?.enabled) {
      initialViolations.slice(0, 3).forEach((v) => {
        console.log(`  - ${v.childId} in ${v.parentId}: ${v.violation}`);
      });
    }
  }

  // Step 1: Clamp all children to parent bounds (aggressive enforcement)
  // CRITICAL: Sort nodes for deterministic processing order
  const sortedNodesForClamp = Array.from(nodes.values())
    .filter((n) => n.parent && n.visible)
    .sort((a, b) => a.id.localeCompare(b.id));

  for (const node of sortedNodesForClamp) {
    const parent = nodes.get(node.parent!.id);
    if (!parent) continue;

    const effectivePadding = Math.max(MIN_PARENT_PADDING, context.options.spacing.parentChildPadding);
    const minX = parent.bbox.x + effectivePadding;
    const minY = parent.bbox.y + effectivePadding + LABEL_HEIGHT_BUFFER;
    const maxX = parent.bbox.x + parent.bbox.width - node.bbox.width - effectivePadding;
    const maxY = parent.bbox.y + parent.bbox.height - node.bbox.height - effectivePadding;

    // CRITICAL: If maxX < minX or maxY < minY, parent is too small - we'll resize it later
    // For now, clamp to valid range
    const clampedMinX = Math.min(minX, maxX);
    const clampedMaxX = Math.max(minX, maxX);
    const clampedMinY = Math.min(minY, maxY);
    const clampedMaxY = Math.max(minY, maxY);

    let newX = Math.max(clampedMinX, Math.min(clampedMaxX, node.bbox.x));
    let newY = Math.max(clampedMinY, Math.min(clampedMaxY, node.bbox.y));

    // Additional safety: ensure child doesn't extend beyond parent
    const childRight = newX + node.bbox.width;
    const childBottom = newY + node.bbox.height;
    const parentRight = parent.bbox.x + parent.bbox.width;
    const parentBottom = parent.bbox.y + parent.bbox.height;

    if (childRight > parentRight - effectivePadding) {
      newX = parentRight - effectivePadding - node.bbox.width;
    }
    if (childBottom > parentBottom - effectivePadding) {
      newY = parentBottom - effectivePadding - node.bbox.height;
    }
    if (newX < minX) {
      newX = minX;
    }
    if (newY < minY) {
      newY = minY;
    }

    if (newX !== node.bbox.x || newY !== node.bbox.y) {
      nodes.set(node.id, updateNodePosition(node, newX, newY));
    }
  }

  const maxDepth = Math.max(...Array.from(nodes.values()).map((n) => n.depth || 0), 0);
  let resizeIterations = 0;
  const maxResizeIterations = 5;

    while (resizeIterations < maxResizeIterations) {
      let anyResized = false;

      for (let depth = maxDepth; depth >= 0; depth--) {
        // CRITICAL: Sort parents for deterministic processing order
        const parentsAtDepth = Array.from(nodes.values())
          .filter((p) => p.depth === depth && !p.collapsed && p.children.length > 0)
          .sort((a, b) => a.id.localeCompare(b.id));

        for (const parent of parentsAtDepth) {

        // CRITICAL: Sort children for deterministic processing
        const visibleChildren = parent.children
          .filter((c) => {
            const child = nodes.get(c.id);
            return child && child.visible;
          })
          .sort((a, b) => a.id.localeCompare(b.id));

        if (visibleChildren.length === 0) continue;

        let minX = Infinity;
        let minY = Infinity;
        let maxX = -Infinity;
        let maxY = -Infinity;

        for (const childRef of visibleChildren) {
          const child = nodes.get(childRef.id);
          if (!child) continue;
          minX = Math.min(minX, child.bbox.x);
          minY = Math.min(minY, child.bbox.y);
          maxX = Math.max(maxX, child.bbox.x + child.bbox.width);
          maxY = Math.max(maxY, child.bbox.y + child.bbox.height);
        }

        if (minX === Infinity) continue;

        const totalPadding = MIN_PARENT_PADDING + SAFETY_MARGIN;
        const requiredWidth = maxX - minX + totalPadding * 2;
        const requiredHeight = maxY - minY + totalPadding * 2 + LABEL_HEIGHT_BUFFER;
        const requiredX = minX - totalPadding;
        const requiredY = minY - totalPadding - LABEL_HEIGHT_BUFFER;

        const currentParent = nodes.get(parent.id)!;
        let needsResize = false;
        let newX = currentParent.bbox.x;
        let newY = currentParent.bbox.y;
        let newWidth = currentParent.bbox.width;
        let newHeight = currentParent.bbox.height;

        if (currentParent.bbox.x > requiredX) {
          newX = requiredX;
          needsResize = true;
        }
        if (currentParent.bbox.y > requiredY) {
          newY = requiredY;
          needsResize = true;
        }
        if (currentParent.bbox.width < requiredWidth) {
          newWidth = requiredWidth;
          needsResize = true;
        }
        if (currentParent.bbox.height < requiredHeight) {
          newHeight = requiredHeight;
          needsResize = true;
        }

        if (needsResize) {
          nodes.set(parent.id, {
            ...currentParent,
            bbox: {
              x: newX,
              y: newY,
              width: newWidth,
              height: newHeight,
            },
          });
          anyResized = true;
        }
      }
    }

    if (!anyResized) break;
    resizeIterations++;
  }

  let repositionIterations = 0;
  const maxRepositionIterations = 3;

  while (repositionIterations < maxRepositionIterations) {
    let anyRepositioned = false;

    // CRITICAL: Sort nodes for deterministic processing order
    const sortedNodesForReposition = Array.from(nodes.values())
      .filter((n) => n.parent && n.visible)
      .sort((a, b) => a.id.localeCompare(b.id));

    for (const node of sortedNodesForReposition) {
      const parent = nodes.get(node.parent!.id);
      if (!parent) continue;

      const effectivePadding = Math.max(MIN_PARENT_PADDING, context.options.spacing.parentChildPadding);
      const minX = parent.bbox.x + effectivePadding;
      const minY = parent.bbox.y + effectivePadding + LABEL_HEIGHT_BUFFER;
      const maxX = parent.bbox.x + parent.bbox.width - node.bbox.width - effectivePadding;
      const maxY = parent.bbox.y + parent.bbox.height - node.bbox.height - effectivePadding;

      if (maxX < minX || maxY < minY) {
        // Parent is too small - center the child as best we can
        const centerX = parent.bbox.x + parent.bbox.width / 2;
        const centerY = parent.bbox.y + parent.bbox.height / 2;
        const newX = Math.max(parent.bbox.x, Math.min(centerX - node.bbox.width / 2, parent.bbox.x + parent.bbox.width - node.bbox.width));
        const newY = Math.max(parent.bbox.y + LABEL_HEIGHT_BUFFER, Math.min(centerY - node.bbox.height / 2, parent.bbox.y + parent.bbox.height - node.bbox.height));
        nodes.set(node.id, updateNodePosition(node, newX, newY));
        anyRepositioned = true;
        continue;
      }

      // Clamp position strictly within bounds
      let newX = Math.max(minX, Math.min(maxX, node.bbox.x));
      let newY = Math.max(minY, Math.min(maxY, node.bbox.y));

      // Additional safety check: ensure child doesn't extend beyond parent
      const childRight = newX + node.bbox.width;
      const childBottom = newY + node.bbox.height;
      const parentRight = parent.bbox.x + parent.bbox.width;
      const parentBottom = parent.bbox.y + parent.bbox.height;

      if (childRight > parentRight - effectivePadding) {
        newX = parentRight - effectivePadding - node.bbox.width;
      }
      if (childBottom > parentBottom - effectivePadding) {
        newY = parentBottom - effectivePadding - node.bbox.height;
      }
      // Ensure we don't go below minimum
      newX = Math.max(minX, newX);
      newY = Math.max(minY, newY);

      if (newX !== node.bbox.x || newY !== node.bbox.y) {
        nodes.set(node.id, updateNodePosition(node, newX, newY));
        anyRepositioned = true;
      }
    }

    if (!anyRepositioned) break;
    repositionIterations++;
  }

  // Final pass: Force fix any remaining violations (last resort)
  const remainingViolations = diagnoseContainmentViolations({ ...context, nodes });
  if (remainingViolations.length > 0) {
    console.warn(`[Containment] ${remainingViolations.length} violations remain, applying force fix`);
    
    for (const violation of remainingViolations) {
      const child = nodes.get(violation.childId);
      const parent = nodes.get(violation.parentId);
      if (!child || !parent) continue;

      const effectivePadding = Math.max(MIN_PARENT_PADDING, context.options.spacing.parentChildPadding);
      const minX = parent.bbox.x + effectivePadding;
      const minY = parent.bbox.y + effectivePadding + LABEL_HEIGHT_BUFFER;
      const maxX = parent.bbox.x + parent.bbox.width - child.bbox.width - effectivePadding;
      const maxY = parent.bbox.y + parent.bbox.height - child.bbox.height - effectivePadding;

      // Force clamp to valid range
      const clampedX = Math.max(minX, Math.min(maxX, child.bbox.x));
      const clampedY = Math.max(minY, Math.min(maxY, child.bbox.y));

      // Ensure child doesn't extend beyond parent
      const finalX = (clampedX + child.bbox.width > parent.bbox.x + parent.bbox.width - effectivePadding)
        ? parent.bbox.x + parent.bbox.width - effectivePadding - child.bbox.width
        : clampedX;
      const finalY = (clampedY + child.bbox.height > parent.bbox.y + parent.bbox.height - effectivePadding)
        ? parent.bbox.y + parent.bbox.height - effectivePadding - child.bbox.height
        : clampedY;

      nodes.set(child.id, updateNodePosition(child, Math.max(minX, finalX), Math.max(minY, finalY)));
    }
  }

  const finalViolations = diagnoseContainmentViolations({ ...context, nodes });
  if (finalViolations.length > 0) {
    console.warn(`[Containment] WARNING: ${finalViolations.length} violations remain after all fixes:`);
    finalViolations.forEach((v) => {
      console.warn(`  - ${v.childId} in ${v.parentId}: ${v.violation}`);
      if (context.options.debug?.enabled) {
        console.warn(`    Details: ${v.details}`);
      }
    });
  } else if (initialViolations.length > 0) {
    console.log(`[Containment] ✓ Fixed all ${initialViolations.length} violations`);
  }

  return { ...context, nodes };
}


// ============================================================================
// CROSSING MINIMIZATION (ENHANCED WITH FORCE-DIRECTED ADJUSTMENTS)
// ============================================================================

function applyCrossingMinimization(context: LayoutContext): LayoutContext {
  const edges = Array.from(context.edges.values());
  if (edges.length < 2) return context;

  let nodes = new Map(context.nodes);

  // Phase 1: Analyze crossing clusters
  const edgeRoutes = edges.map((e) => ({
    id: e.id,
    sourceId: e.source.id,
    targetId: e.target.id,
    path: [...e.points] as Point[],
  }));

  // Count crossings per node
  const crossingCounts = new Map<string, number>();
  for (let i = 0; i < edgeRoutes.length; i++) {
    for (let j = i + 1; j < edgeRoutes.length; j++) {
      const e1 = edgeRoutes[i];
      const e2 = edgeRoutes[j];
      if (pathsCross(e1.path, e2.path, e1.sourceId, e1.targetId, e2.sourceId, e2.targetId)) {
        crossingCounts.set(e1.sourceId, (crossingCounts.get(e1.sourceId) || 0) + 1);
        crossingCounts.set(e1.targetId, (crossingCounts.get(e1.targetId) || 0) + 1);
        crossingCounts.set(e2.sourceId, (crossingCounts.get(e2.sourceId) || 0) + 1);
        crossingCounts.set(e2.targetId, (crossingCounts.get(e2.targetId) || 0) + 1);
      }
    }
  }

  const totalCrossings = Array.from(crossingCounts.values()).reduce((a, b) => a + b, 0) / 4;
  if (totalCrossings === 0) return context;

  // Phase 2: Apply force-directed adjustments to high-crossing nodes
  nodes = applyForceDirectedAdjustments(nodes, crossingCounts, context.graph.relationships);

  // Phase 3: Apply localized swaps within ranks
  nodes = applyRankSwaps(nodes, edgeRoutes);

  // Phase 4: Increase spacing for still-crossing node pairs
  if (totalCrossings > 15) {
    nodes = increaseSpacingForCrossingPairs(nodes, edgeRoutes, 60);
  }

  return { ...context, nodes };
}

function applyForceDirectedAdjustments(
  nodes: Map<string, LayoutNode>,
  crossingCounts: Map<string, number>,
  relationships: readonly { from: string; to: string }[]
): Map<string, LayoutNode> {
  const result = new Map(nodes);

  // Build adjacency map
  const neighbors = new Map<string, Set<string>>();
  for (const rel of relationships) {
    if (!neighbors.has(rel.from)) neighbors.set(rel.from, new Set());
    if (!neighbors.has(rel.to)) neighbors.set(rel.to, new Set());
    neighbors.get(rel.from)!.add(rel.to);
    neighbors.get(rel.to)!.add(rel.from);
  }

  // Get high-crossing nodes (top 30%)
  const allNodes = Array.from(result.entries())
    .map(([id, node]) => ({ id, node, crossings: crossingCounts.get(id) || 0 }))
    .filter((n) => n.crossings > 0)
    .sort((a, b) => b.crossings - a.crossings);

  const topCount = Math.max(5, Math.ceil(allNodes.length * 0.3));
  const highCrossingNodes = allNodes.slice(0, topCount);
  if (highCrossingNodes.length === 0) return result;

  // Apply force iterations
  const maxIterations = 4;
  const stepSize = 15;

  for (let iter = 0; iter < maxIterations; iter++) {
    const forces = new Map<string, { fx: number; fy: number }>();

    for (const { id } of highCrossingNodes) {
      forces.set(id, { fx: 0, fy: 0 });
    }

    // Calculate repulsion forces
    for (let i = 0; i < highCrossingNodes.length; i++) {
      for (let j = i + 1; j < highCrossingNodes.length; j++) {
        const n1 = highCrossingNodes[i];
        const n2 = highCrossingNodes[j];

        // Skip neighbors
        if (neighbors.get(n1.id)?.has(n2.id)) continue;

        // Skip parent-child
        const n1Node = result.get(n1.id);
        const n2Node = result.get(n2.id);
        if (n1Node?.parent?.id === n2.id || n2Node?.parent?.id === n1.id) continue;

        const dx = n2.node.bbox.x - n1.node.bbox.x;
        const dy = n2.node.bbox.y - n1.node.bbox.y;
        const distSq = dx * dx + dy * dy;
        const dist = Math.sqrt(distSq) || 1;

        const strength = Math.min(n1.crossings * n2.crossings * 0.5, 150);
        const force = strength / Math.max(distSq, 50);

        const fx = (dx / dist) * force;
        const fy = (dy / dist) * force;

        const f1 = forces.get(n1.id)!;
        const f2 = forces.get(n2.id)!;
        f1.fx -= fx;
        f1.fy -= fy;
        f2.fx += fx;
        f2.fy += fy;
      }
    }

    // Apply forces
    const damping = 0.7 - iter * 0.1;
    for (const [id, force] of forces) {
      const node = result.get(id)!;
      const moveX = Math.max(-30, Math.min(30, force.fx * stepSize * damping));
      const moveY = Math.max(-30, Math.min(30, force.fy * stepSize * damping));

      if (Math.abs(moveX) > 1 || Math.abs(moveY) > 1) {
        const newX = node.bbox.x + moveX;
        const newY = node.bbox.y + moveY;

        // Check parent containment
        if (node.parent) {
          const parent = result.get(node.parent.id);
          if (parent) {
            const minX = parent.bbox.x + 20;
            const minY = parent.bbox.y + 20;
            const maxX = parent.bbox.x + parent.bbox.width - node.bbox.width - 20;
            const maxY = parent.bbox.y + parent.bbox.height - node.bbox.height - 20;
            if (newX < minX || newY < minY || newX > maxX || newY > maxY) continue;
          }
        }

        result.set(id, updateNodePosition(node, newX, newY));
      }
    }
  }

  return result;
}

function applyRankSwaps(
  nodes: Map<string, LayoutNode>,
  _edges: Array<{ id: string; sourceId: string; targetId: string; path: Point[] }>
): Map<string, LayoutNode> {
  const result = new Map(nodes);
  const RANK_TOLERANCE = 50;

  // Group nodes by rank
  const rankGroups = new Map<number, Array<{ id: string; node: LayoutNode }>>();
  // CRITICAL: Sort entries for deterministic iteration
  const sortedResult = Array.from(result.entries()).sort((a, b) => a[0].localeCompare(b[0]));
  
  for (const [id, node] of sortedResult) {
    if (!node.visible) continue;
    const centerY = node.bbox.y + node.bbox.height / 2;
    const rankKey = Math.round(centerY / RANK_TOLERANCE) * RANK_TOLERANCE;
    if (!rankGroups.has(rankKey)) rankGroups.set(rankKey, []);
    rankGroups.get(rankKey)!.push({ id, node });
  }

  // Sort each rank by X and try swaps
  let swapsApplied = 0;
  const maxSwaps = 20;

  // CRITICAL: Sort rank keys for deterministic iteration
  const sortedRankKeys = Array.from(rankGroups.keys()).sort((a, b) => a - b);
  for (const rankKey of sortedRankKeys) {
    const nodesInRank = rankGroups.get(rankKey)!;
    if (nodesInRank.length < 2 || swapsApplied >= maxSwaps) continue;
    nodesInRank.sort((a, b) => a.node.bbox.x - b.node.bbox.x);

    for (let i = 0; i < nodesInRank.length - 1 && swapsApplied < maxSwaps; i++) {
      const node1 = nodesInRank[i];
      const node2 = nodesInRank[i + 1];

      // Skip parent-child
      if (node1.node.parent?.id === node2.id || node2.node.parent?.id === node1.id) continue;

      // Swap positions
      const newX1 = node2.node.bbox.x;
      const newY1 = node2.node.bbox.y;
      const newX2 = node1.node.bbox.x;
      const newY2 = node1.node.bbox.y;

      const swapped1 = updateNodePosition(node1.node, newX1, newY1);
      const swapped2 = updateNodePosition(node2.node, newX2, newY2);

      // Validate containment
      if (swapped1.parent) {
        const parent = result.get(swapped1.parent.id);
        if (parent && !isContained(swapped1, parent)) continue;
      }
      if (swapped2.parent) {
        const parent = result.get(swapped2.parent.id);
        if (parent && !isContained(swapped2, parent)) continue;
      }

      result.set(node1.id, swapped1);
      result.set(node2.id, swapped2);
      nodesInRank[i] = { id: node2.id, node: swapped2 };
      nodesInRank[i + 1] = { id: node1.id, node: swapped1 };
      swapsApplied++;
    }
  }

  return result;
}

function isContained(child: LayoutNode, parent: LayoutNode): boolean {
  const padding = 20;
  return (
    child.bbox.x >= parent.bbox.x + padding &&
    child.bbox.y >= parent.bbox.y + padding &&
    child.bbox.x + child.bbox.width <= parent.bbox.x + parent.bbox.width - padding &&
    child.bbox.y + child.bbox.height <= parent.bbox.y + parent.bbox.height - padding
  );
}

function increaseSpacingForCrossingPairs(
  nodes: Map<string, LayoutNode>,
  edges: Array<{ id: string; sourceId: string; targetId: string; path: Point[] }>,
  minSpacing: number
): Map<string, LayoutNode> {
  const result = new Map(nodes);
  const nodePairs = new Map<string, number>();

  // Find crossing pairs
  for (let i = 0; i < edges.length; i++) {
    for (let j = i + 1; j < edges.length; j++) {
      const e1 = edges[i];
      const e2 = edges[j];
      if (pathsCross(e1.path, e2.path, e1.sourceId, e1.targetId, e2.sourceId, e2.targetId)) {
        const pairs = [
          [e1.sourceId, e2.sourceId],
          [e1.sourceId, e2.targetId],
          [e1.targetId, e2.sourceId],
          [e1.targetId, e2.targetId],
        ];
        for (const [n1, n2] of pairs) {
          if (n1 !== n2) {
            const key = n1 < n2 ? `${n1}-${n2}` : `${n2}-${n1}`;
            nodePairs.set(key, (nodePairs.get(key) || 0) + 1);
          }
        }
      }
    }
  }

  // Push apart crossing pairs
  for (const [key, crossingCount] of nodePairs) {
    if (!key || typeof key !== 'string') {
      console.warn(`[Optimization] Invalid key, skipping:`, key);
      continue;
    }

    const parts = key.split("-");
    if (parts.length !== 2) {
      console.warn(`[Optimization] Invalid key format, expected 'id1-id2', got:`, key);
      continue;
    }

    const [id1, id2] = parts;
    const node1 = result.get(id1);
    const node2 = result.get(id2);
    if (!node1 || !node2) continue;

    const dx = node2.bbox.x - node1.bbox.x;
    const dy = node2.bbox.y - node1.bbox.y;
    const dist = Math.hypot(dx, dy);
    const requiredDist = minSpacing + crossingCount * 15;

    if (dist < requiredDist && dist > 0) {
      const pushDist = Math.min((requiredDist - dist) / 2, 15);
      const pushX = (dx / dist) * pushDist;
      const pushY = (dy / dist) * pushDist;

      // Only push nodes that don't have parents (or verify containment)
      if (!node1.parent) {
        result.set(id1, updateNodePosition(node1, node1.bbox.x - pushX, node1.bbox.y - pushY));
      }
      if (!node2.parent) {
        result.set(id2, updateNodePosition(node2, node2.bbox.x + pushX, node2.bbox.y + pushY));
      }
    }
  }

  return result;
}

function pathsCross(
  path1: Point[],
  path2: Point[],
  from1?: string,
  to1?: string,
  from2?: string,
  to2?: string
): boolean {
  // Don't count as crossing if edges share a node
  if (from1 === from2 || from1 === to2 || to1 === from2 || to1 === to2) {
    return false;
  }

  for (let i = 0; i < path1.length - 1; i++) {
    for (let j = 0; j < path2.length - 1; j++) {
      if (lineSegmentsIntersect(path1[i], path1[i + 1], path2[j], path2[j + 1])) {
        return true;
      }
    }
  }
  return false;
}

function lineSegmentsIntersect(p1: Point, p2: Point, p3: Point, p4: Point): boolean {
  const det = (p2.x - p1.x) * (p4.y - p3.y) - (p4.x - p3.x) * (p2.y - p1.y);
  if (Math.abs(det) < 1e-10) return false;

  const lambda = ((p4.y - p3.y) * (p4.x - p1.x) + (p3.x - p4.x) * (p4.y - p1.y)) / det;
  const gamma = ((p1.y - p2.y) * (p4.x - p1.x) + (p2.x - p1.x) * (p4.y - p1.y)) / det;

  return 0 < lambda && lambda < 1 && 0 < gamma && gamma < 1;
}

// ============================================================================
// SPACING OPTIMIZATION
// ============================================================================

function applySpacingOptimization(context: LayoutContext): LayoutContext {
  const nodes = new Map(context.nodes);
  const minSpacing = context.options.spacing.nodePadding;

  // Ensure minimum spacing between all visible nodes
  const visibleNodes = Array.from(nodes.values()).filter((n) => n.visible && !n.parent);

  for (let i = 0; i < visibleNodes.length; i++) {
    const a = nodes.get(visibleNodes[i].id)!;

    for (let j = i + 1; j < visibleNodes.length; j++) {
      const b = nodes.get(visibleNodes[j].id)!;

      const dx = a.bbox.x + a.bbox.width / 2 - (b.bbox.x + b.bbox.width / 2);
      const dy = a.bbox.y + a.bbox.height / 2 - (b.bbox.y + b.bbox.height / 2);
      const dist = Math.hypot(dx, dy);

      const minDist =
        minSpacing +
        (Math.max(a.bbox.width, a.bbox.height) + Math.max(b.bbox.width, b.bbox.height)) / 2;

      if (dist < minDist && dist > 0) {
        const force = (minDist - dist) / 2;
        const fx = (dx / dist) * force;
        const fy = (dy / dist) * force;

        nodes.set(a.id, updateNodePosition(a, a.bbox.x + fx, a.bbox.y + fy));
        nodes.set(b.id, updateNodePosition(b, b.bbox.x - fx, b.bbox.y - fy));
      }
    }
  }

  return { ...context, nodes };
}

// ============================================================================
// LABEL PLACEMENT
// ============================================================================

function applyLabelPlacement(context: LayoutContext): LayoutContext {
  const edges = new Map(context.edges);
  const nodes = context.nodes;
  const PADDING = 8; // Minimum padding around nodes for label clearance

  // Adjust edge label positions to avoid node overlaps
  for (const [id, edge] of edges) {
    const labelBounds = edge.labelBounds;
    if (!labelBounds) continue;

    // Check if label overlaps with any node
    let needsAdjustment = false;
    let bestPosition = edge.labelPosition;

    for (const node of nodes.values()) {
      if (!node.visible) continue;
      if (node.id === edge.source.id || node.id === edge.target.id) continue;

      // Use padding to be more sensitive
      const nodeBoundsWithPadding: Rect = {
        x: node.bbox.x - PADDING,
        y: node.bbox.y - PADDING,
        width: node.bbox.width + PADDING * 2,
        height: node.bbox.height + PADDING * 2,
      };

      if (rectsOverlap(labelBounds, nodeBoundsWithPadding)) {
        needsAdjustment = true;
        break;
      }
    }

    if (needsAdjustment) {
      // Try more candidate positions in a spiral pattern
      const baseX = edge.labelPosition.x;
      const baseY = edge.labelPosition.y;
      const candidates: Array<{ x: number; y: number }> = [];

      // Generate candidates in expanding spiral
      for (let radius = 20; radius <= 100; radius += 20) {
        for (let angle = 0; angle < 360; angle += 45) {
          const rad = (angle * Math.PI) / 180;
          candidates.push({
            x: baseX + Math.cos(rad) * radius,
            y: baseY + Math.sin(rad) * radius,
          });
        }
      }

      // Also try positions along the edge path if available
      if (edge.points && edge.points.length > 2) {
        const midPoint = Math.floor(edge.points.length / 2);
        const point = edge.points[midPoint];
        candidates.push(
          { x: point.x, y: point.y - 30 },
          { x: point.x, y: point.y + 30 },
          { x: point.x - 40, y: point.y },
          { x: point.x + 40, y: point.y }
        );
      }

      // Find first non-overlapping position
      for (const candidate of candidates) {
        const candidateBounds: Rect = {
          x: candidate.x - labelBounds.width / 2,
          y: candidate.y - labelBounds.height / 2,
          width: labelBounds.width,
          height: labelBounds.height,
        };

        let overlaps = false;
        for (const node of nodes.values()) {
          if (!node.visible) continue;
          if (node.id === edge.source.id || node.id === edge.target.id) continue;

          const nodeBoundsWithPadding: Rect = {
            x: node.bbox.x - PADDING,
            y: node.bbox.y - PADDING,
            width: node.bbox.width + PADDING * 2,
            height: node.bbox.height + PADDING * 2,
          };

          if (rectsOverlap(candidateBounds, nodeBoundsWithPadding)) {
            overlaps = true;
            break;
          }
        }

        if (!overlaps) {
          bestPosition = candidate;
          break;
        }
      }
    }

    if (bestPosition !== edge.labelPosition) {
      edges.set(id, {
        ...edge,
        labelPosition: bestPosition,
        labelBounds: {
          x: bestPosition.x - labelBounds.width / 2,
          y: bestPosition.y - labelBounds.height / 2,
          width: labelBounds.width,
          height: labelBounds.height,
        },
      });
    }
  }

  return { ...context, edges };
}

function rectsOverlap(a: Rect, b: Rect): boolean {
  return !(
    a.x + a.width < b.x ||
    b.x + b.width < a.x ||
    a.y + a.height < b.y ||
    b.y + b.height < a.y
  );
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

function getTargetScore(grade: "A" | "B" | "C"): number {
  switch (grade) {
    case "A":
      return 90;
    case "B":
      return 80;
    case "C":
      return 70;
    default:
      return 80;
  }
}

function updateNodePosition(node: LayoutNode, x: number, y: number): LayoutNode {
  return {
    ...node,
    bbox: { ...node.bbox, x, y },
    contentBox: { ...node.contentBox, x: x + 10, y: y + 10 },
    labelBox: { ...node.labelBox, x: x + 10, y: y + 10 },
  };
}
