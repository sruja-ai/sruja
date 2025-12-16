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
  maxIterations: number = 15 // Increased default from 12
): EdgeForReduction[] {
  let improved = true;
  let iterations = 0;
  let lastTotalCrossings = Infinity;
  let noImprovementCount = 0;

  while (improved && iterations < maxIterations) {
    improved = false;
    iterations++;

    // Calculate crossing counts for each edge (O(n²) but necessary)
    // Optimized: compare each pair exactly once (i < j) to avoid double counting
    const crossingCounts = new Map<string, number>();
    let totalCrossings = 0;

    // Initialize all edge counts to 0
    for (const edge of edges) {
      crossingCounts.set(edge.id, 0);
    }

    // Compare each pair of edges exactly once (i < j)
    // Cache edge references to avoid repeated array access
    for (let i = 0; i < edges.length; i++) {
      const edge1 = edges[i];
      for (let j = i + 1; j < edges.length; j++) {
        const edge2 = edges[j];
        if (
          pathsCrossFn(
            edge1.path,
            edge2.path,
            edge1.sourceId,
            edge1.targetId,
            edge2.sourceId,
            edge2.targetId
          )
        ) {
          // Increment count for both edges (each crossing involves two edges)
          crossingCounts.set(edge1.id, (crossingCounts.get(edge1.id) || 0) + 1);
          crossingCounts.set(edge2.id, (crossingCounts.get(edge2.id) || 0) + 1);
          totalCrossings += 2; // Each crossing counts for both edges
        }
      }
    }

    // Early exit: if no crossings, we're done
    if (totalCrossings === 0) {
      break;
    }

    // Early exit: if no improvement for 2 consecutive iterations, stop
    if (totalCrossings >= lastTotalCrossings) {
      noImprovementCount++;
      if (noImprovementCount >= 2) {
        break; // No improvement for 2 iterations, stop
      }
    } else {
      noImprovementCount = 0; // Reset counter on improvement
    }
    lastTotalCrossings = totalCrossings;

    // Sort edges by crossing count (most crossings first)
    const sortedEdges = [...edges].sort((a, b) => {
      const aCrossings = crossingCounts.get(a.id) || 0;
      const bCrossings = crossingCounts.get(b.id) || 0;
      return bCrossings - aCrossings;
    });

    // Try to re-route edges with crossings
    // Limit to top N problematic edges per iteration for efficiency
    const maxEdgesToProcess = Math.min(
      sortedEdges.length,
      Math.max(10, Math.floor(edges.length * 0.3))
    );
    for (let idx = 0; idx < maxEdgesToProcess; idx++) {
      const edge = sortedEdges[idx];
      const currentCrossings = crossingCounts.get(edge.id) || 0;
      if (currentCrossings === 0) break; // No more edges with crossings, stop early

      // Get current path's crossings with other edges
      // Optimized: cache edge reference and use early exit
      const countCrossings = (path: Point[]): number => {
        let crossings = 0;
        const edgeId = edge.id;
        const sourceId = edge.sourceId;
        const targetId = edge.targetId;

        for (const otherEdge of edges) {
          if (otherEdge.id === edgeId) continue;
          if (
            pathsCrossFn(
              path,
              otherEdge.path,
              sourceId,
              targetId,
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
      const allSides: ReadonlyArray<"north" | "south" | "east" | "west"> = [
        "north",
        "south",
        "east",
        "west",
      ] as const;
      let bestPath = edge.path;
      let bestCrossings = currentPathCrossings;

      // For high-crossing edges, be aggressive but limit options for performance
      // Lower threshold to catch more edges, especially for expanded/hierarchical diagrams
      const isHighCrossing = currentPathCrossings > 2; // Lower threshold: catch edges with 3+ crossings
      const isVeryHighCrossing = currentPathCrossings > 5; // Very high: 6+ crossings
      // Reduced padding options for performance - focus on most effective values
      const paddingOptions = isVeryHighCrossing
        ? [15, 25, 40, 60, 80, 120, 180, 250] // Reduced from 18 to 8 options
        : isHighCrossing
          ? [15, 25, 40, 60, 80, 120, 180] // Reduced from 16 to 7 options
          : currentPathCrossings > 1
            ? [20, 30, 50, 70, 100] // Reduced from 12 to 5 options
            : [30, 50, 70]; // Reduced from 7 to 3 options

      // Try all combinations, prioritizing opposite sides for high-crossing edges
      // Optimized: use Set for O(1) lookup instead of array.some() which is O(n)
      const combinationSet = new Set<string>();
      const combinations: Array<{
        srcSide: (typeof allSides)[number];
        tgtSide: (typeof allSides)[number];
      }> = [];

      const getComboKey = (src: string, tgt: string) => `${src}:${tgt}`;
      const addCombo = (src: (typeof allSides)[number], tgt: (typeof allSides)[number]) => {
        const key = getComboKey(src, tgt);
        if (!combinationSet.has(key)) {
          combinationSet.add(key);
          combinations.push({ srcSide: src, tgtSide: tgt });
        }
      };

      if (isHighCrossing || isVeryHighCrossing) {
        // For high-crossing edges, prioritize opposite sides first
        addCombo("north", "south");
        addCombo("south", "north");
        addCombo("east", "west");
        addCombo("west", "east");

        // For very high crossings, also try perpendicular combinations
        if (isVeryHighCrossing) {
          addCombo("north", "east");
          addCombo("north", "west");
          addCombo("south", "east");
          addCombo("south", "west");
          addCombo("east", "north");
          addCombo("east", "south");
          addCombo("west", "north");
          addCombo("west", "south");
        }
      }

      // Then add all other combinations
      for (const srcSide of allSides) {
        for (const tgtSide of allSides) {
          addCombo(srcSide, tgtSide);
        }
      }

      // Cache obstacles reference to avoid repeated property access
      const obstacles = edge.obstacles;

      for (const combo of combinations) {
        const ports = edge.getPorts(combo.srcSide, combo.tgtSide);
        const sourcePort = ports.sourcePort;
        const targetPort = ports.targetPort;

        for (const padding of paddingOptions) {
          const newPath = routeOrthogonalAvoid(sourcePort, targetPort, obstacles, padding);
          const newCrossings = countCrossings(newPath);

          if (newCrossings < bestCrossings) {
            bestPath = newPath;
            bestCrossings = newCrossings;
            // For high-crossing edges, accept any improvement immediately
            if (
              bestCrossings === 0 ||
              (isHighCrossing && bestCrossings < currentPathCrossings - 1)
            ) {
              break;
            }
          }
        }
        if (bestCrossings === 0 || (isHighCrossing && bestCrossings < currentPathCrossings - 1)) {
          break;
        }
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
