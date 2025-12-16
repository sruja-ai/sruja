import type { C4Id } from "../brand";
import type { Rect as BBox } from "../geometry/rect";
import type { SizedNode } from "./sizing";
import type { HierarchyNode } from "./hierarchy";
import type { C4LayoutOptions } from "../c4-options";
import type { PositionedNode } from "./coordinates";
import { isContainerKind } from "../c4-model";

// L0 Layout Constants
const SYSTEM_WIDTH = 420;
const SYSTEM_HEIGHT = 260;
const SYSTEM_H_SPACING = 300;
const SYSTEM_V_SPACING = 240;
const BADGE_SIZE = 18;
const BADGE_H_SPACING = 10;
const BADGE_V_SPACING = 10;
const BOUNDARY_PADDING = 24;

/**
 * Calculate relationship-based clustering for L0 landscape layout.
 * Groups systems that communicate frequently to minimize edge crossings.
 */
function clusterByRelationships(
  rootNodes: { id: C4Id; size: { width: number; height: number } }[],
  relationships: { from: C4Id; to: C4Id }[]
): { id: C4Id; size: { width: number; height: number } }[] {
  if (rootNodes.length <= 2 || relationships.length === 0) {
    return rootNodes; // No clustering needed
  }

  // Build adjacency count for each node
  const edgeCount = new Map<string, number>();
  const connections = new Map<string, Set<string>>();

  // Optimized: cache node ID strings to avoid repeated String() calls
  const nodeIdMap = new Map<C4Id, string>();
  for (const node of rootNodes) {
    const nodeIdStr = String(node.id);
    nodeIdMap.set(node.id, nodeIdStr);
    edgeCount.set(nodeIdStr, 0);
    connections.set(nodeIdStr, new Set());
  }

  for (const rel of relationships) {
    // Optimized: use cached string IDs or convert once
    const fromStr = nodeIdMap.get(rel.from) ?? String(rel.from);
    const toStr = nodeIdMap.get(rel.to) ?? String(rel.to);
    edgeCount.set(fromStr, (edgeCount.get(fromStr) || 0) + 1);
    edgeCount.set(toStr, (edgeCount.get(toStr) || 0) + 1);
    connections.get(fromStr)?.add(toStr);
    connections.get(toStr)?.add(fromStr);
  }

  // Sort nodes: high-connectivity nodes first (they become cluster centers)
  // Optimized: use cached string IDs
  const sorted = [...rootNodes].sort((a, b) => {
    const aIdStr = nodeIdMap.get(a.id) ?? String(a.id);
    const bIdStr = nodeIdMap.get(b.id) ?? String(b.id);
    const countA = edgeCount.get(aIdStr) || 0;
    const countB = edgeCount.get(bIdStr) || 0;
    return countB - countA; // Descending
  });

  // Reorder to keep connected nodes adjacent
  const result: typeof rootNodes = [];
  const placed = new Set<string>();

  // Optimized: cache rootNodes as Map for O(1) lookup instead of O(n) find
  const rootNodesMap = new Map<string, (typeof rootNodes)[0]>();
  for (const node of rootNodes) {
    const nodeIdStr = nodeIdMap.get(node.id) ?? String(node.id);
    rootNodesMap.set(nodeIdStr, node);
  }

  for (const node of sorted) {
    const nodeId = nodeIdMap.get(node.id) ?? String(node.id);
    if (placed.has(nodeId)) continue;

    result.push(node);
    placed.add(nodeId);

    // Add connected nodes immediately after
    const nodeConnections = connections.get(nodeId) || new Set();
    for (const connectedId of nodeConnections) {
      if (!placed.has(connectedId)) {
        const connectedNode = rootNodesMap.get(connectedId);
        if (connectedNode) {
          result.push(connectedNode);
          placed.add(connectedId);
        }
      }
    }
  }

  return result;
}

export function layoutL0(
  rootNodes: { id: C4Id; size: { width: number; height: number } }[],
  relationships: { from: C4Id; to: C4Id }[],
  _options: C4LayoutOptions,
  getHierarchyNode: (id: C4Id) => HierarchyNode | undefined,
  getSizedNode: (id: C4Id) => SizedNode | undefined
): { nodes: PositionedNode[]; width: number; height: number } {
  const result: PositionedNode[] = [];

  // Apply relationship-aware clustering before grid placement
  const clusteredNodes = clusterByRelationships(rootNodes, relationships);

  // 1. Position Systems using Flow Layout (Flex-wrap style) with variable sizes
  let currentX = 0;
  let currentY = 0;
  let rowHeight = 0;
  const MAX_ROW_WIDTH = 1200; // Break to new row after this width
  const DEFAULT_SYSTEM_WIDTH = SYSTEM_WIDTH;
  const DEFAULT_SYSTEM_HEIGHT = SYSTEM_HEIGHT;

  // Optimized: use for loop instead of forEach for better performance
  for (const sys of clusteredNodes) {
    // Use provided size or default
    // Optimized: cache size reference
    const sysSize = sys.size;
    const width = sysSize.width > 0 ? sysSize.width : DEFAULT_SYSTEM_WIDTH;
    const height = sysSize.height > 0 ? sysSize.height : DEFAULT_SYSTEM_HEIGHT;

    // Wrap to next row?
    if (currentX + width > MAX_ROW_WIDTH && currentX > 0) {
      currentX = 0;
      currentY += rowHeight + SYSTEM_V_SPACING;
      rowHeight = 0;
    }

    const x = currentX;
    const y = currentY;
    const bbox: BBox = { x, y, width, height };

    // Update generic flow tracking
    currentX += width + SYSTEM_H_SPACING;
    rowHeight = Math.max(rowHeight, height);

    // We need to fetch original node data to create PositionedNode
    const sized = getSizedNode(sys.id);
    if (sized) {
      result.push({
        ...sized,
        size: { width, height },
        x,
        y,
        bbox,
      });
    }

    // 3. Container Badges
    const hierarchyNode = getHierarchyNode(sys.id);
    if (hierarchyNode && hierarchyNode.children.length > 0) {
      const containers = hierarchyNode.children.filter((c) => isContainerKind(c.node.kind));
      if (containers.length > 0) {
        // If system is large (expanded), we assume badges are already handled by caller
        // via layoutL2Containers logic. But layoutL0 is responsible for badges if not provided?
        // The caller (applyL0Layout) handles 'badges' logic.
        // But here we place badges at bottom-right if we are using standard badging.
        // If the system is expanded (large), typically we might not want standard badges
        // OR we want them placed specifically.

        // Heuristic: If size is standard, do standard badges.
        // If size is custom (expanded), skip automatic badge placement
        // (assume caller positioned them relative or they are real nodes).
        const isExpanded = width > DEFAULT_SYSTEM_WIDTH || height > DEFAULT_SYSTEM_HEIGHT;

        if (!isExpanded) {
          const badgeGridCols = Math.min(6, Math.ceil(Math.sqrt(containers.length)));

          const badgesStartX = x + BOUNDARY_PADDING;
          const rowsNeeded = Math.ceil(containers.length / badgeGridCols);
          const startY =
            y +
            height -
            BOUNDARY_PADDING -
            rowsNeeded * BADGE_SIZE -
            (rowsNeeded - 1) * BADGE_V_SPACING;

          containers.forEach((container, cIndex) => {
            const cCol = cIndex % badgeGridCols;
            const cRow = Math.floor(cIndex / badgeGridCols);

            const bx = badgesStartX + cCol * (BADGE_SIZE + BADGE_H_SPACING);
            const by = startY + cRow * (BADGE_SIZE + BADGE_V_SPACING);

            const containerSized = getSizedNode(container.id);
            if (containerSized) {
              result.push({
                ...containerSized,
                size: { width: BADGE_SIZE, height: BADGE_SIZE },
                x: bx,
                y: by,
                bbox: { x: bx, y: by, width: BADGE_SIZE, height: BADGE_SIZE },
              });
            }
          });
        }
      }
    }
  }

  // Calculate total bounding box dimensions
  const totalWidth = MAX_ROW_WIDTH; // approx
  const totalHeight = currentY + rowHeight;

  return { nodes: result, width: totalWidth, height: totalHeight };
}
