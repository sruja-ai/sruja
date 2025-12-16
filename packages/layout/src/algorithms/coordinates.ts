import type { C4Id } from "../brand";
import type { Rect as BBox } from "../geometry/rect";
import type { SizedNode } from "./sizing";
import type { C4LayoutOptions } from "../c4-options";
import type { HierarchyTree, HierarchyNode } from "./hierarchy";
import { layoutSugiyama } from "./sugiyama";
import { layoutL0 } from "./l0-layout";
import { layoutL1SystemContext } from "./l1-layout";

export interface PositionedNode extends SizedNode {
  x: number;
  y: number;
  bbox: BBox;
}

/**
 * Position roots using the appropriate top-level layout algorithm
 */
function positionRoots(
  roots: HierarchyNode[],
  sizes: Map<C4Id, SizedNode>,
  relationships: { from: C4Id; to: C4Id }[],
  options: C4LayoutOptions,
  tree: HierarchyTree
): Map<C4Id, { x: number; y: number }> {
  const rootPositions = new Map<C4Id, { x: number; y: number }>();

  // Aggregate relationships to root level
  const nodeToRoot = new Map<C4Id, C4Id>();
  function mapToRoot(node: HierarchyNode, rootId: C4Id) {
    nodeToRoot.set(node.id, rootId);
    for (const c of node.children) mapToRoot(c, rootId);
  }
  for (const root of roots) mapToRoot(root, root.id);

  const rootRels = new Set<string>();
  const aggregatedRels: { from: C4Id; to: C4Id }[] = [];
  for (const rel of relationships) {
    const rootFrom = nodeToRoot.get(rel.from);
    const rootTo = nodeToRoot.get(rel.to);
    if (rootFrom && rootTo && rootFrom !== rootTo) {
      const key = `${rootFrom}->${rootTo}`;
      if (!rootRels.has(key)) {
        rootRels.add(key);
        aggregatedRels.push({ from: rootFrom, to: rootTo });
      }
    }
  }

  // Choose layout strategy
  // Choose layout strategy
  if (options.strategy === "l1-context") {
    // L1 System Context: Star/Radial Layout
    // The new algorithm handles identifying center/satellites internally
    const l1Result = layoutL1SystemContext(
      tree,
      new Map([...sizes].map(([k, v]) => [k, v.size])),
      relationships,
      options
    );

    // Copy positions
    for (const [id, node] of l1Result) {
      if (rootPositions.has(id as C4Id)) continue; // Don't overwrite if already set (unlikely)
      rootPositions.set(id as C4Id, { x: node.bbox.x, y: node.bbox.y });
    }
  } else if (options.strategy === "incremental" && options.stability?.previousPositions) {
    // Incremental: Keep roots at previous positions
    for (const root of roots) {
      const prev = options.stability.previousPositions.get(root.id);
      if (prev) {
        rootPositions.set(root.id, { x: prev.x, y: prev.y });
      } else {
        // New root? Place at 0,0 (overlap removal will fix) or try to find a gap
        // For now, center at 0,0
        rootPositions.set(root.id, { x: 0, y: 0 });
      }
    }
  } else if (options.strategy === "grid") {
    // L0 Grid layout
    const rootNodes = roots.map((r) => ({ id: r.id, size: sizes.get(r.id)!.size }));
    const resultL0 = layoutL0(
      rootNodes,
      aggregatedRels,
      options,
      (id) => tree.nodeMap.get(id),
      (id) => sizes.get(id)
    );
    resultL0.nodes.forEach((n) => rootPositions.set(n.id, { x: n.x, y: n.y }));
  } else {
    // Default: Sugiyama hierarchical layout
    const rootNodes = roots.map((r) => ({ id: r.id, size: sizes.get(r.id)!.size }));
    const layoutResult = layoutSugiyama(rootNodes, aggregatedRels, options);
    layoutResult.nodes.forEach((n) => rootPositions.set(n.id, { x: n.x, y: n.y }));
  }

  return rootPositions;
}

export function assignCoordinates(
  tree: HierarchyTree,
  sizes: Map<C4Id, SizedNode>,
  relationships: { from: C4Id; to: C4Id }[],
  options: C4LayoutOptions
): Map<C4Id, PositionedNode> {
  const result = new Map<C4Id, PositionedNode>();
  const paddingMap = options.spacing.padding;

  // Safety margin to ensure children have adequate breathing room within parent
  // Increased for expanded nodes to ensure proper containment
  // Increased further to prevent any child nodes from being outside parent bounds
  // Increased to 100px to ensure children never overflow parent bounds and have good spacing
  const CONTAINMENT_BUFFER = 24;

  /**
   * Recursively position a node and all its children (top-down, pre-order)
   * @param node - The hierarchy node to position
   * @param absolutePos - The absolute position for this node
   */
  function positionNode(node: HierarchyNode, absolutePos: { x: number; y: number }): void {
    const sized = sizes.get(node.id)!;
    const padding = paddingMap[node.node.kind] ?? 16;

    // Create positioned node with absolute coordinates
    const positioned: PositionedNode = {
      ...sized,
      x: absolutePos.x,
      y: absolutePos.y,
      bbox: {
        x: absolutePos.x,
        y: absolutePos.y,
        width: sized.size.width,
        height: sized.size.height,
      },
    };
    result.set(node.id, positioned);

    // Position children if not collapsed
    const hasChildLayout = !!sized.childLayout;
    const hasValidPositions = hasChildLayout && (sized.childLayout?.positions?.size ?? 0) > 0;
    const isNotCollapsed = !node.node.collapseChildren;
    const hasChildren = node.children.length > 0;

    if (isNotCollapsed && hasChildren) {
      const headerHeight = sized.contentSize.height + padding;

      // Content area starts after header with additional safety buffer
      // Elastic padding: increases when many external edges attach or child density is high
      // (Section 3.3 of LayoutBestPractice.md - Parent Boundary Elasticity)
      const childCount = node.children.length;

      // Count external edges (edges connecting to/from this node that go outside parent)
      // Elastic padding increases when many external edges attach (Section 3.3 of LayoutBestPractice.md)
      let externalEdgeCount = 0;
      if (relationships && relationships.length > 0) {
        externalEdgeCount = relationships.filter((r) => {
          const fromNode = result.get(r.from as any);
          const toNode = result.get(r.to as any);
          // Count edges where this node is involved but the other node is outside this parent
          return (
            (fromNode?.id === node.id || toNode?.id === node.id) &&
            fromNode?.parent?.id !== node.id &&
            toNode?.parent?.id !== node.id
          );
        }).length;
      } else {
        // Estimate: nodes with many children likely have more external edges
        externalEdgeCount = Math.min(childCount, 10);
      }

      // Elastic padding: base + scale with child density + scale with external edges
      const baseBuffer = CONTAINMENT_BUFFER;
      const childDensityFactor = childCount > 5 ? 1.5 : childCount > 3 ? 1.25 : 1.1;
      const externalEdgeFactor = externalEdgeCount > 5 ? 1.2 : externalEdgeCount > 2 ? 1.1 : 1.0;
      const expandedBuffer = baseBuffer * childDensityFactor * externalEdgeFactor;
      const contentStartX = absolutePos.x + padding + expandedBuffer;
      const contentStartY = absolutePos.y + headerHeight + expandedBuffer;

      // Check if we have valid child layout positions
      if (hasValidPositions && sized.childLayout?.positions) {
        // First, find the min offset in child positions to normalize them
        let minX = Infinity;
        let minY = Infinity;
        let foundAnyPosition = false;
        for (const child of node.children) {
          const relativePos = sized.childLayout.positions.get(child.id);
          if (relativePos) {
            foundAnyPosition = true;
            minX = Math.min(minX, relativePos.x);
            minY = Math.min(minY, relativePos.y);
          }
        }

        // If we found at least some positions, use them
        if (foundAnyPosition && minX !== Infinity && minY !== Infinity) {
          // Normalize: shift all positions so minX/minY are 0
          const offsetX = minX < 0 ? -minX : 0;
          const offsetY = minY < 0 ? -minY : 0;

          // Position each child using normalized relative positions
          for (const child of node.children) {
            const relativePos = sized.childLayout.positions.get(child.id);
            if (relativePos) {
              const childAbsolutePos = {
                x: contentStartX + relativePos.x + offsetX,
                y: contentStartY + relativePos.y + offsetY,
              };
              positionNode(child, childAbsolutePos);
            } else {
              // Fallback: if this specific child has no position, use a default offset
              console.warn(
                `[COORDS] No position for child ${child.id} of ${node.id}, using default offset`
              );
              const childIndex = node.children.indexOf(child);
              const childAbsolutePos = {
                x: contentStartX + (childIndex % 3) * 60,
                y: contentStartY + Math.floor(childIndex / 3) * 60,
              };
              positionNode(child, childAbsolutePos);
            }
          }
        } else {
          // No valid positions found, use grid fallback
          console.warn(`[COORDS] No valid child positions for ${node.id}, using grid fallback`);
          const cols = Math.ceil(Math.sqrt(node.children.length));
          const childSpacing = 150;
          let childIndex = 0;
          for (const child of node.children) {
            const col = childIndex % cols;
            const row = Math.floor(childIndex / cols);
            const childAbsolutePos = {
              x: contentStartX + col * childSpacing,
              y: contentStartY + row * childSpacing,
            };
            positionNode(child, childAbsolutePos);
            childIndex++;
          }
        }
      } else {
        // No childLayout or positions at all, use grid fallback
        console.warn(`[COORDS] No childLayout for ${node.id}, using grid fallback`);
        const cols = Math.ceil(Math.sqrt(node.children.length));
        const childSpacing = 150;
        let childIndex = 0;
        for (const child of node.children) {
          const col = childIndex % cols;
          const row = Math.floor(childIndex / cols);
          const childAbsolutePos = {
            x: contentStartX + col * childSpacing,
            y: contentStartY + row * childSpacing,
          };
          positionNode(child, childAbsolutePos);
          childIndex++;
        }
      }

      // After positioning children, verify containment and resize parent if needed
      // Calculate bounds of all children (including grandchildren recursively)
      let maxChildRight = absolutePos.x;
      let maxChildBottom = absolutePos.y;
      let minChildLeft = Infinity;
      let minChildTop = Infinity;

      function collectChildBounds(childNode: HierarchyNode) {
        const childPos = result.get(childNode.id);
        if (childPos) {
          minChildLeft = Math.min(minChildLeft, childPos.bbox.x);
          minChildTop = Math.min(minChildTop, childPos.bbox.y);
          maxChildRight = Math.max(maxChildRight, childPos.bbox.x + childPos.bbox.width);
          maxChildBottom = Math.max(maxChildBottom, childPos.bbox.y + childPos.bbox.height);

          // Also check grandchildren recursively
          for (const grandchild of childNode.children) {
            collectChildBounds(grandchild);
          }
        }
      }

      for (const child of node.children) {
        collectChildBounds(child);
      }

      // Ensure we have valid bounds
      if (minChildLeft === Infinity) {
        minChildLeft = absolutePos.x;
        minChildTop = absolutePos.y;
      }

      // Calculate required parent size with additional safety buffer
      // Use expanded buffer for nodes with many children
      // Add extra margin to ensure children never touch parent edges
      // Account for children that might extend to the left/top of the parent's origin
      const leftOffset = Math.max(0, absolutePos.x - minChildLeft);
      const topOffset = Math.max(0, absolutePos.y - minChildTop);
      // Increased safety margin to match MIN_PARENT_PADDING (80px) in diagramQuality.ts
      // Scale margin based on child count to handle larger expansions
      const MIN_QUALITY_PADDING = 32; // Reduced from 80
      const extraMargin = Math.max(
        MIN_QUALITY_PADDING,
        childCount > 5 ? 60 : childCount > 3 ? 48 : 40
      );
      const requiredWidth = Math.max(
        positioned.bbox.width,
        maxChildRight - absolutePos.x + padding + expandedBuffer + extraMargin + leftOffset
      );
      const requiredHeight = Math.max(
        positioned.bbox.height,
        maxChildBottom - absolutePos.y + padding + expandedBuffer + extraMargin + topOffset
      );

      // Always resize parent to ensure containment - don't just check if it exceeds
      positioned.bbox.width = requiredWidth;
      positioned.bbox.height = requiredHeight;
      positioned.size = {
        width: requiredWidth,
        height: requiredHeight,
      };
      // Update in result map
      result.set(node.id, positioned);
    }
  }

  // Step 1: Position all roots
  const rootPositions = positionRoots(tree.roots, sizes, relationships, options, tree);

  // Step 2: Recursively position each root and its descendants
  for (const root of tree.roots) {
    const pos = rootPositions.get(root.id);
    if (pos) {
      positionNode(root, pos);
    }
  }

  return result;
}
