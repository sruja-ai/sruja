import type { Node as RFNode } from "@xyflow/react";
import type { GraphvizResult, C4Node } from "./types";
import { SCALE } from "./layoutEngine";

/**
 * Parse Graphviz bounding box string "llx,lly,urx,ury" to position and size
 * Graphviz coordinates are in points, with origin at bottom-left
 * React Flow uses top-left origin, so we need to convert
 *
 * The bounding box coordinates are already in points (same as node positions),
 * so we apply the same SCALE factor and coordinate conversion as nodes.
 */
function parseBoundingBox(
  bb: string | undefined,
  canvasHeight: number
): { x: number; y: number; width: number; height: number } | null {
  if (!bb) return null;

  const parts = bb.split(",").map(Number);
  if (parts.length !== 4) return null;

  const [llx, lly, urx, ury] = parts;

  // Graphviz: llx, lly (lower-left), urx, ury (upper-right)
  // All coordinates are in points
  // Width and height in points
  const width = (urx - llx) * SCALE;
  const height = (ury - lly) * SCALE;

  // Convert to React Flow coordinates (top-left origin)
  // Graphviz uses bottom-left origin
  // x: left edge, apply SCALE
  // y: top edge = canvasHeight - ury, then apply SCALE
  const x = llx * SCALE;
  const y = (canvasHeight - ury) * SCALE;

  return { x, y, width, height };
}

/**
 * Build compound node structure from Graphviz clusters
 *
 * This function:
 * 1. Creates parent container nodes (group nodes) from Graphviz clusters
 * 2. Sets child nodes' parentId to establish parent-child relationships
 * 3. Adjusts positions to account for parent containers
 *
 * React Flow automatically handles positioning of children relative to parents
 * when parentId is set, but we need to ensure parent nodes are created first.
 */
export function buildCompoundNodeStructure(
  layoutResult: GraphvizResult,
  c4Nodes: C4Node[],
  manualPositionsMap: Record<string, { x: number; y: number } | { X: number; Y: number }>
): RFNode[] {
  const nodes: RFNode[] = [];
  const clusters = layoutResult.clusters || {};

  // Create a map of C4 nodes by ID for quick lookup
  const c4NodeMap = new Map(c4Nodes.map((n) => [n.id, n]));

  // Create a map of layout nodes by ID
  const layoutNodeMap = new Map(layoutResult.nodes.map((n) => [n.id, n]));

  // Track which nodes are children (to avoid creating them as standalone nodes)
  const childNodeIds = new Set<string>();
  Object.values(clusters).forEach((cluster) => {
    cluster.children.forEach((childId) => childNodeIds.add(childId));
  });

  // First, create parent container nodes (group nodes) from clusters
  // Sort clusters by depth (parents before children) to handle nested structures
  // A container that is a child of a system should be positioned relative to the system
  const clusterEntries = Array.from(Object.entries(clusters));

  // Sort so that if cluster A contains cluster B's parent, A comes before B
  clusterEntries.sort(([idA], [idB]) => {
    // Check if A is a child of B (B should come first)
    const clusterB = clusters[idB];
    if (clusterB && clusterB.children.includes(idA)) {
      return 1; // A comes after B
    }
    // Check if B is a child of A (A should come first)
    const clusterA = clusters[idA];
    if (clusterA && clusterA.children.includes(idB)) {
      return -1; // A comes before B
    }
    return 0; // No relationship, keep original order
  });

  clusterEntries.forEach(([parentId, cluster]) => {
    const c4Node = c4NodeMap.get(parentId);
    if (!c4Node) {
      // Parent node not found in C4 nodes - skip this cluster
      console.warn(`[buildCompoundNodeStructure] Parent node ${parentId} not found in C4 nodes`);
      return;
    }

    // Parse bounding box to get position and size
    const bb = parseBoundingBox(cluster.bb, layoutResult.height);
    if (!bb) {
      console.warn(
        `[buildCompoundNodeStructure] Invalid bounding box for parent ${parentId}: ${cluster.bb}`
      );
      return;
    }

    // Check if this parent is also a child of another parent (nested structure)
    // For example, a container (L3) that is inside a system (L2)
    let grandParentId: string | undefined;
    let grandParentNode: RFNode | undefined;
    Object.entries(clusters).forEach(([gpid, gcluster]) => {
      if (gcluster.children.includes(parentId) && gpid !== parentId) {
        grandParentId = gpid;
        grandParentNode = nodes.find((n) => n.id === gpid);
      }
    });

    // Debug logging in dev mode
    if (typeof import.meta !== "undefined" && import.meta.env?.DEV) {
      console.log(`[buildCompoundNodeStructure] Creating parent container:`, {
        parentId,
        bb,
        childCount: cluster.children.length,
        children: cluster.children,
        grandParentId,
        isNested: !!grandParentId,
      });
    }

    // Use manual position if available, otherwise use calculated from bounding box
    const savedPosition = manualPositionsMap[parentId];
    let absolutePosition = { x: bb.x, y: bb.y };
    if (savedPosition) {
      const pos = savedPosition as { x?: number; y?: number; X?: number; Y?: number };
      absolutePosition = {
        x: pos.x ?? pos.X ?? bb.x,
        y: pos.y ?? pos.Y ?? bb.y,
      };
    }

    // Create parent container node (group node)
    // Use the bounding box dimensions, but ensure minimum size
    // Add padding to ensure children are fully visible
    const padding = 40; // Padding around children
    const minWidth = 300;
    const minHeight = 200;
    const width = Math.max(bb.width + padding * 2, minWidth);
    const height = Math.max(bb.height + padding * 2, minHeight);

    // Calculate position: if this parent is nested inside another parent,
    // convert absolute position to relative position
    let position = absolutePosition;
    if (grandParentId && grandParentNode) {
      // This parent is nested - position relative to grandparent
      const grandParentPadding = 40; // Match padding used in grandparent
      position = {
        x: absolutePosition.x - grandParentNode.position.x + grandParentPadding,
        y: absolutePosition.y - grandParentNode.position.y + grandParentPadding,
      };
    } else {
      // Top-level parent - adjust position to account for padding (move left and up)
      position = {
        x: absolutePosition.x - padding,
        y: absolutePosition.y - padding,
      };
    }

    nodes.push({
      id: parentId,
      type: "group",
      position,
      parentId: grandParentId, // Set parentId if this parent is nested
      data: {
        ...c4Node,
        // Mark as parent container
        _isParent: true,
        _childCount: cluster.children.length,
      },
      width,
      height,
      style: {
        zIndex: grandParentId ? 2 : 1, // Nested parents should be above their parents
      },
    });
  });

  // Then, create child nodes and set their parentId
  c4Nodes.forEach((c4Node) => {
    // Skip if this node is a parent (already created above)
    if (clusters[c4Node.id]) {
      return;
    }

    // Find layout position for this node
    const layoutNode = layoutNodeMap.get(c4Node.id);
    if (!layoutNode) {
      console.warn(`[buildCompoundNodeStructure] Layout node not found for ${c4Node.id}`);
      return;
    }

    // Determine parent ID (if this node is a child)
    let parentId: string | undefined;
    let parentNode: RFNode | undefined;
    Object.entries(clusters).forEach(([pid, cluster]) => {
      if (cluster.children.includes(c4Node.id)) {
        parentId = pid;
        // Find the parent node we just created
        parentNode = nodes.find((n) => n.id === pid);
      }
    });

    // Use manual position if available, otherwise use layout position
    const savedPosition = manualPositionsMap[c4Node.id];
    let absolutePosition = { x: layoutNode.x, y: layoutNode.y };
    if (savedPosition) {
      const pos = savedPosition as { x?: number; y?: number; X?: number; Y?: number };
      absolutePosition = {
        x: pos.x ?? pos.X ?? layoutNode.x,
        y: pos.y ?? pos.Y ?? layoutNode.y,
      };
    }

    // If this node has a parent, convert absolute position to relative position
    // React Flow expects child positions to be relative to parent's top-left corner
    // Account for parent container padding
    const parentPadding = 40; // Should match padding in parent creation
    let position = absolutePosition;
    if (parentId && parentNode) {
      position = {
        x: absolutePosition.x - parentNode.position.x + parentPadding,
        y: absolutePosition.y - parentNode.position.y + parentPadding,
      };
    }

    // Create child node
    nodes.push({
      id: c4Node.id,
      type: "sruja",
      position,
      parentId, // Set parent ID to establish parent-child relationship
      data: {
        ...c4Node,
        _isChild: parentId !== undefined,
      },
      width: c4Node.width,
      height: c4Node.height,
      style: {
        zIndex: parentId ? 2 : 1, // Children should be above parents
      },
    });
  });

  // Also create nodes that are neither parents nor children (standalone nodes)
  // These are nodes that exist in the layout but aren't part of any cluster
  layoutResult.nodes.forEach((layoutNode) => {
    // Skip if already created (as parent or child)
    if (nodes.find((n) => n.id === layoutNode.id)) {
      return;
    }

    const c4Node = c4NodeMap.get(layoutNode.id);
    if (!c4Node) {
      return;
    }

    const savedPosition = manualPositionsMap[layoutNode.id];
    let position = { x: layoutNode.x, y: layoutNode.y };
    if (savedPosition) {
      const pos = savedPosition as { x?: number; y?: number; X?: number; Y?: number };
      position = {
        x: pos.x ?? pos.X ?? layoutNode.x,
        y: pos.y ?? pos.Y ?? layoutNode.y,
      };
    }

    nodes.push({
      id: layoutNode.id,
      type: "sruja",
      position,
      data: c4Node,
      width: layoutNode.width,
      height: layoutNode.height,
      style: {
        zIndex: 1,
      },
    });
  });

  return nodes;
}
