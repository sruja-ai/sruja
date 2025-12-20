/**
 * Edge Routing Helpers
 * 
 * Extracted utilities for edge routing in the layout engine.
 * Reduces duplication in c4-layout.ts.
 */
import type { Point } from "./geometry/point";
import type { Rect as BBox } from "./geometry/rect";

/** Node representation for edge routing */
export interface EdgeRoutingNode {
    id: string;
    bbox: BBox;
    parent?: { id: string };
}

/**
 * Get all ancestor IDs by walking up the parent chain.
 * Works with runtime positioned nodes that have parent.id structure.
 * 
 * @param startId - Starting parent ID
 * @param nodeGetter - Function to get a node by ID
 * @returns Set of ancestor node IDs
 */
export function getEdgeRoutingAncestors(
    startId: string | undefined,
    nodeGetter: (id: string) => EdgeRoutingNode | undefined
): Set<string> {
    const ancestors = new Set<string>();
    let curr = startId;
    while (curr) {
        ancestors.add(curr);
        const node = nodeGetter(curr);
        const parent = node?.parent;
        curr = parent?.id;
    }
    return ancestors;
}

/**
 * Get the opposite side for bidirectional edge routing.
 */
export function getOppositeSide(side: "north" | "south" | "east" | "west"): "north" | "south" | "east" | "west" {
    switch (side) {
        case "north": return "south";
        case "south": return "north";
        case "east": return "west";
        case "west": return "east";
    }
}

/**
 * Calculate obstacles for edge routing, excluding source, target, and their ancestors.
 * 
 * @param nodes - All positioned nodes
 * @param sourceId - Source node ID
 * @param targetId - Target node ID
 * @param sourceParentId - Source's parent ID (if any)
 * @param targetParentId - Target's parent ID (if any)
 * @param nodeGetter - Function to get a node by ID
 * @returns Array of bounding boxes to avoid
 */
export function calculateEdgeObstacles(
    nodes: Iterable<EdgeRoutingNode>,
    sourceId: string,
    targetId: string,
    sourceParentId: string | undefined,
    targetParentId: string | undefined,
    nodeGetter: (id: string) => EdgeRoutingNode | undefined
): BBox[] {
    const srcAncestors = getEdgeRoutingAncestors(sourceParentId, nodeGetter);
    const dstAncestors = getEdgeRoutingAncestors(targetParentId, nodeGetter);

    const obstacles: BBox[] = [];
    for (const n of nodes) {
        if (
            n.id !== sourceId &&
            n.id !== targetId &&
            !srcAncestors.has(n.id) &&
            !dstAncestors.has(n.id)
        ) {
            obstacles.push(n.bbox);
        }
    }
    return obstacles;
}

/** Edge preparation data for routing */
export interface EdgeForRouting {
    id: string;
    sourceId: string;
    targetId: string;
    source: { id: string; bbox: BBox; parentId?: string };
    target: { id: string; bbox: BBox; parentId?: string };
    obstacles: BBox[];
    getPorts: (
        sourceSide?: "north" | "south" | "east" | "west",
        targetSide?: "north" | "south" | "east" | "west"
    ) => {
        sourcePort: { side: "north" | "south" | "east" | "west"; position: Point; angle: number };
        targetPort: { side: "north" | "south" | "east" | "west"; position: Point; angle: number };
    };
}

/**
 * Check if a node depth map indicates hierarchical structure.
 */
export function hasHierarchy(nodeDepths: Map<string, number>): boolean {
    for (const depth of nodeDepths.values()) {
        if (depth > 0) return true;
    }
    return false;
}

/**
 * Build node depth map from positioned nodes.
 */
export function buildDepthMap(
    nodes: Iterable<[string, EdgeRoutingNode]>,
    nodeGetter: (id: string) => EdgeRoutingNode | undefined
): Map<string, number> {
    const depths = new Map<string, number>();
    for (const [id, node] of nodes) {
        let depth = 0;
        let current = node.parent;
        while (current) {
            depth++;
            current = nodeGetter(current.id)?.parent;
        }
        depths.set(id, depth);
    }
    return depths;
}
