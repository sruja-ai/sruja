// packages/layout/src/algorithms/crossing-minimizer.ts
// Global edge crossing minimization algorithm
import type { Point } from "../geometry/point";
import type { Rect } from "../geometry/rect";
import { routeOrthogonalAvoid } from "./edge-router";
// Import pathsCross from c4-layout (will be passed as parameter to avoid circular dependency)

export interface EdgeRoute {
  id: string;
  sourceId: string;
  targetId: string;
  sourcePort: { side: "north" | "south" | "east" | "west"; position: Point; angle: number };
  targetPort: { side: "north" | "south" | "east" | "west"; position: Point; angle: number };
  path: Point[];
  obstacles: Rect[];
}

export interface NodeInfo {
  id: string;
  bbox: Rect;
  parentId?: string;
}

/**
 * Global crossing minimization using iterative improvement
 * Tries different edge orderings and port combinations to minimize total crossings
 */
export function minimizeCrossingsGlobally(
  edges: Array<{
    id: string;
    sourceId: string;
    targetId: string;
    source: NodeInfo;
    target: NodeInfo;
    obstacles: Rect[];
    getPorts: (
      sourceSide?: "north" | "south" | "east" | "west",
      targetSide?: "north" | "south" | "east" | "west"
    ) => {
      sourcePort: { side: "north" | "south" | "east" | "west"; position: Point; angle: number };
      targetPort: { side: "north" | "south" | "east" | "west"; position: Point; angle: number };
    };
  }>,
  pathsCrossFn: (
    path1: Point[],
    path2: Point[],
    from1?: string,
    to1?: string,
    from2?: string,
    to2?: string
  ) => boolean,
  options: {
    maxIterations?: number;
    tryAllPorts?: boolean;
    prioritizeShortEdges?: boolean;
  } = {}
): EdgeRoute[] {
  const edgeCount = edges.length;
  // Increase iterations for iterative repair as each iteration is faster
  const {
    maxIterations = edgeCount > 20 ? 30 : 20,
    tryAllPorts: _tryAllPorts = true,
    prioritizeShortEdges = true,
  } = options;

  // Initial sort order
  const sortedEdges = [...edges].sort((a, b) => {
    if (!prioritizeShortEdges) return 0;
    // Parent-child priority
    const aIsParentChild = a.source.parentId === b.target.id || a.target.parentId === b.source.id;
    const bIsParentChild = b.source.parentId === a.target.id || b.target.parentId === a.source.id;
    if (aIsParentChild && !bIsParentChild) return -1;
    if (!aIsParentChild && bIsParentChild) return 1;

    // Length priority
    const aDx = a.target.bbox.x - a.source.bbox.x;
    const aDy = a.target.bbox.y - a.source.bbox.y;
    const bDx = b.target.bbox.x - b.source.bbox.x;
    const bDy = b.target.bbox.y - b.source.bbox.y;
    return aDx * aDx + aDy * aDy - (bDx * bDx + bDy * bDy);
  });

  // Map to store current best routes
  const currentRoutes = new Map<string, EdgeRoute>();

  // Helper to route a single edge against current routes
  const routeEdge = (edge: (typeof edges)[0], otherRoutes: EdgeRoute[]): EdgeRoute => {
    let bestPath: Point[] | null = null;
    let bestSourcePort = edge.getPorts().sourcePort; // Default
    let bestTargetPort = edge.getPorts().targetPort; // Default
    let minCrossings = Infinity;

    // Cache obstacle access
    const obstacles = edge.obstacles;

    // 1. Try default ports first
    const defaultPorts = edge.getPorts();
    const defaultPath = routeOrthogonalAvoid(
      defaultPorts.sourcePort,
      defaultPorts.targetPort,
      obstacles,
      20
    );

    // Count crossings for default path
    let defaultCrossings = 0;
    for (const other of otherRoutes) {
      if (
        pathsCrossFn(
          defaultPath,
          other.path,
          edge.sourceId,
          edge.targetId,
          other.sourceId,
          other.targetId
        )
      ) {
        defaultCrossings++;
      }
    }

    if (defaultCrossings === 0) {
      return {
        id: edge.id,
        sourceId: edge.sourceId,
        targetId: edge.targetId,
        sourcePort: defaultPorts.sourcePort,
        targetPort: defaultPorts.targetPort,
        path: defaultPath,
        obstacles: edge.obstacles,
      };
    }

    minCrossings = defaultCrossings;
    bestPath = defaultPath;
    bestSourcePort = defaultPorts.sourcePort;
    bestTargetPort = defaultPorts.targetPort;

    // 2. Try alternatives if needed
    const combinations = [];
    const allSides = ["north", "south", "east", "west"] as const;

    // Heuristic: prioritize opposite sides
    for (const srcSide of allSides) {
      for (const tgtSide of allSides) {
        // Skip default if we already tried it (not strictly necessary but clean)
        combinations.push({ srcSide, tgtSide });
      }
    }

    // Sort combinations: opposite sides first
    combinations.sort((a, b) => {
      const aOpp =
        (a.srcSide === "north" && a.tgtSide === "south") ||
        (a.srcSide === "south" && a.tgtSide === "north") ||
        (a.srcSide === "east" && a.tgtSide === "west") ||
        (a.srcSide === "west" && a.tgtSide === "east");
      const bOpp =
        (b.srcSide === "north" && b.tgtSide === "south") ||
        (b.srcSide === "south" && b.tgtSide === "north") ||
        (b.srcSide === "east" && b.tgtSide === "west") ||
        (b.srcSide === "west" && b.tgtSide === "east");
      return (aOpp ? -1 : 1) - (bOpp ? -1 : 1);
    });

    const maxCombos = minCrossings > 2 ? 8 : 4; // Limit combinations
    const paddingOptions = minCrossings > 2 ? [15, 30, 60] : [20, 40];

    for (const combo of combinations.slice(0, maxCombos)) {
      const ports = edge.getPorts(combo.srcSide, combo.tgtSide);

      for (const padding of paddingOptions) {
        const path = routeOrthogonalAvoid(ports.sourcePort, ports.targetPort, obstacles, padding);

        let currentCrossings = 0;
        let possible = true;
        for (const other of otherRoutes) {
          if (
            pathsCrossFn(
              path,
              other.path,
              edge.sourceId,
              edge.targetId,
              other.sourceId,
              other.targetId
            )
          ) {
            currentCrossings++;
            if (currentCrossings >= minCrossings) {
              possible = false;
              break;
            }
          }
        }

        if (possible && currentCrossings < minCrossings) {
          minCrossings = currentCrossings;
          bestPath = path;
          bestSourcePort = ports.sourcePort;
          bestTargetPort = ports.targetPort;
          if (minCrossings === 0) break;
        }
      }
      if (minCrossings === 0) break;
    }

    return {
      id: edge.id,
      sourceId: edge.sourceId,
      targetId: edge.targetId,
      sourcePort: bestSourcePort,
      targetPort: bestTargetPort,
      path: bestPath!,
      obstacles: edge.obstacles,
    };
  };

  // Phase 1: Initial construction (greedy)
  for (const edge of sortedEdges) {
    const existingRoutes = Array.from(currentRoutes.values());
    const route = routeEdge(edge, existingRoutes);
    currentRoutes.set(edge.id, route);
  }

  // Phase 2: Iterative Repair
  let improvement = true;
  let iter = 0;

  while (improvement && iter < maxIterations) {
    improvement = false;
    iter++;

    // Calculate crossings for all edges
    const crossingCounts = new Map<string, number>();
    const allRoutes = Array.from(currentRoutes.values());
    let totalCrossings = 0;

    for (let i = 0; i < allRoutes.length; i++) {
      for (let j = i + 1; j < allRoutes.length; j++) {
        const r1 = allRoutes[i];
        const r2 = allRoutes[j];
        if (pathsCrossFn(r1.path, r2.path, r1.sourceId, r1.targetId, r2.sourceId, r2.targetId)) {
          crossingCounts.set(r1.id, (crossingCounts.get(r1.id) || 0) + 1);
          crossingCounts.set(r2.id, (crossingCounts.get(r2.id) || 0) + 1);
          totalCrossings++;
        }
      }
    }

    if (totalCrossings === 0) break;

    // Identify bad edges (those with crossings)
    const badEdges = sortedEdges
      .filter((e) => (crossingCounts.get(e.id) || 0) > 0)
      .sort((a, b) => (crossingCounts.get(b.id) || 0) - (crossingCounts.get(a.id) || 0));

    // Limit re-routing to worst offenders to avoid oscillation
    // Re-route top 30% of bad edges or at one least
    const edgesToReroute = badEdges.slice(0, Math.max(1, Math.ceil(badEdges.length * 0.3)));

    for (const edge of edgesToReroute) {
      // Remove current route
      const oldRoute = currentRoutes.get(edge.id)!;
      currentRoutes.delete(edge.id);

      // Re-route against all OTHER routes
      const others = Array.from(currentRoutes.values());
      const newRoute = routeEdge(edge, others);

      // Calculate new crossing count for this edge
      // Note: routeEdge already minimizes this, but we need to check if it's strictly better
      // than previous state? Not necessarily, routeEdge finds GLOBAL opt against current others.
      // So just accept it.

      // Optimization: Check if path actually changed
      // Simple length check or just accept
      if (
        newRoute.path.length !== oldRoute.path.length ||
        JSON.stringify(newRoute.path) !== JSON.stringify(oldRoute.path)
      ) {
        improvement = true;
      }

      currentRoutes.set(edge.id, newRoute);
    }
  }

  return Array.from(currentRoutes.values());
}

/**
 * Layer-based routing for hierarchical structures
 * Routes edges in layers to minimize crossings between different hierarchy levels
 * For complex graphs (>15 edges), uses fully global approach instead
 */
export function routeWithLayers(
  edges: Array<{
    id: string;
    sourceId: string;
    targetId: string;
    source: NodeInfo;
    target: NodeInfo;
    obstacles: Rect[];
    getPorts: (
      sourceSide?: "north" | "south" | "east" | "west",
      targetSide?: "north" | "south" | "east" | "west"
    ) => {
      sourcePort: { side: "north" | "south" | "east" | "west"; position: Point; angle: number };
      targetPort: { side: "north" | "south" | "east" | "west"; position: Point; angle: number };
    };
  }>,
  nodeDepths: Map<string, number>,
  pathsCrossFn: (
    path1: Point[],
    path2: Point[],
    from1?: string,
    to1?: string,
    from2?: string,
    to2?: string
  ) => boolean
): EdgeRoute[] {
  const totalEdges = edges.length;

  // For complex graphs with many edges, use fully global approach
  // Layer-based routing can miss cross-layer crossing opportunities
  if (totalEdges > 15) {
    // Sort edges: prioritize short edges and parent-child relationships
    const sortedEdges = [...edges].sort((a, b) => {
      // Parent-child edges first (within hierarchy)
      const aIsParentChild = a.source.parentId === b.target.id || a.target.parentId === b.source.id;
      const bIsParentChild = b.source.parentId === a.target.id || b.target.parentId === a.source.id;
      if (aIsParentChild && !bIsParentChild) return -1;
      if (!aIsParentChild && bIsParentChild) return 1;

      // Then by distance (shorter first)
      const aDist = Math.hypot(
        a.target.bbox.x - a.source.bbox.x,
        a.target.bbox.y - a.source.bbox.y
      );
      const bDist = Math.hypot(
        b.target.bbox.x - b.source.bbox.x,
        b.target.bbox.y - b.source.bbox.y
      );
      return aDist - bDist;
    });

    // Use global minimization with high iteration count
    return minimizeCrossingsGlobally(sortedEdges, pathsCrossFn, {
      maxIterations: totalEdges > 20 ? 20 : 15,
      tryAllPorts: true,
      prioritizeShortEdges: true,
    });
  }

  // For simpler graphs, use layer-based approach
  const layerGroups = new Map<number, typeof edges>();

  for (const edge of edges) {
    const sourceDepth = nodeDepths.get(edge.sourceId) || 0;
    const targetDepth = nodeDepths.get(edge.targetId) || 0;
    const layerDiff = Math.abs(sourceDepth - targetDepth);

    if (!layerGroups.has(layerDiff)) {
      layerGroups.set(layerDiff, []);
    }
    layerGroups.get(layerDiff)!.push(edge);
  }

  // Route edges layer by layer
  const sortedLayerDiffs = Array.from(layerGroups.keys()).sort((a, b) => a - b);
  const routes: EdgeRoute[] = [];

  for (const layerDiff of sortedLayerDiffs) {
    const layerEdges = layerGroups.get(layerDiff)!;

    const layerRoutes = minimizeCrossingsGlobally(layerEdges, pathsCrossFn, {
      maxIterations: layerDiff === 0 ? 12 : 10,
      tryAllPorts: true,
      prioritizeShortEdges: layerDiff === 0,
    });

    routes.push(...layerRoutes);
  }

  return routes;
}
