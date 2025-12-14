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
  // Scale iterations based on edge count - more edges need more optimization
  const edgeCount = edges.length;
  const baseIterations = edgeCount > 20 ? 15 : edgeCount > 10 ? 12 : 10;
  const {
    maxIterations = baseIterations,
    tryAllPorts = true,
    prioritizeShortEdges = true,
  } = options;

  // Sort edges by priority: short edges first (they're less likely to cross)
  const sortedEdges = [...edges].sort((a, b) => {
    if (!prioritizeShortEdges) return 0;

    const aDist = Math.hypot(a.target.bbox.x - a.source.bbox.x, a.target.bbox.y - a.source.bbox.y);
    const bDist = Math.hypot(b.target.bbox.x - b.source.bbox.x, b.target.bbox.y - b.source.bbox.y);
    return aDist - bDist;
  });

  // Initial routing with default ports
  let routes: EdgeRoute[] = [];
  let minTotalCrossings = Infinity;
  let bestRoutes: EdgeRoute[] = [];

  // Try multiple iterations with different strategies
  for (let iter = 0; iter < maxIterations; iter++) {
    routes = [];

    // For later iterations, re-sort edges based on current crossing counts
    // This helps prioritize edges that are causing the most problems
    let edgesToProcess = sortedEdges;
    if (iter > 0 && bestRoutes.length > 0) {
      // Calculate crossing counts for each edge in the best solution so far
      const edgeCrossingCounts = new Map<string, number>();
      for (let i = 0; i < bestRoutes.length; i++) {
        let crossings = 0;
        for (let j = 0; j < bestRoutes.length; j++) {
          if (
            i !== j &&
            pathsCrossFn(
              bestRoutes[i].path,
              bestRoutes[j].path,
              bestRoutes[i].sourceId,
              bestRoutes[i].targetId,
              bestRoutes[j].sourceId,
              bestRoutes[j].targetId
            )
          ) {
            crossings++;
          }
        }
        edgeCrossingCounts.set(bestRoutes[i].id, crossings);
      }

      // Re-sort: edges with most crossings first (so we can fix them)
      edgesToProcess = [...sortedEdges].sort((a, b) => {
        const aCrossings = edgeCrossingCounts.get(a.id) || 0;
        const bCrossings = edgeCrossingCounts.get(b.id) || 0;
        return bCrossings - aCrossings; // Most crossings first
      });
    }

    for (const edge of edgesToProcess) {
      // Get initial ports
      const ports = edge.getPorts();

      // Try to find best port combination for this edge
      let bestPath: Point[] | null = null;
      let bestSourcePort = ports.sourcePort;
      let bestTargetPort = ports.targetPort;
      let minCrossings = Infinity;

      // Count crossings with already-routed edges
      const countCrossings = (path: Point[]): number => {
        let crossings = 0;
        for (const existingRoute of routes) {
          if (
            pathsCrossFn(
              path,
              existingRoute.path,
              edge.sourceId,
              edge.targetId,
              existingRoute.sourceId,
              existingRoute.targetId
            )
          ) {
            crossings++;
          }
        }
        return crossings;
      };

      // Try default ports first
      const defaultPath = routeOrthogonalAvoid(bestSourcePort, bestTargetPort, edge.obstacles, 20);
      minCrossings = countCrossings(defaultPath);
      bestPath = defaultPath;

      // If we have crossings, always try alternatives (especially for expanded nodes)
      // Try all 16 port combinations (4x4) to find the best routing
      if (minCrossings > 0) {
        const allSides: Array<"north" | "south" | "east" | "west"> = [
          "north",
          "south",
          "east",
          "west",
        ];

        // For expanded nodes or when tryAllPorts is true, try ALL 16 combinations (4x4)
        // This gives us maximum flexibility to find the best routing
        let combinationsTried = 0;
        const maxCombinations = tryAllPorts ? 16 : minCrossings > 2 ? 12 : 8;

        // For later iterations or high crossing counts, be more aggressive
        const isHighCrossingEdge = minCrossings > 2 || iter > 0;
        // More aggressive padding options for better crossing reduction
        const paddingOptions = isHighCrossingEdge
          ? [10, 15, 20, 25, 30, 35, 40, 50, 60, 70, 80, 100, 120, 150]
          : [15, 20, 25, 30, 35, 40, 50, 60, 70, 80];

        // Try all combinations, prioritizing those that might reduce crossings
        const combinations: Array<{
          srcSide: (typeof allSides)[number];
          tgtSide: (typeof allSides)[number];
        }> = [];
        for (const srcSide of allSides) {
          for (const tgtSide of allSides) {
            if (srcSide === bestSourcePort.side && tgtSide === bestTargetPort.side) continue;
            combinations.push({ srcSide, tgtSide });
          }
        }

        // Sort combinations by potential: prefer opposite sides for better separation
        combinations.sort((a, b) => {
          const aIsOpposite =
            (a.srcSide === "north" && a.tgtSide === "south") ||
            (a.srcSide === "south" && a.tgtSide === "north") ||
            (a.srcSide === "east" && a.tgtSide === "west") ||
            (a.srcSide === "west" && a.tgtSide === "east");
          const bIsOpposite =
            (b.srcSide === "north" && b.tgtSide === "south") ||
            (b.srcSide === "south" && b.tgtSide === "north") ||
            (b.srcSide === "east" && b.tgtSide === "west") ||
            (b.srcSide === "west" && b.tgtSide === "east");
          if (aIsOpposite && !bIsOpposite) return -1;
          if (!aIsOpposite && bIsOpposite) return 1;
          return 0;
        });

        for (const combo of combinations) {
          if (combinationsTried >= maxCombinations) break;
          combinationsTried++;

          const altPorts = edge.getPorts(combo.srcSide, combo.tgtSide);

          // Try multiple padding options for each port combination
          for (const padding of paddingOptions) {
            const altPath = routeOrthogonalAvoid(
              altPorts.sourcePort,
              altPorts.targetPort,
              edge.obstacles,
              padding
            );
            const crossings = countCrossings(altPath);

            if (crossings < minCrossings) {
              minCrossings = crossings;
              bestPath = altPath;
              bestSourcePort = altPorts.sourcePort;
              bestTargetPort = altPorts.targetPort;
              // For high-crossing edges, be more aggressive - accept any improvement
              if (minCrossings === 0 || (isHighCrossingEdge && minCrossings < 2)) break;
            }
          }
          if (minCrossings === 0 || (isHighCrossingEdge && minCrossings < 2)) break;
        }
      }

      // Track port usage for distribution (not used in this function but kept for interface compatibility)

      routes.push({
        id: edge.id,
        sourceId: edge.sourceId,
        targetId: edge.targetId,
        sourcePort: bestSourcePort,
        targetPort: bestTargetPort,
        path: bestPath,
        obstacles: edge.obstacles,
      });
    }

    // Count total crossings
    let totalCrossings = 0;
    for (let i = 0; i < routes.length; i++) {
      for (let j = i + 1; j < routes.length; j++) {
        if (
          pathsCrossFn(
            routes[i].path,
            routes[j].path,
            routes[i].sourceId,
            routes[i].targetId,
            routes[j].sourceId,
            routes[j].targetId
          )
        ) {
          totalCrossings++;
        }
      }
    }

    if (totalCrossings < minTotalCrossings) {
      minTotalCrossings = totalCrossings;
      bestRoutes = routes.map((r) => ({ ...r }));
    }

    // If we found a perfect solution (0 crossings), stop early
    if (minTotalCrossings === 0) break;

    // If we've improved significantly (reduced crossings by 30%+), continue
    // Otherwise, try different strategies
    if (iter > 0) {
      const previousTotal = minTotalCrossings;
      const improvement = (previousTotal - totalCrossings) / previousTotal;
      if (improvement < 0.1 && iter < maxIterations - 1) {
        // Not improving much, try different edge ordering
        if (iter === 1) {
          // Second iteration: reverse order
          sortedEdges.reverse();
        } else if (iter === 2) {
          // Third iteration: prioritize by edge length (long edges first)
          sortedEdges.sort((a, b) => {
            const aDist = Math.hypot(
              a.target.bbox.x - a.source.bbox.x,
              a.target.bbox.y - a.source.bbox.y
            );
            const bDist = Math.hypot(
              b.target.bbox.x - b.source.bbox.x,
              b.target.bbox.y - b.source.bbox.y
            );
            return bDist - aDist; // Longer edges first
          });
        } else {
          // Later iterations: prioritize edges with most crossings
          const edgeCrossingCounts = new Map<string, number>();
          for (let i = 0; i < routes.length; i++) {
            let crossings = 0;
            for (let j = 0; j < routes.length; j++) {
              if (
                i !== j &&
                pathsCrossFn(
                  routes[i].path,
                  routes[j].path,
                  routes[i].sourceId,
                  routes[i].targetId,
                  routes[j].sourceId,
                  routes[j].targetId
                )
              ) {
                crossings++;
              }
            }
            edgeCrossingCounts.set(routes[i].id, crossings);
          }
          sortedEdges.sort((a, b) => {
            const aCrossings = edgeCrossingCounts.get(a.id) || 0;
            const bCrossings = edgeCrossingCounts.get(b.id) || 0;
            return bCrossings - aCrossings; // Most crossings first
          });
        }
      }
    }
  }

  return bestRoutes;
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
