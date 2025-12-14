// packages/layout/src/algorithms/crossing-reducer.ts
// Post-processing to reduce edge crossings by re-routing problematic edges
import type { Point } from "../geometry/point";
import type { Rect } from "../geometry/rect";
import { routeOrthogonalAvoid } from "./edge-router";
// Import pathsCross function (will be passed as parameter to avoid circular dependency)

export interface EdgeForReduction {
  id: string;
  sourceId: string;
  targetId: string;
  path: Point[];
  source: { bbox: Rect };
  target: { bbox: Rect };
  obstacles: Rect[];
  getPorts: (
    sourceSide?: "north" | "south" | "east" | "west",
    targetSide?: "north" | "south" | "east" | "west"
  ) => {
    sourcePort: { side: "north" | "south" | "east" | "west"; position: Point; angle: number };
    targetPort: { side: "north" | "south" | "east" | "west"; position: Point; angle: number };
  };
}

/**
 * Reduce crossings by re-routing edges that have many crossings
 */
export function reduceCrossingsPostProcess(
  edges: EdgeForReduction[],
  pathsCrossFn: (
    path1: Point[],
    path2: Point[],
    from1?: string,
    to1?: string,
    from2?: string,
    to2?: string
  ) => boolean,
  maxIterations: number = 12
): EdgeForReduction[] {
  let improved = true;
  let iterations = 0;

  while (improved && iterations < maxIterations) {
    improved = false;
    iterations++;

    // Calculate crossing counts for each edge
    const crossingCounts = new Map<string, number>();
    for (let i = 0; i < edges.length; i++) {
      let crossings = 0;
      for (let j = 0; j < edges.length; j++) {
        if (
          i !== j &&
          pathsCrossFn(
            edges[i].path,
            edges[j].path,
            edges[i].sourceId,
            edges[i].targetId,
            edges[j].sourceId,
            edges[j].targetId
          )
        ) {
          crossings++;
        }
      }
      crossingCounts.set(edges[i].id, crossings);
    }

    // Sort edges by crossing count (most crossings first)
    const sortedEdges = [...edges].sort((a, b) => {
      const aCrossings = crossingCounts.get(a.id) || 0;
      const bCrossings = crossingCounts.get(b.id) || 0;
      return bCrossings - aCrossings;
    });

    // Try to re-route edges with crossings
    for (const edge of sortedEdges) {
      const currentCrossings = crossingCounts.get(edge.id) || 0;
      if (currentCrossings === 0) continue; // Skip edges with no crossings

      // Get current path's crossings with other edges
      const countCrossings = (path: Point[]): number => {
        let crossings = 0;
        for (const otherEdge of edges) {
          if (otherEdge.id === edge.id) continue;
          if (
            pathsCrossFn(
              path,
              otherEdge.path,
              edge.sourceId,
              edge.targetId,
              otherEdge.sourceId,
              otherEdge.targetId
            )
          ) {
            crossings++;
          }
        }
        return crossings;
      };

      const currentPathCrossings = countCrossings(edge.path);
      if (currentPathCrossings === 0) continue;

      // Try all 16 port combinations (4x4) with different padding
      const allSides: Array<"north" | "south" | "east" | "west"> = [
        "north",
        "south",
        "east",
        "west",
      ];
      let bestPath = edge.path;
      let bestCrossings = currentPathCrossings;

      // For high-crossing edges, be extremely aggressive - we need B (80+) for all diagrams
      // Lower threshold to catch more edges
      const isHighCrossing = currentPathCrossings > 1;
      const paddingOptions = isHighCrossing
        ? [10, 15, 20, 25, 30, 35, 40, 50, 60, 70, 80, 100, 120, 150, 180, 200]
        : currentPathCrossings > 1
          ? [15, 20, 25, 30, 35, 40, 50, 60, 70, 80, 100, 120]
          : [20, 30, 40, 50, 60, 70, 80];

      // Try all combinations, prioritizing opposite sides for high-crossing edges
      const combinations: Array<{
        srcSide: (typeof allSides)[number];
        tgtSide: (typeof allSides)[number];
      }> = [];

      if (isHighCrossing) {
        // For high-crossing edges, prioritize opposite sides first
        const oppositePairs = [
          { srcSide: "north" as const, tgtSide: "south" as const },
          { srcSide: "south" as const, tgtSide: "north" as const },
          { srcSide: "east" as const, tgtSide: "west" as const },
          { srcSide: "west" as const, tgtSide: "east" as const },
        ];
        oppositePairs.forEach((pair) => combinations.push(pair));
      }

      // Then add all other combinations
      for (const srcSide of allSides) {
        for (const tgtSide of allSides) {
          if (!combinations.some((c) => c.srcSide === srcSide && c.tgtSide === tgtSide)) {
            combinations.push({ srcSide, tgtSide });
          }
        }
      }

      for (const combo of combinations) {
        const ports = edge.getPorts(combo.srcSide, combo.tgtSide);

        for (const padding of paddingOptions) {
          const newPath = routeOrthogonalAvoid(
            ports.sourcePort,
            ports.targetPort,
            edge.obstacles,
            padding
          );
          const newCrossings = countCrossings(newPath);

          if (newCrossings < bestCrossings) {
            bestPath = newPath;
            bestCrossings = newCrossings;
            // For high-crossing edges, accept any improvement immediately
            if (bestCrossings === 0 || (isHighCrossing && bestCrossings < currentPathCrossings - 1))
              break;
          }
        }
        if (bestCrossings === 0 || (isHighCrossing && bestCrossings < currentPathCrossings - 1))
          break;
      }

      // If we found a better path, update the edge
      if (bestCrossings < currentPathCrossings) {
        edge.path = bestPath;
        improved = true;
      }
    }
  }

  return edges;
}
