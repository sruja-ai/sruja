import type { Node, Edge } from "@xyflow/react";
import type { C4NodeData } from "../types";

export interface LayoutOptions {
  direction?: "DOWN" | "RIGHT" | "UP" | "LEFT";
  level?: string;
  expandedNodes?: Set<string>;
  optimizationRules?: OptimizationRuleConfig; // Rule-based optimization config
  strategy?: "sugiyama" | "grid" | "l1-context" | "incremental"; // Added strategy
}

interface LayoutResult {
  nodes: Node<C4NodeData>[];
  edges: Edge[];
}

export function getNodeWidth(type?: string): number {
  switch (type) {
    case "person":
      return 180; // Increased from 120 to accommodate icon + longer labels
    case "system":
      return 240; // Increased from 200 for better label fit
    case "container":
      return 220; // Increased from 180 for better label fit
    case "component":
      return 200; // Increased from 160 for better label fit
    case "datastore":
      return 200; // Increased from 160
    case "queue":
      return 200; // Increased from 160
    case "deployment":
      return 280; // Larger for containing nodes
    default:
      return 220; // Increased from 180
  }
}

export function getNodeHeight(type?: string): number {
  switch (type) {
    case "person":
      return 160; // Increased from 140 to prevent label clipping (icon ~40px + label ~20px + padding)
    case "system":
      return 140; // Increased from 120 (label ~20px + description ~22px + technology ~22px + padding)
    case "container":
      return 120; // Increased from 100 for better content fit
    case "component":
      return 100; // Increased from 80 for better label fit
    case "datastore":
      return 100; // Increased from 80
    case "queue":
      return 100; // Increased from 80
    case "deployment":
      return 200; // Larger height
    default:
      return 120; // Increased from 100
  }
}

// @sruja/layout Adapter
import {
  applyOptimizationRules,
  type OptimizationRuleConfig,
  DEFAULT_OPTIMIZATION_RULES,
} from "./optimizationRules";
import {
  createC4Graph,
  layout,
  createDefaultViewState,
  createC4Id,
  InteractivePreset,
  type C4Node,
  type C4Relationship,
  type C4Level,
  type C4Kind,
} from "@sruja/layout";

// Remove the broken import
// import type { TextMeasurer, Size } from '@sruja/layout/dist/utils/text-measurer';

export interface ITextMeasurer {
  measure(
    text: string,
    kind: C4Kind,
    level: C4Level,
    maxWidth?: number
  ): { width: number; height: number };
  measureMultiline(
    text: string,
    kind: C4Kind,
    level: C4Level,
    maxWidth: number
  ): { width: number; height: number; lines: string[] };
  getLineHeight(kind: C4Kind, level: C4Level): number;
  getDescent(kind: C4Kind, level: C4Level): number;
}

export class BrowserTextMeasurer implements ITextMeasurer {
  private ctx: CanvasRenderingContext2D;

  constructor() {
    const canvas = document.createElement("canvas");
    this.ctx = canvas.getContext("2d")!;
  }

  private setFont(isBold: boolean, fontSize: number) {
    // Use the actual font stack from CSS
    this.ctx.font = `${isBold ? "600" : "400"} ${fontSize}px "Inter", system-ui, sans-serif`;
  }

  measure(
    text: string,
    _kind: C4Kind,
    _level: C4Level,
    _maxWidth?: number
  ): { width: number; height: number } {
    // Assume worst case (Title) - Bold 14px
    this.setFont(true, 14);
    const m = this.ctx.measureText(text);
    // Add safety buffer for padding (12px each side) to prevent clipping
    return { width: m.width + 24, height: 22 }; // Increased height for better line spacing
  }

  measureMultiline(
    text: string,
    _kind: C4Kind,
    _level: C4Level,
    maxWidth: number
  ): { width: number; height: number; lines: string[] } {
    // Heuristic: If it's short, it's likely a label/title -> 14px Bold
    // If it's long, it's likely description -> 12px Regular (CSS says 11px, we use 12 for safety)
    const isLikelyTitle = text.length < 50 && !text.includes("\n");

    const fontSize = isLikelyTitle ? 14 : 12;
    const isBold = isLikelyTitle;
    const lineHeight = isLikelyTitle ? 20 : 16;

    this.setFont(isBold, fontSize);

    const words = text.split(/\s+/);
    const lines: string[] = [];
    let currentLine = words[0];

    // Basic wrapping logic matching sizing.ts
    for (let i = 1; i < words.length; i++) {
      const word = words[i];
      const width = this.ctx.measureText(currentLine + " " + word).width;
      if (width < maxWidth) {
        currentLine += " " + word;
      } else {
        lines.push(currentLine);
        currentLine = word;
      }
    }
    lines.push(currentLine);

    // Calculate max width of lines
    let maxLineWidth = 0;
    for (const line of lines) {
      maxLineWidth = Math.max(maxLineWidth, this.ctx.measureText(line).width);
    }

    return {
      width: maxLineWidth + 24, // Increased safety padding (12px each side) to prevent clipping
      height: lines.length * lineHeight + 8, // Add extra vertical padding
      lines,
    };
  }

  getLineHeight(_kind: C4Kind, _level: C4Level): number {
    return 22; // Generous line height
  }

  getDescent(_kind: C4Kind, _level: C4Level): number {
    return 6;
  }
}

export interface LayoutOptions {
  direction?: "DOWN" | "RIGHT" | "UP" | "LEFT";
  level?: string;
  expandedNodes?: Set<string>;
  optimizationRules?: OptimizationRuleConfig; // Rule-based optimization config
  strategy?: "sugiyama" | "grid" | "l1-context" | "incremental"; // Added strategy
}

export async function applySrujaLayout(
  nodes: Node<C4NodeData>[],
  edges: Edge[],
  options: LayoutOptions = {}
): Promise<LayoutResult> {
  const { direction = "DOWN", optimizationRules = DEFAULT_OPTIMIZATION_RULES } = options;

  // Pass previous positions for stability
  const previousPositions = new Map<string, { x: number; y: number }>();
  let hasExistingLayout = false;
  nodes.forEach((n) => {
    previousPositions.set(n.id, { x: n.position.x, y: n.position.y });
    // Use a small epsilon to detect if layout happened (default 0,0)
    if (Math.abs(n.position.x) > 1 || Math.abs(n.position.y) > 1) hasExistingLayout = true;
  });

  // Heuristic: If we have an existing layout (positions are not all 0,0),
  // and the requested strategy is default (undefined/sugiyama), switch to 'incremental'
  // to preserve stability ("Expand in Place").
  let strategy = options.strategy;
  if (!strategy && hasExistingLayout) {
    strategy = "incremental";
    console.log("[Layout] Switching to INCREMENTAL strategy for stability");
  }

  // Build a map of all nodes to understand parent-child relationships
  const nodeChildrenMap = new Map<string, Node<C4NodeData>[]>();
  nodes.forEach((n) => {
    if (n.parentId) {
      if (!nodeChildrenMap.has(n.parentId)) {
        nodeChildrenMap.set(n.parentId, []);
      }
      nodeChildrenMap.get(n.parentId)!.push(n);
    }
  });

  // Since jsonToReactFlow already filters nodes based on expansion state,
  // all nodes passed here should be visible. We just need to track which ones are expanded
  // for the layout engine to know which nodes have their children visible.
  const expandedNodeIds = new Set<string>();
  nodes.forEach((n) => {
    if (n.data.expanded === true) {
      expandedNodeIds.add(n.id);
    }
  });

  // All nodes passed to layout are already visible (filtered by jsonToReactFlow)
  const visibleNodes = nodes;
  const visibleNodeIdSet = new Set(visibleNodes.map((n) => n.id));
  const visibleEdges = edges.filter(
    (e) => visibleNodeIdSet.has(e.source) && visibleNodeIdSet.has(e.target)
  );

  // 1. Convert to C4Graph - all nodes are visible
  const c4Nodes: C4Node[] = visibleNodes.map((n) => {
    const hasChildren = nodeChildrenMap.has(n.id);
    const isExpanded = expandedNodeIds.has(n.id);
    const shouldCollapse = hasChildren && !isExpanded;

    const c4Node: C4Node = {
      id: createC4Id(n.id),
      label: n.data.label || n.id,
      kind: mapTypeToKind(n.data.type),
      level: mapTypeToLevel(n.data.type),
      parentId: n.parentId && visibleNodeIdSet.has(n.parentId) ? createC4Id(n.parentId) : undefined,
      description: n.data.description,
      technology: n.data.technology,
      tags: new Set((n.data.tags as unknown as string[]) || []),
      collapseChildren: shouldCollapse,
    };

    if (n.parentId) {
      console.log(
        `  C4Node ${n.id}: parentId=${c4Node.parentId}, collapseChildren=${shouldCollapse}, hasChildren=${hasChildren}, isExpanded=${isExpanded}`
      );
    }

    return c4Node;
  });

  console.log(
    `C4Nodes with parentId:`,
    c4Nodes.filter((n) => n.parentId).map((n) => `${n.id} -> ${n.parentId}`)
  );

  const c4Relationships: C4Relationship[] = visibleEdges.map((e) => ({
    id: e.id,
    from: createC4Id(e.source),
    to: createC4Id(e.target),
    label: (e.label as string) || (e.data?.label as string) || "",
  }));

  try {
    const graph = createC4Graph(c4Nodes, c4Relationships);
    const view = createDefaultViewState();

    // Set collapsed nodes in view state (nodes that have children but are not expanded)
    const collapsedC4Ids = new Set(
      Array.from(visibleNodes)
        .filter((n) => !expandedNodeIds.has(n.id) && nodeChildrenMap.has(n.id))
        .map((n) => createC4Id(n.id))
    );
    view.collapsedNodeIds = collapsedC4Ids;

    // Start with base layout options
    const baseLayoutOptions: any = {
      ...InteractivePreset,
      direction: direction === "RIGHT" ? "LR" : direction === "LEFT" ? "RL" : "TB",
      maxSize: { width: 5000, height: 5000 },
      measurer: (options as any).measurer ?? new BrowserTextMeasurer(),
      strategy, // Explicitly set strategy
      stability: { previousPositions }, // Pass prev positions
    };

    // Apply rule-based optimizations
    const layoutOptions = applyOptimizationRules(baseLayoutOptions, optimizationRules);

    // 2. Run Layout

    const result = layout(graph, view, layoutOptions);
    // layout() is synchronous. If we want async, layoutAsync()

    // 3. Convert back to React Flow - only include nodes that were laid out
    const updatedNodes: Node<C4NodeData>[] = [];
    const missingPositions: string[] = [];

    visibleNodes.forEach((n) => {
      const c4Id = createC4Id(n.id);
      const pos = result.nodes.get(c4Id);
      if (pos) {
        if (!pos.visible) {
          console.warn(`Node ${n.id} is not visible in layout result (but was in input)`);
          // Still include it - visibility might be for rendering hints, not filtering
        }

        // For nodes with parentId, React Flow expects relative positioning
        let position = { x: pos.bbox.x, y: pos.bbox.y };
        const isChild = !!n.parentId;

        if (isChild) {
          const parentC4Id = createC4Id(n.parentId!);
          const parent = result.nodes.get(parentC4Id);
          if (parent) {
            // Convert absolute to relative position
            const absPos = { x: pos.bbox.x, y: pos.bbox.y };
            position = {
              x: pos.bbox.x - parent.bbox.x,
              y: pos.bbox.y - parent.bbox.y,
            };
            console.log(
              `[LAYOUT] Child ${n.id}: absolute(${absPos.x.toFixed(0)}, ${absPos.y.toFixed(0)}) -> relative(${position.x.toFixed(0)}, ${position.y.toFixed(0)}) within parent ${n.parentId} at (${parent.bbox.x.toFixed(0)}, ${parent.bbox.y.toFixed(0)})`
            );
          } else {
            console.warn(
              `[LAYOUT] Parent ${n.parentId} (${parentC4Id}) not found in layout result for node ${n.id}, using absolute position`
            );
            // Use absolute position if parent not found
          }
        } else {
          console.log(
            `[LAYOUT] Root ${n.id}: position (${position.x.toFixed(0)}, ${position.y.toFixed(0)}), size (${pos.bbox.width.toFixed(0)}x${pos.bbox.height.toFixed(0)})`
          );
        }

        // Ensure valid dimensions - use minimum sizes to prevent clipping
        const minWidth = getNodeWidth(n.data?.type);
        const minHeight = getNodeHeight(n.data?.type);
        const width = Math.max(minWidth, pos.bbox.width);
        const height = Math.max(minHeight, pos.bbox.height);

        updatedNodes.push({
          ...n,
          position,
          width,
          height,
          style: {
            ...n.style,
            width,
            height,
          },
        });
      } else {
        missingPositions.push(n.id);
        console.warn(`No layout position found for node ${n.id} (C4Id: ${c4Id}), skipping`);
      }
    });

    if (missingPositions.length > 0) {
      console.error(`Missing positions for ${missingPositions.length} nodes:`, missingPositions);
    }

    // React Flow REQUIREMENT: Parents must verify before children in the array
    // We sort by level (L1 -> L2 -> L3) to ensure hierarchy
    const finalNodes = updatedNodes.sort((a, b) => {
      // If b is a child of a, a comes first
      if (b.parentId === a.id) return -1;
      if (a.parentId === b.id) return 1;

      // Otherwise sort by depth/nesting (less nested first)
      const depthA = getDepth(a.id, updatedNodes);
      const depthB = getDepth(b.id, updatedNodes);
      return depthA - depthB;
    });

    // Post-process: Ensure all nodes have adequate size for their content
    const finalNodesWithContentSizing = ensureNodeContentSizing(finalNodes, optimizationRules);

    // Post-process: Ensure parent nodes are properly sized to contain children
    const finalNodesWithSizedParents = ensureParentChildSizing(
      finalNodesWithContentSizing,
      optimizationRules
    );

    // Validate the final layout
    validateGraph(finalNodesWithSizedParents);

    // 4. Update Edges to use Custom Polyline with Points
    const relMap = new Map(result.relationships.map((r) => [r.relationshipId, r]));

    const finalEdges = visibleEdges.map((e) => {
      const rel = relMap.get(e.id);
      // Preserve label from original edge
      const edgeLabel = e.label || (e.data?.label as string | undefined);

      if (rel && rel.points) {
        // Adjust label position to avoid overlapping with nodes (if enabled in rules)
        let adjustedLabelPosition = rel.labelPosition;
        if (rel.labelPosition && optimizationRules.labelPositioning.enabled) {
          adjustedLabelPosition = adjustEdgeLabelPosition(
            rel.labelPosition,
            finalNodesWithSizedParents,
            e.source,
            e.target,
            optimizationRules
          );
        }

        return {
          ...e,
          label: edgeLabel, // Ensure label is preserved
          type: "relation",
          data: {
            ...e.data,
            points: rel.points,
            label: edgeLabel,
            labelPosition: adjustedLabelPosition,
          },
          animated: false,
          style: { ...e.style }, // Preserve original style, don't overwrite with default gray here
        };
      }
      // Even if no layout points, preserve label
      return {
        ...e,
        label: edgeLabel,
        data: {
          ...e.data,
          label: edgeLabel,
        },
      };
    });

    return { nodes: finalNodesWithSizedParents, edges: finalEdges };
  } catch (error) {
    console.error("Sruja Layout failed:", error);
    return { nodes, edges };
  }
}

/**
 * Validates that the graph layout is correct, specifically checking:
 * 1. That all child nodes are strictly contained within their parent's bounding box.
 * 2. That no nodes are overlapping (optional, but good for future).
 */
function validateGraph(nodes: Node<C4NodeData>[]) {
  const nodeMap = new Map(nodes.map((n) => [n.id, n]));
  const violations: string[] = [];

  nodes.forEach((child) => {
    if (child.parentId && nodeMap.has(child.parentId)) {
      const parent = nodeMap.get(child.parentId)!;

      // React Flow positions are relative for children!
      // So child.position is {x, y} relative to parent top-left.
      // Parent's local coordinate system starts at (0,0) and extends to (parent.width, parent.height).

      const cLeft = child.position.x;
      const cRight = child.position.x + (child.width ?? 0);
      const cTop = child.position.y;
      const cBottom = child.position.y + (child.height ?? 0);

      const pWidth = parent.width ?? 0;
      const pHeight = parent.height ?? 0;

      if (cLeft < 0 || cTop < 0 || cRight > pWidth || cBottom > pHeight) {
        violations.push(`Layout Violation: Child '${child.data.label}' (${child.id}) is outside Parent '${parent.data.label}' (${parent.id}).
                  Child Relative: [${cLeft}, ${cTop}, ${cRight}, ${cBottom}]
                  Parent Size: [0, 0, ${pWidth}, ${pHeight}]
                  Overflow: Left=${Math.min(0, cLeft)}, Top=${Math.min(0, cTop)}, Right=${Math.max(0, cRight - pWidth)}, Bottom=${Math.max(0, cBottom - pHeight)}`);
      }
    }
  });

  if (violations.length > 0) {
    console.error("Layout Validation Failed:\n" + violations.join("\n"));
  } else {
    console.log("Layout Validation Passed: All nodes contained.");
  }
}

function getDepth(nodeId: string, nodes: Node[]): number {
  let depth = 0;
  let current = nodes.find((n) => n.id === nodeId);
  while (current && current.parentId) {
    depth++;
    current = nodes.find((n) => n.id === current?.parentId);
  }
  return depth;
}

/**
 * Ensure all nodes have adequate size for their content (labels, descriptions, technology tags)
 * Prevents label clipping
 * Uses optimization rules for node sizing configuration
 */
export function ensureNodeContentSizing(
  nodes: Node<C4NodeData>[],
  rules: OptimizationRuleConfig = DEFAULT_OPTIMIZATION_RULES
): Node<C4NodeData>[] {
  const updatedNodes = nodes.map((n) => {
    const type = n.data?.type;
    const label = n.data?.label || "";
    const description = n.data?.description || "";
    const technology = n.data?.technology || "";

    // Calculate required dimensions based on content
    const hasIcon = type === "person";
    const iconSpace = hasIcon ? 40 : 0;

    // Estimate content dimensions
    const labelWidth = Math.max(label.length * 7, 60); // Minimum label width
    const labelHeight = 22;

    const descriptionHeight = description ? 22 : 0; // ~22px per line (max 3 lines = 66px)
    const technologyHeight = technology ? 22 : 0;

    // Calculate total content height
    const contentHeight = labelHeight + descriptionHeight + technologyHeight;

    // Use content padding from optimization rules
    const contentPadding = rules.nodeSizing.enabled ? rules.nodeSizing.contentPadding : 24;

    const requiredWidth = Math.max(
      rules.nodeSizing.enabled ? rules.nodeSizing.minWidth : getNodeWidth(type),
      iconSpace + labelWidth + contentPadding, // icon + label + padding
      n.width || getNodeWidth(type)
    );

    const requiredHeight = Math.max(
      rules.nodeSizing.enabled ? rules.nodeSizing.minHeight : getNodeHeight(type),
      contentHeight + contentPadding, // content + padding
      n.height || getNodeHeight(type)
    );

    return {
      ...n,
      width: requiredWidth,
      height: requiredHeight,
      style: {
        ...n.style,
        width: requiredWidth,
        height: requiredHeight,
      },
    };
  });

  return updatedNodes;
}

/**
 * Ensure parent nodes are properly sized to contain their children
 * Prevents horizontal/vertical distortion
 * Uses optimization rules for containment configuration
 */
export function ensureParentChildSizing(
  nodes: Node<C4NodeData>[],
  rules: OptimizationRuleConfig = DEFAULT_OPTIMIZATION_RULES
): Node<C4NodeData>[] {
  const updatedNodes = nodes.map((n) => ({ ...n }));

  // Group children by parent
  const childrenByParent = new Map<string, Node<C4NodeData>[]>();
  updatedNodes.forEach((node) => {
    if (node.parentId) {
      if (!childrenByParent.has(node.parentId)) {
        childrenByParent.set(node.parentId, []);
      }
      childrenByParent.get(node.parentId)!.push(node);
    }
  });

  // Process parents from deepest to shallowest (to handle nested parents)
  const parentIds = Array.from(childrenByParent.keys());
  const sortedParentIds = parentIds.sort((a, b) => {
    const depthA = getDepth(a, updatedNodes);
    const depthB = getDepth(b, updatedNodes);
    return depthB - depthA; // Deeper first
  });

  sortedParentIds.forEach((parentId) => {
    const parentIndex = updatedNodes.findIndex((n) => n.id === parentId);
    const parent = updatedNodes[parentIndex];
    const children = childrenByParent.get(parentId) || [];

    if (!parent || children.length === 0) return;

    // Calculate required bounds to contain all children
    // Use padding from optimization rules or default
    const MIN_PARENT_PADDING = 50;
    const padding = rules.containment.enabled
      ? rules.containment.minParentPadding * rules.containment.paddingMultiplier
      : MIN_PARENT_PADDING * 2.0;

    const headerHeight = 60; // Space for label/header

    // 1. Calculate current bounding box of children
    let minChildX = Infinity;
    let maxChildX = -Infinity;
    let minChildY = Infinity;
    let maxChildY = -Infinity;

    children.forEach((child) => {
      const childX = child.position.x;
      const childY = child.position.y;
      const childWidth = child.width || 100;
      const childHeight = child.height || 100;

      minChildX = Math.min(minChildX, childX);
      maxChildX = Math.max(maxChildX, childX + childWidth);
      minChildY = Math.min(minChildY, childY);
      maxChildY = Math.max(maxChildY, childY + childHeight);
    });

    // 2. Shift children if they are too close to edges or negative
    // We want minChildX to be at least `padding`
    // We want minChildY to be at least `headerHeight + padding`
    const targetMinX = padding;
    const targetMinY = headerHeight + padding;

    const shiftX = targetMinX - minChildX;
    const shiftY = targetMinY - minChildY;

    // Apply shift if significant
    if (Math.abs(shiftX) > 1 || Math.abs(shiftY) > 1) {
      children.forEach((child) => {
        child.position.x += shiftX;
        child.position.y += shiftY;
        // Note: we are mutating the objects in updatedNodes array directly
      });
      // Update bounds
      maxChildX += shiftX;
      maxChildY += shiftY;
      // minChildX/Y are now targetMinX/Y
    }

    // 3. Calculate required parent size
    // Width = maxChildX + padding
    // Height = maxChildY + padding
    const minParentWidth = getNodeWidth(parent.data?.type);
    const minParentHeight = getNodeHeight(parent.data?.type);

    const requiredWidth = Math.max(
      minParentWidth,
      parent.width || minParentWidth,
      maxChildX + padding
    );
    const requiredHeight = Math.max(
      minParentHeight,
      parent.height || minParentHeight,
      maxChildY + padding
    );

    // Update parent size
    updatedNodes[parentIndex] = {
      ...updatedNodes[parentIndex],
      width: requiredWidth,
      height: requiredHeight,
      style: {
        ...updatedNodes[parentIndex].style,
        width: requiredWidth,
        height: requiredHeight,
      },
    };
  });

  return updatedNodes;
}

/**
 * Adjust edge label position to avoid overlapping with nodes
 */
export function adjustEdgeLabelPosition(
  labelPos: { x: number; y: number },
  nodes: Node<C4NodeData>[],
  sourceId: string,
  targetId: string,
  rules: OptimizationRuleConfig = DEFAULT_OPTIMIZATION_RULES
): { x: number; y: number } {
  if (!rules.labelPositioning.enabled || !rules.labelPositioning.adjustEdgeLabels) {
    return labelPos; // Return original if disabled
  }

  const MIN_LABEL_DISTANCE = rules.labelPositioning.minLabelDistance;
  const MAX_ADJUSTMENT = rules.labelPositioning.maxAdjustment;
  const LABEL_WIDTH = 80; // Estimated label width
  const LABEL_HEIGHT = 44; // Estimated label height (stacked: label + technology)

  let adjustedX = labelPos.x;
  let adjustedY = labelPos.y;
  const originalX = labelPos.x;
  const originalY = labelPos.y;

  // Check all nodes (excluding source and target edges connect to)
  nodes.forEach((node) => {
    if (node.id === sourceId || node.id === targetId) return;

    // Calculate absolute position
    let nodeLeft = node.position.x;
    let nodeTop = node.position.y;

    if (node.parentId) {
      const parent = nodes.find((n) => n.id === node.parentId);
      if (parent) {
        nodeLeft = parent.position.x + node.position.x;
        nodeTop = parent.position.y + node.position.y;
      }
    }

    const nodeRight = nodeLeft + (node.width || 100);
    const nodeBottom = nodeTop + (node.height || 100);

    // Expand node bounds by minimum distance
    const expandedLeft = nodeLeft - MIN_LABEL_DISTANCE;
    const expandedRight = nodeRight + MIN_LABEL_DISTANCE;
    const expandedTop = nodeTop - MIN_LABEL_DISTANCE;
    const expandedBottom = nodeBottom + MIN_LABEL_DISTANCE;

    // Check if label overlaps with expanded node bounds
    const labelLeft = adjustedX - LABEL_WIDTH / 2;
    const labelRight = adjustedX + LABEL_WIDTH / 2;
    const labelTop = adjustedY - LABEL_HEIGHT / 2;
    const labelBottom = adjustedY + LABEL_HEIGHT / 2;

    if (
      !(
        labelRight < expandedLeft ||
        labelLeft > expandedRight ||
        labelBottom < expandedTop ||
        labelTop > expandedBottom
      )
    ) {
      // Label overlaps - shift it away
      const dx = adjustedX - (nodeLeft + nodeRight) / 2;
      const dy = adjustedY - (nodeTop + nodeBottom) / 2;
      const distance = Math.sqrt(dx * dx + dy * dy);

      if (distance > 0) {
        // Move label away from node center, but limit adjustment
        const moveDistance = Math.min(MIN_LABEL_DISTANCE + LABEL_WIDTH / 2, MAX_ADJUSTMENT);
        const newX = (nodeLeft + nodeRight) / 2 + (dx / distance) * moveDistance;
        const newY = (nodeTop + nodeBottom) / 2 + (dy / distance) * moveDistance;

        // Only use if it's not too far from original position
        const adjustmentDistance = Math.sqrt(
          Math.pow(newX - originalX, 2) + Math.pow(newY - originalY, 2)
        );
        if (adjustmentDistance <= MAX_ADJUSTMENT) {
          adjustedX = newX;
          adjustedY = newY;
        }
      }
    }
  });

  return { x: adjustedX, y: adjustedY };
}

export function mapTypeToKind(type?: string): C4Kind {
  switch (type) {
    case "person":
      return "Person";
    case "system":
      return "SoftwareSystem";
    case "container":
      return "Container";
    case "component":
      return "Component";
    case "datastore":
      return "Database"; // Mapped to C4Kind
    case "queue":
      return "Queue";
    case "boundary":
      return "EnterpriseBoundary" as any;
    case "deployment":
      return "DeploymentNode" as any;
    default:
      return "SoftwareSystem";
  }
}

function mapTypeToLevel(type?: string): C4Level {
  // This is heuristic.
  switch (type) {
    case "person":
    case "system":
      return "context";
    case "container":
    case "datastore":
    case "queue":
      return "container";
    case "deployment":
      return "deployment" as any;
    case "component":
      return "component";
    default:
      return "context";
  }
}
