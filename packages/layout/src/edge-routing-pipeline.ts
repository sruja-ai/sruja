/**
 * Edge Routing Pipeline
 * 
 * Consolidates edge routing logic extracted from c4-layout.ts.
 * Handles edge preparation, crossing detection, and optimization.
 */
import type { Point } from "./geometry/point";
import type { Rect as BBox } from "./geometry/rect";
import {
    calculateBestPortWithObstacles,
} from "./algorithms/edge-router";
import { routeWithLayers } from "./algorithms/crossing-minimizer";
import { reduceCrossingsPostProcess } from "./algorithms/crossing-reducer";
import { pathsCross, calculatePortForSide } from "./layout-helpers";
import {
    getEdgeRoutingAncestors,
    getOppositeSide,
    type EdgeRoutingNode,
} from "./edge-routing-helpers";

/** Port information for edge routing */
export interface PortInfo {
    side: "north" | "south" | "east" | "west";
    position: Point;
    angle: number;
}

/** Edge prepared for routing */
export interface PreparedEdge {
    id: string;
    sourceId: string;
    targetId: string;
    source: { id: string; bbox: BBox; parentId?: string };
    target: { id: string; bbox: BBox; parentId?: string };
    obstacles: BBox[];
    getPorts: (
        sourceSide?: "north" | "south" | "east" | "west",
        targetSide?: "north" | "south" | "east" | "west"
    ) => { sourcePort: PortInfo; targetPort: PortInfo };
}

/** Routed edge with path */
export interface RoutedEdge {
    id: string;
    sourceId: string;
    targetId: string;
    path: Point[];
}

/** Relationship input for edge preparation */
export interface RelationshipInput {
    id: string;
    from: string;
    to: string;
}

/**
 * Build node depth map from positioned nodes.
 */
export function buildNodeDepthMap(
    positioned: Map<string, EdgeRoutingNode>,
): Map<string, number> {
    const depths = new Map<string, number>();
    for (const [id, node] of positioned) {
        let depth = 0;
        let current = node.parent;
        while (current) {
            depth++;
            current = positioned.get(current.id)?.parent;
        }
        depths.set(id, depth);
    }
    return depths;
}

/**
 * Prepare edges for routing by calculating obstacles and port functions.
 * 
 * @param relationships - Input relationships
 * @param positioned - Positioned nodes
 * @param portUsage - Port usage tracking map
 * @returns Prepared edges ready for routing
 */
export function prepareEdgesForRouting(
    relationships: readonly RelationshipInput[],
    positioned: Map<string, EdgeRoutingNode>,
    portUsage: Map<string, number>
): PreparedEdge[] {
    const prepared: PreparedEdge[] = [];

    for (const rel of relationships) {
        const src = positioned.get(rel.from);
        const dst = positioned.get(rel.to);
        if (!src || !dst) continue;

        // Calculate obstacles: exclude source, target, and their ancestors
        const srcAncestors = getEdgeRoutingAncestors(
            src.parent?.id,
            (id) => positioned.get(id)
        );
        const dstAncestors = getEdgeRoutingAncestors(
            dst.parent?.id,
            (id) => positioned.get(id)
        );

        const obstacles: BBox[] = [];
        for (const n of positioned.values()) {
            if (
                n.id !== src.id &&
                n.id !== dst.id &&
                !srcAncestors.has(n.id) &&
                !dstAncestors.has(n.id)
            ) {
                obstacles.push(n.bbox);
            }
        }

        prepared.push({
            id: rel.id,
            sourceId: rel.from,
            targetId: rel.to,
            source: { id: src.id, bbox: src.bbox, parentId: src.parent?.id },
            target: { id: dst.id, bbox: dst.bbox, parentId: dst.parent?.id },
            obstacles,
            getPorts: (
                sourceSide?: "north" | "south" | "east" | "west",
                targetSide?: "north" | "south" | "east" | "west"
            ) => {
                const srcPort = sourceSide
                    ? calculatePortForSide(src.bbox, dst.bbox, sourceSide)
                    : calculateBestPortWithObstacles(src.bbox, dst.bbox, obstacles, portUsage);
                const tgtPort = targetSide
                    ? calculatePortForSide(dst.bbox, src.bbox, targetSide)
                    : calculateBestPortWithObstacles(dst.bbox, src.bbox, obstacles, portUsage);
                return { sourcePort: srcPort, targetPort: tgtPort };
            },
        });
    }

    return prepared;
}

/**
 * Count crossings for each edge in a set of routed edges.
 * 
 * @param routes - Routed edges with paths
 * @returns Map of edge ID to crossing count
 */
export function countEdgeCrossings(
    routes: readonly RoutedEdge[]
): Map<string, number> {
    const crossingCounts = new Map<string, number>();

    for (let i = 0; i < routes.length; i++) {
        let crossings = 0;
        for (let j = 0; j < routes.length; j++) {
            if (i !== j && pathsCross(
                routes[i].path,
                routes[j].path,
                routes[i].sourceId,
                routes[i].targetId,
                routes[j].sourceId,
                routes[j].targetId
            )) {
                crossings++;
            }
        }
        crossingCounts.set(routes[i].id, crossings);
    }

    return crossingCounts;
}

/**
 * Filter edges that have high crossing counts.
 * 
 * @param routes - Routed edges
 * @param crossingCounts - Crossing counts per edge
 * @param threshold - Minimum crossings to include (default: 3)
 * @param maxPercentage - Maximum percentage of edges to include (default: 0.5)
 * @returns High-crossing edges or empty array if too many
 */
export function getHighCrossingEdges(
    routes: readonly RoutedEdge[],
    crossingCounts: Map<string, number>,
    threshold: number = 3,
    maxPercentage: number = 0.5
): RoutedEdge[] {
    const highCrossing = routes.filter(
        (r) => (crossingCounts.get(r.id) || 0) >= threshold
    );

    // Avoid returning too many to prevent infinite loops
    if (highCrossing.length >= routes.length * maxPercentage) {
        return [];
    }

    return highCrossing;
}

/**
 * Run the full edge routing pipeline.
 * 
 * @param edges - Prepared edges for routing
 * @param nodeDepths - Node depth map for layer-based routing
 * @param hasExpandedNodes - Whether the graph has expanded nodes
 * @returns Optimized routed edges
 */
export function runEdgeRoutingPipeline(
    edges: PreparedEdge[],
    nodeDepths: Map<string, number>,
    hasExpandedNodes: boolean
): RoutedEdge[] {
    if (edges.length === 0) return [];

    // Phase 1: Initial layer-based routing
    let routes = routeWithLayers(edges, nodeDepths, pathsCross);

    // Phase 2: First crossing reduction pass
    const edgesForReduction = routes.map((route) => {
        const edge = edges.find((e) => e.id === route.id)!;
        return {
            id: route.id,
            sourceId: route.sourceId,
            targetId: route.targetId,
            path: route.path,
            source: edge.source,
            target: edge.target,
            obstacles: edge.obstacles,
            getPorts: edge.getPorts,
        };
    });

    const edgeCount = edgesForReduction.length;
    const baseIterations = hasExpandedNodes
        ? Math.min(20, Math.max(12, Math.floor(edgeCount / 3)))
        : Math.min(15, Math.max(10, Math.floor(edgeCount / 4)));

    const reducedEdges = reduceCrossingsPostProcess(edgesForReduction, pathsCross, baseIterations);
    routes = reducedEdges.map((reduced) => {
        const original = routes.find((r) => r.id === reduced.id)!;
        return { ...original, path: reduced.path };
    });

    // Phase 3: Final pass for high-crossing edges
    const crossingCounts = countEdgeCrossings(routes);
    const highCrossingEdges = getHighCrossingEdges(routes, crossingCounts, 3);

    if (highCrossingEdges.length > 0) {
        const finalEdges = highCrossingEdges.map((route) => {
            const edge = edges.find((e) => e.id === route.id)!;
            return {
                id: route.id,
                sourceId: route.sourceId,
                targetId: route.targetId,
                path: route.path,
                source: edge.source as { bbox: BBox },
                target: edge.target as { bbox: BBox },
                obstacles: edge.obstacles,
                getPorts: edge.getPorts,
            };
        });

        const finalIterations = hasExpandedNodes ? 12 : 8;
        const finalReduced = reduceCrossingsPostProcess(finalEdges, pathsCross, finalIterations);

        for (const reduced of finalReduced) {
            const routeIndex = routes.findIndex((r) => r.id === reduced.id);
            if (routeIndex >= 0) {
                routes[routeIndex] = { ...routes[routeIndex], path: reduced.path };
            }
        }
    }

    return routes;
}

export { getOppositeSide };
