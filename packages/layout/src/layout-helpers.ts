/**
 * Layout Helper Utilities
 * 
 * Extracted from c4-layout.ts to improve modularity and reduce duplication.
 * FAANG best practice: Single responsibility and DRY principles.
 */
import type { Point } from "./geometry/point";
import type { Rect as BBox } from "./geometry/rect";
import type { PositionedC4Node } from "./types";

/**
 * Get all ancestors of a node by walking up the parent chain.
 * Used for edge routing to exclude parent containers from obstacle detection.
 * 
 * @param nodeId - The starting node ID (parentId)
 * @param positioned - Map of all positioned nodes
 * @returns Set of ancestor node IDs (excluding the node itself)
 */
export function getAncestors(
    nodeId: string | undefined,
    positioned: ReadonlyMap<string, PositionedC4Node>
): Set<string> {
    const ancestors = new Set<string>();
    let curr = nodeId;
    while (curr) {
        ancestors.add(curr);
        const node = positioned.get(curr);
        const parentId = node?.parentId;
        if (parentId) {
            curr = parentId;
        } else {
            break;
        }
    }
    return ancestors;
}

/**
 * Check if two line segments cross each other.
 * Uses the counter-clockwise (CCW) algorithm for intersection detection.
 * 
 * @param p1 - Start of first segment
 * @param p2 - End of first segment
 * @param p3 - Start of second segment
 * @param p4 - End of second segment
 * @returns true if segments cross
 */
export function segmentsCross(p1: Point, p2: Point, p3: Point, p4: Point): boolean {
    const ccw = (A: Point, B: Point, C: Point) => {
        return (C.y - A.y) * (B.x - A.x) > (B.y - A.y) * (C.x - A.x);
    };
    return ccw(p1, p3, p4) !== ccw(p2, p3, p4) && ccw(p1, p2, p3) !== ccw(p1, p2, p4);
}

/**
 * Check if two edge paths cross each other.
 * Excludes cases where edges share endpoints (meeting at a node is not a crossing).
 * 
 * @param path1 - Points of first path
 * @param path2 - Points of second path
 * @param from1 - Source node ID of first edge (optional, for endpoint detection)
 * @param to1 - Target node ID of first edge (optional)
 * @param from2 - Source node ID of second edge (optional)
 * @param to2 - Target node ID of second edge (optional)
 * @returns true if paths cross
 */
export function pathsCross(
    path1: Point[],
    path2: Point[],
    from1?: string,
    to1?: string,
    from2?: string,
    to2?: string
): boolean {
    // If edges share an endpoint, they don't cross (they meet at a node)
    if (from1 && from2 && to1 && to2) {
        if ((from1 === from2 && to1 === to2) || (from1 === to2 && to1 === from2)) {
            return false;
        }
    }

    for (let i = 0; i < path1.length - 1; i++) {
        for (let j = 0; j < path2.length - 1; j++) {
            const p1Start = path1[i];
            const p1End = path1[i + 1];
            const p2Start = path2[j];
            const p2End = path2[j + 1];

            // Check if segments share endpoints (within small tolerance)
            const tolerance = 1;
            const shareStart =
                Math.abs(p1Start.x - p2Start.x) < tolerance && Math.abs(p1Start.y - p2Start.y) < tolerance;
            const shareEnd =
                Math.abs(p1End.x - p2End.x) < tolerance && Math.abs(p1End.y - p2End.y) < tolerance;
            const shareStartEnd =
                Math.abs(p1Start.x - p2End.x) < tolerance && Math.abs(p1Start.y - p2End.y) < tolerance;
            const shareEndStart =
                Math.abs(p1End.x - p2Start.x) < tolerance && Math.abs(p1End.y - p2Start.y) < tolerance;

            if (shareStart || shareEnd || shareStartEnd || shareEndStart) {
                continue; // Segments are connected, not crossing
            }

            if (segmentsCross(p1Start, p1End, p2Start, p2End)) {
                return true;
            }
        }
    }
    return false;
}

export type Side = "north" | "south" | "east" | "west";

export interface PortResult {
    side: Side;
    position: Point;
    angle: number;
}

/**
 * Calculate port position for a specific side of a bounding box.
 * Used for edge routing at specific node sides.
 * 
 * @param source - Source node bounding box
 * @param target - Target node bounding box (for alignment)
 * @param side - Which side of the source to place the port
 * @returns Port result with position and angle
 */
export function calculatePortForSide(
    source: BBox,
    target: BBox,
    side: Side
): PortResult {
    const tc = { x: target.x + target.width / 2, y: target.y + target.height / 2 };
    const srcRight = source.x + source.width;
    const srcBottom = source.y + source.height;

    let position: Point;
    let angle: number;

    switch (side) {
        case "east":
            position = { x: srcRight, y: Math.max(source.y, Math.min(srcBottom, tc.y)) };
            angle = 0;
            break;
        case "west":
            position = { x: source.x, y: Math.max(source.y, Math.min(srcBottom, tc.y)) };
            angle = 180;
            break;
        case "south":
            position = { x: Math.max(source.x, Math.min(srcRight, tc.x)), y: srcBottom };
            angle = 90;
            break;
        case "north":
        default:
            position = { x: Math.max(source.x, Math.min(srcRight, tc.x)), y: source.y };
            angle = 270;
            break;
    }

    return { side, position, angle };
}

/**
 * Get obstacles for edge routing, excluding source, target, and their ancestors.
 * 
 * @param positioned - All positioned nodes
 * @param sourceId - Source node ID
 * @param targetId - Target node ID
 * @returns Array of bounding boxes to avoid
 */
export function getObstaclesForEdge(
    positioned: ReadonlyMap<string, PositionedC4Node>,
    sourceId: string,
    targetId: string
): BBox[] {
    const src = positioned.get(sourceId);
    const dst = positioned.get(targetId);
    if (!src || !dst) return [];

    const srcAncestors = getAncestors(src.parentId, positioned);
    const dstAncestors = getAncestors(dst.parentId, positioned);

    return [...positioned.values()]
        .filter(
            (n) =>
                n.nodeId !== sourceId &&
                n.nodeId !== targetId &&
                !srcAncestors.has(n.nodeId) &&
                !dstAncestors.has(n.nodeId)
        )
        .map((n) => n.bbox);
}

/**
 * Get the opposite side.
 */
export function oppositeSide(side: Side): Side {
    switch (side) {
        case "north": return "south";
        case "south": return "north";
        case "east": return "west";
        case "west": return "east";
    }
}

/**
 * Build a map of node depths for hierarchical routing.
 * 
 * @param positioned - All positioned nodes
 * @returns Map of node ID to depth (0 = root level)
 */
export function buildNodeDepths(
    positioned: ReadonlyMap<string, PositionedC4Node>
): Map<string, number> {
    const depths = new Map<string, number>();
    for (const [id, node] of positioned) {
        // Use the depth property if available, otherwise count ancestors
        if (node.depth !== undefined) {
            depths.set(id, node.depth);
        } else {
            let depth = 0;
            let currentId = node.parentId;
            while (currentId) {
                depth++;
                const parent = positioned.get(currentId);
                currentId = parent?.parentId;
            }
            depths.set(id, depth);
        }
    }
    return depths;
}

/**
 * Detect if a graph is dense (many edges relative to nodes).
 * Dense graphs need enhanced routing strategies.
 * 
 * @param nodeCount - Number of nodes
 * @param edgeCount - Number of edges
 * @returns true if graph is considered dense
 */
export function isDenseGraph(nodeCount: number, edgeCount: number): boolean {
    return edgeCount > 10 || edgeCount / nodeCount > 1.5;
}

/**
 * Check if any nodes have a parent (hierarchical structure).
 */
export function hasHierarchicalNodes(
    positioned: ReadonlyMap<string, PositionedC4Node>
): boolean {
    return positioned.size > 0 && [...positioned.values()].some((n) => n.parentId);
}
