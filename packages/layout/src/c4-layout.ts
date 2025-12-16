import type { C4Graph } from "./c4-model";
import type { C4ViewState } from "./c4-view";
import { buildHierarchy } from "./algorithms/hierarchy";
import { calculateSizes } from "./algorithms/sizing";
import { assignCoordinates } from "./algorithms/coordinates";
import {
  calculateBestPortWithObstacles,
  routeOrthogonalAvoid,
  routeSpline,
  pathMidpoint,
  arrowAngle,
  pathLength,
} from "./algorithms/edge-router";
import { routeWithLayers } from "./algorithms/crossing-minimizer";
import { reduceCrossingsPostProcess } from "./algorithms/crossing-reducer";
import {
  analyzeNodeCrossings,
  adjustNodePositionsForCrossings,
  increaseSpacingForCrossingNodes,
  findCrossingClusters,
} from "./algorithms/crossing-based-positioning";
import { applyLocalizedSwaps } from "./algorithms/localized-swap-optimizer";

/**
 * Calculate port position for a specific side
 */
function calculatePortForSide(
  source: BBox,
  target: BBox,
  side: "north" | "south" | "east" | "west"
): { side: "north" | "south" | "east" | "west"; position: Point; angle: number } | null {
  const tc = { x: target.x + target.width / 2, y: target.y + target.height / 2 };
  const srcRight = source.x + source.width;
  const srcBottom = source.y + source.height;

  let position: Point;
  let angle: number;

  if (side === "east") {
    position = { x: srcRight, y: Math.max(source.y, Math.min(srcBottom, tc.y)) };
    angle = 0;
  } else if (side === "west") {
    position = { x: source.x, y: Math.max(source.y, Math.min(srcBottom, tc.y)) };
    angle = 180;
  } else if (side === "south") {
    position = { x: Math.max(source.x, Math.min(srcRight, tc.x)), y: srcBottom };
    angle = 90;
  } else {
    // north
    position = { x: Math.max(source.x, Math.min(srcRight, tc.x)), y: source.y };
    angle = 270;
  }

  return { side, position, angle };
}
import type { Point } from "./geometry/point";

// Helper function to check if two edge paths cross
// Excludes cases where edges share endpoints (meeting at a node is not a crossing)
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
      // Skip if segments share an endpoint (they're connected, not crossing)
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

function segmentsCross(p1: Point, p2: Point, p3: Point, p4: Point): boolean {
  const ccw = (A: Point, B: Point, C: Point) => {
    return (C.y - A.y) * (B.x - A.x) > (B.y - A.y) * (C.x - A.x);
  };
  return ccw(p1, p3, p4) !== ccw(p2, p3, p4) && ccw(p1, p2, p3) !== ccw(p1, p2, p4);
}
import { calculateMetrics } from "./metrics";
import type { C4LayoutOptions } from "./c4-options";
import { InteractivePreset } from "./c4-options";
import { beautify } from "./beautifier";
import { removeOverlaps } from "./algorithms/overlap";
import { applyPostProcessors } from "./plugin";
import type { Rect as BBox } from "./geometry/rect";
import { bundleEdges } from "./algorithms/edge-bundler";
import { placeLabels, type NodeForLabeling } from "./algorithms/label-placer";
import { expandToViewport } from "./algorithms/viewport-expander";
import { applyMultiPassOptimization } from "./algorithms/optimizer";

export interface PositionedC4Node {
  nodeId: string;
  bbox: BBox;
  contentBox: BBox;
  labelBox: BBox;
  parentId?: string;
  childrenIds: readonly string[];
  depth: number;
  level: string;
  collapsed: boolean;
  visible: boolean;
  zIndex: number;
  ports: readonly { side: "north" | "south" | "east" | "west"; position: Point; angle: number }[];
}

export interface PositionedC4Relationship {
  relationshipId: string;
  sourceId: string;
  targetId: string;
  points: readonly Point[];
  controlPoints?: readonly Point[];
  segmentTypes: readonly ("line" | "arc" | "orthogonal")[];
  labelPosition: Point;
  labelAngle: number;
  labelBounds: BBox;
  arrowEnd: Point;
  arrowAngle: number;
  length: number;
  bendCount: number;
  crossesBoundaries: boolean;
}

export interface C4LayoutResult {
  nodes: ReadonlyMap<string, PositionedC4Node>;
  relationships: readonly PositionedC4Relationship[];
  bbox: BBox;
  center: Point;
  metrics: {
    aspectRatio: number;
    coverage: number;
    edgeCrossings: number;
    edgeBends: number;
    totalEdgeLength: number;
    uniformity: number;
    balance: number;
    compactness: number;
  };
  debug?: {
    layoutTimeMs: number;
    phases: { name: string; durationMs: number; nodesProcessed: number }[];
    warnings: string[];
  };
}

export function layout(
  graph: C4Graph,
  view: C4ViewState,
  options: C4LayoutOptions = InteractivePreset
): C4LayoutResult {
  const start = Date.now();
  const phases: { name: string; durationMs: number; nodesProcessed: number }[] = [];
  const h0 = Date.now();
  const tree = buildHierarchy(graph, view);
  phases.push({
    name: "buildHierarchy",
    durationMs: Date.now() - h0,
    nodesProcessed: tree.nodeMap.size,
  });

  const s0 = Date.now();
  const sizes = calculateSizes(
    tree,
    graph.relationships.map((r) => ({ from: r.from, to: r.to })),
    options.measurer,
    options
  );
  phases.push({ name: "calculateSizes", durationMs: Date.now() - s0, nodesProcessed: sizes.size });

  const c0 = Date.now();
  const rels = graph.relationships.map((r) => ({ from: r.from, to: r.to }));
  let positioned = assignCoordinates(tree, sizes, rels, options);
  phases.push({
    name: "assignCoordinates",
    durationMs: Date.now() - c0,
    nodesProcessed: positioned.size,
  });

  // Phase 4: Multi-pass optimization (NEW!)
  // This replaces the old single-pass overlap removal with a comprehensive optimization system
  if (options.optimization?.enabled) {
    const opt0 = Date.now();
    positioned = applyMultiPassOptimization(
      positioned,
      [...graph.relationships],
      tree,
      options.optimization
    );
    phases.push({
      name: "optimizeLayout",
      durationMs: Date.now() - opt0,
      nodesProcessed: positioned.size,
    });
  } else if (options.overlapRemoval?.enabled) {
    // Fallback to legacy overlap removal for backwards compatibility
    const o0 = Date.now();
    positioned = removeOverlaps(
      positioned,
      options.overlapRemoval.padding,
      options.overlapRemoval.iterations
    );
    phases.push({
      name: "removeOverlaps",
      durationMs: Date.now() - o0,
      nodesProcessed: positioned.size,
    });
  }

  const e0 = Date.now();
  const edges: PositionedC4Relationship[] = [];

  // Track bidirectional edge pairs to use different port sides and avoid crossings
  const bidirectionalPairs = new Map<string, number>();
  graph.relationships.forEach((rel) => {
    const forwardKey = `${rel.from}::${rel.to}`;
    const reverseKey = `${rel.to}::${rel.from}`;
    if (bidirectionalPairs.has(reverseKey)) {
      bidirectionalPairs.set(reverseKey, (bidirectionalPairs.get(reverseKey) || 0) + 1);
    } else {
      bidirectionalPairs.set(forwardKey, (bidirectionalPairs.get(forwardKey) || 0) + 1);
    }
  });

  // Track which edges we've processed to assign opposite sides for bidirectional pairs
  const processedPairs = new Map<string, number>();

  // Track already-placed edge paths to avoid crossings in dense graphs
  const placedEdges: Array<{
    from: string;
    to: string;
    path: Point[];
    sourceId: string;
    targetId: string;
  }> = [];

  // Track port usage to distribute edges across all sides
  const portUsage = new Map<string, number>();

  // Detect dense graphs (many edges relative to nodes)
  const totalEdges = graph.relationships.length;
  const totalNodes = graph.nodes.size;
  const isDenseGraph = totalEdges > 10 || totalEdges / totalNodes > 1.5;

  // Check for expanded nodes (hierarchical structure with parent-child relationships)
  const hasExpandedNodes =
    graph.nodes.size > 0 && Array.from(graph.nodes.values()).some((n) => n.parentId);

  // Treat expanded hierarchical graphs similarly to dense graphs for routing
  const needsEnhancedRouting = isDenseGraph || hasExpandedNodes;

  // For dense graphs and expanded nodes, sort edges by priority to route less complex ones first
  // Priority: 1) Parent-child edges (hierarchical), 2) Short edges, 3) Long edges
  // This helps reduce crossings by establishing good paths early
  const nodeParentMap = new Map<string, { parentId?: string }>();
  for (const [id, node] of positioned) {
    nodeParentMap.set(id, { parentId: node.parent?.id });
  }

  // For expanded nodes with many edges, use global crossing minimization
  // This considers all edges together to find the best overall routing
  // Lower threshold to trigger for expanded nodes (they need it more)
  // Also use for dense graphs with many potential crossings
  // Use global minimizer for any expanded nodes or dense graphs
  // Lower threshold to ensure we use it for all expanded cases to achieve B (80+)
  if (hasExpandedNodes || (isDenseGraph && graph.relationships.length > 8)) {
    // Build node depth map for layer-based routing
    const nodeDepths = new Map<string, number>();
    for (const [id, node] of positioned) {
      let depth = 0;
      let current = node.parent;
      while (current) {
        depth++;
        current = positioned.get(current.id)?.parent;
      }
      nodeDepths.set(id, depth);
    }

    // Prepare edges for global minimization
    const edgesForMinimization = graph.relationships
      .map((rel) => {
        const src = positioned.get(rel.from as any);
        const dst = positioned.get(rel.to as any);
        if (!src || !dst) return null;

        const getAncestors = (id: string | undefined): Set<string> => {
          const ancestors = new Set<string>();
          let curr = id;
          while (curr) {
            ancestors.add(curr);
            const parent = positioned.get(curr as any)?.parent;
            curr = parent?.id;
          }
          return ancestors;
        };

        const srcAncestors = getAncestors(src.parent?.id);
        const dstAncestors = getAncestors(dst.parent?.id);

        const obstacles = [...positioned.values()]
          .filter(
            (n) =>
              n.id !== src.id &&
              n.id !== dst.id &&
              !srcAncestors.has(n.id) &&
              !dstAncestors.has(n.id)
          )
          .map((n) => n.bbox);

        return {
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
              ? calculatePortForSide(src.bbox, dst.bbox, sourceSide)!
              : calculateBestPortWithObstacles(src.bbox, dst.bbox, obstacles, portUsage);
            const tgtPort = targetSide
              ? calculatePortForSide(dst.bbox, src.bbox, targetSide)!
              : calculateBestPortWithObstacles(dst.bbox, src.bbox, obstacles, portUsage);
            return { sourcePort: srcPort, targetPort: tgtPort };
          },
        };
      })
      .filter((e): e is NonNullable<typeof e> => e !== null);

    // Use layer-based routing for hierarchical structures
    let optimizedRoutes = routeWithLayers(edgesForMinimization, nodeDepths, pathsCross);

    // Post-process to further reduce crossings
    const edgesForReduction = optimizedRoutes.map((route) => {
      const edge = edgesForMinimization.find((e) => e.id === route.id)!;
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

    // Adaptive iterations: start with fewer, algorithm will early-exit when no improvement
    // For expanded nodes, use more iterations but algorithm optimizes internally
    const edgeCount = edgesForReduction.length;
    // Reduced max iterations - algorithm has early exit for efficiency
    const baseIterations = hasExpandedNodes
      ? Math.min(20, Math.max(12, Math.floor(edgeCount / 3))) // 12-20 iterations for expanded (was 20-30)
      : Math.min(15, Math.max(10, Math.floor(edgeCount / 4))); // 10-15 for normal (was 15-20)
    const reducedEdges = reduceCrossingsPostProcess(edgesForReduction, pathsCross, baseIterations);
    optimizedRoutes = reducedEdges.map((reduced) => {
      const original = optimizedRoutes.find((r) => r.id === reduced.id)!;
      return {
        ...original,
        path: reduced.path,
      };
    });

    // Final pass: Try one more round of aggressive optimization for edges still with crossings
    // Count crossings for each edge
    const finalCrossingCounts = new Map<string, number>();
    for (let i = 0; i < optimizedRoutes.length; i++) {
      let crossings = 0;
      for (let j = 0; j < optimizedRoutes.length; j++) {
        if (
          i !== j &&
          pathsCross(
            optimizedRoutes[i].path,
            optimizedRoutes[j].path,
            optimizedRoutes[i].sourceId,
            optimizedRoutes[i].targetId,
            optimizedRoutes[j].sourceId,
            optimizedRoutes[j].targetId
          )
        ) {
          crossings++;
        }
      }
      finalCrossingCounts.set(optimizedRoutes[i].id, crossings);
    }

    // Re-route edges with 3+ crossings one more time with even more aggressive settings
    const highCrossingEdges = optimizedRoutes.filter(
      (r) => (finalCrossingCounts.get(r.id) || 0) >= 3
    );
    if (highCrossingEdges.length > 0 && highCrossingEdges.length < optimizedRoutes.length * 0.5) {
      // Only re-route if less than 50% of edges are high-crossing (to avoid infinite loops)
      const edgesForFinalReduction = highCrossingEdges.map((route) => {
        const edge = edgesForMinimization.find((e) => e.id === route.id)!;
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

      // Adaptive iterations for final reduction - algorithm has early exit
      const finalReductionIterations = hasExpandedNodes ? 12 : 8; // Reduced from 15/10, early exit handles rest
      const finalReduced = reduceCrossingsPostProcess(
        edgesForFinalReduction,
        pathsCross,
        finalReductionIterations
      );
      for (const reduced of finalReduced) {
        const routeIndex = optimizedRoutes.findIndex((r) => r.id === reduced.id);
        if (routeIndex >= 0) {
          optimizedRoutes[routeIndex] = {
            ...optimizedRoutes[routeIndex],
            path: reduced.path,
          };
        }
      }
    }

    // Phase 5: Node position adjustment based on crossing analysis
    // After initial routing, analyze crossings and adjust node positions, then re-route
    const posAdj0 = Date.now();
    const currentEdges: Array<{ id: string; sourceId: string; targetId: string; path: Point[] }> =
      optimizedRoutes.map((r) => ({
        id: r.id,
        sourceId: r.sourceId,
        targetId: r.targetId,
        path: r.path,
      }));

    // Analyze which nodes are involved in crossings
    const crossingCounts = analyzeNodeCrossings(positioned, currentEdges, pathsCross);
    // Count actual crossings (each crossing involves 4 nodes, so divide by 4)
    const totalCrossings = Array.from(crossingCounts.values()).reduce((a, b) => a + b, 0) / 4;

    // Only adjust if we have significant crossings (more than 20)
    // This ensures we only adjust when there are real problems
    if (totalCrossings > 20) {
      // Find crossing clusters - groups of nodes causing many crossings
      const clusters = findCrossingClusters(positioned, currentEdges, pathsCross);

      // Focus on the worst clusters (top 3)
      const worstClusters = clusters.slice(0, 3);

      if (worstClusters.length > 0) {
        // For each worst cluster, try to spread nodes apart
        for (const cluster of worstClusters) {
          if (cluster.nodes.length < 2) continue;

          // Calculate cluster center
          let centerX = 0;
          let centerY = 0;
          for (const nodeId of cluster.nodes) {
            const node = positioned.get(nodeId);
            if (node) {
              centerX += node.bbox.x + node.bbox.width / 2;
              centerY += node.bbox.y + node.bbox.height / 2;
            }
          }
          centerX /= cluster.nodes.length;
          centerY /= cluster.nodes.length;

          // Push nodes away from cluster center
          for (const nodeId of cluster.nodes) {
            const node = positioned.get(nodeId);
            if (!node) continue;

            // Skip if node has parent (hierarchical constraint)
            if (node.parent) continue;

            const nodeCenterX = node.bbox.x + node.bbox.width / 2;
            const nodeCenterY = node.bbox.y + node.bbox.height / 2;

            const dx = nodeCenterX - centerX;
            const dy = nodeCenterY - centerY;
            const dist = Math.sqrt(dx * dx + dy * dy) || 1;

            // Push away from center
            const pushDist = Math.min(40, cluster.crossingCount * 5);
            const pushX = (dx / dist) * pushDist;
            const pushY = (dy / dist) * pushDist;

            const newX = node.bbox.x + pushX;
            const newY = node.bbox.y + pushY;

            positioned.set(nodeId, {
              ...node,
              x: newX,
              y: newY,
              bbox: {
                ...node.bbox,
                x: newX,
                y: newY,
              },
            });
          }
        }
      }

      // Apply force-directed adjustments to push high-crossing nodes apart
      // For very high crossing counts, be more aggressive
      const adjustmentIterations = totalCrossings > 40 ? 4 : 3;
      const adjustmentStepSize = totalCrossings > 40 ? 20 : 15;

      positioned = adjustNodePositionsForCrossings(
        positioned,
        crossingCounts,
        [...graph.relationships], // Convert readonly to array
        adjustmentIterations,
        adjustmentStepSize
      );

      // Increase spacing between nodes with crossing edges
      // Only do this if we still have many crossings after force adjustment
      const crossingsAfterForce =
        Array.from(analyzeNodeCrossings(positioned, currentEdges, pathsCross).values()).reduce(
          (a, b) => a + b,
          0
        ) / 4;

      if (crossingsAfterForce > 25) {
        positioned = increaseSpacingForCrossingNodes(
          positioned,
          currentEdges,
          pathsCross,
          60 // min spacing - more conservative
        );
      }

      // Re-route edges with new node positions
      // Rebuild edgesForMinimization with updated positions
      const updatedEdgesForMinimization = graph.relationships
        .map((rel) => {
          const src = positioned.get(rel.from as any);
          const dst = positioned.get(rel.to as any);
          if (!src || !dst) return null;

          const getAncestors = (id: string | undefined): Set<string> => {
            const ancestors = new Set<string>();
            let curr = id;
            while (curr) {
              const parent = positioned.get(curr as any)?.parent;
              if (parent) {
                ancestors.add(parent.id);
                curr = parent.id;
              } else {
                break;
              }
            }
            return ancestors;
          };

          const srcAncestors = getAncestors(src.parent?.id);
          const dstAncestors = getAncestors(dst.parent?.id);

          const obstacles = [...positioned.values()]
            .filter(
              (n) =>
                n.id !== src.id &&
                n.id !== dst.id &&
                !srcAncestors.has(n.id) &&
                !dstAncestors.has(n.id)
            )
            .map((n) => n.bbox);

          return {
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
              // Use the existing portUsage map for consistency
              const srcPort = sourceSide
                ? calculatePortForSide(src.bbox, dst.bbox, sourceSide)!
                : calculateBestPortWithObstacles(src.bbox, dst.bbox, obstacles, portUsage);
              const tgtPort = targetSide
                ? calculatePortForSide(dst.bbox, src.bbox, targetSide)!
                : calculateBestPortWithObstacles(dst.bbox, src.bbox, obstacles, portUsage);
              return { sourcePort: srcPort, targetPort: tgtPort };
            },
          };
        })
        .filter((e): e is NonNullable<typeof e> => e !== null);

      // Update nodeDepths after position adjustments (in case hierarchy changed)
      const updatedNodeDepths = new Map<string, number>();
      for (const [id, node] of positioned) {
        let depth = 0;
        let current = node.parent;
        while (current) {
          depth++;
          current = positioned.get(current.id)?.parent;
        }
        updatedNodeDepths.set(id, depth);
      }

      // Re-route with updated positions
      optimizedRoutes = routeWithLayers(updatedEdgesForMinimization, updatedNodeDepths, pathsCross);

      // Post-process again with new routes - be aggressive since we've moved nodes
      const edgesForReduction2 = optimizedRoutes.map((route) => {
        const edge = updatedEdgesForMinimization.find((e) => e.id === route.id)!;
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

      // Count crossings after re-routing to see if we improved
      let crossingsAfterReroute = 0;
      for (let i = 0; i < optimizedRoutes.length; i++) {
        for (let j = i + 1; j < optimizedRoutes.length; j++) {
          if (
            pathsCross(
              optimizedRoutes[i].path,
              optimizedRoutes[j].path,
              optimizedRoutes[i].sourceId,
              optimizedRoutes[i].targetId,
              optimizedRoutes[j].sourceId,
              optimizedRoutes[j].targetId
            )
          ) {
            crossingsAfterReroute++;
          }
        }
      }

      // Always do additional post-processing after node position adjustment
      // The node positions changed, so we need aggressive re-optimization
      const reducedEdges2 = reduceCrossingsPostProcess(edgesForReduction2, pathsCross, 12);
      optimizedRoutes = reducedEdges2.map((reduced) => {
        const original = optimizedRoutes.find((r) => r.id === reduced.id)!;
        return {
          ...original,
          path: reduced.path,
        };
      });

      // One more final pass for edges still with crossings
      const finalCrossingCounts2 = new Map<string, number>();
      for (let i = 0; i < optimizedRoutes.length; i++) {
        let crossings = 0;
        for (let j = 0; j < optimizedRoutes.length; j++) {
          if (
            i !== j &&
            pathsCross(
              optimizedRoutes[i].path,
              optimizedRoutes[j].path,
              optimizedRoutes[i].sourceId,
              optimizedRoutes[i].targetId,
              optimizedRoutes[j].sourceId,
              optimizedRoutes[j].targetId
            )
          ) {
            crossings++;
          }
        }
        finalCrossingCounts2.set(optimizedRoutes[i].id, crossings);
      }

      const highCrossingEdges2 = optimizedRoutes.filter(
        (r) => (finalCrossingCounts2.get(r.id) || 0) >= 2
      );
      if (
        highCrossingEdges2.length > 0 &&
        highCrossingEdges2.length < optimizedRoutes.length * 0.6
      ) {
        const edgesForFinalReduction2 = highCrossingEdges2.map((route) => {
          const edge = updatedEdgesForMinimization.find((e) => e.id === route.id)!;
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

        // Adaptive iterations for final reduction in swap optimization
        const finalReductionIterations2 = hasExpandedNodes ? 12 : 8; // Reduced from 15/10, early exit handles rest
        const finalReduced2 = reduceCrossingsPostProcess(
          edgesForFinalReduction2,
          pathsCross,
          finalReductionIterations2
        );
        for (const reduced of finalReduced2) {
          const routeIndex = optimizedRoutes.findIndex((r) => r.id === reduced.id);
          if (routeIndex >= 0) {
            optimizedRoutes[routeIndex] = {
              ...optimizedRoutes[routeIndex],
              path: reduced.path,
            };
          }
        }
      }

      // Apply localized node swaps after position adjustment (Section 2.3 of LayoutBestPractice.md)
      // Try swapping adjacent nodes in the same rank to reduce crossings
      // Increased maxSwaps for better crossing reduction, especially for expanded states
      const maxSwapsForOptimization = Math.min(
        25,
        Math.max(15, Math.floor(graph.relationships.length / 2))
      );
      const swapResult = applyLocalizedSwaps(
        positioned,
        currentEdges,
        graph.relationships,
        pathsCross,
        maxSwapsForOptimization // Adaptive: more swaps for complex diagrams
      );

      if (swapResult.improved) {
        positioned = swapResult.nodes;

        // Re-route edges with swapped positions
        const swappedEdgesForMinimization = graph.relationships
          .map((rel) => {
            const src = positioned.get(rel.from as any);
            const dst = positioned.get(rel.to as any);
            if (!src || !dst) return null;

            const getAncestors = (id: string | undefined): Set<string> => {
              const ancestors = new Set<string>();
              let curr = id;
              while (curr) {
                const parent = positioned.get(curr as any)?.parent;
                if (parent) {
                  ancestors.add(parent.id);
                  curr = parent.id;
                } else {
                  break;
                }
              }
              return ancestors;
            };

            const srcAncestors = getAncestors(src.parent?.id);
            const dstAncestors = getAncestors(dst.parent?.id);

            const obstacles = [...positioned.values()]
              .filter(
                (n) =>
                  n.id !== src.id &&
                  n.id !== dst.id &&
                  !srcAncestors.has(n.id) &&
                  !dstAncestors.has(n.id)
              )
              .map((n) => n.bbox);

            return {
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
                  ? calculatePortForSide(src.bbox, dst.bbox, sourceSide)!
                  : calculateBestPortWithObstacles(src.bbox, dst.bbox, obstacles, portUsage);
                const tgtPort = targetSide
                  ? calculatePortForSide(dst.bbox, src.bbox, targetSide)!
                  : calculateBestPortWithObstacles(dst.bbox, src.bbox, obstacles, portUsage);
                return { sourcePort: srcPort, targetPort: tgtPort };
              },
            };
          })
          .filter((e): e is NonNullable<typeof e> => e !== null);

        // Re-route with swapped positions
        const updatedNodeDepths2 = new Map<string, number>();
        for (const [id, node] of positioned) {
          let depth = 0;
          let current = node.parent;
          while (current) {
            depth++;
            current = positioned.get(current.id)?.parent;
          }
          updatedNodeDepths2.set(id, depth);
        }

        optimizedRoutes = routeWithLayers(
          swappedEdgesForMinimization,
          updatedNodeDepths2,
          pathsCross
        );

        // Final post-processing after swaps
        const edgesForSwapReduction = optimizedRoutes.map((route) => {
          const edge = swappedEdgesForMinimization.find((e) => e.id === route.id)!;
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

        const swapReducedEdges = reduceCrossingsPostProcess(edgesForSwapReduction, pathsCross, 8);
        optimizedRoutes = swapReducedEdges.map((reduced) => {
          const original = optimizedRoutes.find((r) => r.id === reduced.id)!;
          return {
            ...original,
            path: reduced.path,
          };
        });
      }

      phases.push({
        name: "nodePositionAdjustment",
        durationMs: Date.now() - posAdj0,
        nodesProcessed: positioned.size,
      });
    }

    // Convert optimized routes to edge format
    for (const route of optimizedRoutes) {
      const rel = graph.relationships.find((r) => r.id === route.id);
      if (!rel) continue;

      const mid = pathMidpoint(route.path);
      const angle = arrowAngle(
        route.path[route.path.length - 2] ?? route.path[0],
        route.path[route.path.length - 1]
      );
      edges.push({
        relationshipId: rel.id,
        sourceId: rel.from as any,
        targetId: rel.to as any,
        points: route.path,
        segmentTypes: route.path.slice(1).map(() => "orthogonal"),
        labelPosition: mid,
        labelAngle: 0,
        labelBounds: { x: mid.x - 50, y: mid.y - 10, width: 100, height: 20 },
        arrowEnd: route.path[route.path.length - 1],
        arrowAngle: angle,
        length: pathLength(route.path),
        bendCount: route.path.length - 2,
        crossesBoundaries: false,
      });
    }

    // Skip the per-edge routing loop for expanded nodes
    // (continue to edge bundling phase)
  } else {
    // Use existing per-edge routing for smaller graphs
    const relationshipsToRoute = [...graph.relationships].sort((a, b) => {
      const aSrc = nodeParentMap.get(a.from);
      const aDst = nodeParentMap.get(a.to);
      const bSrc = nodeParentMap.get(b.from);
      const bDst = nodeParentMap.get(b.to);

      // Check if edges are parent-child relationships (hierarchical)
      const aIsHierarchical = aSrc?.parentId === a.to || aDst?.parentId === a.from;
      const bIsHierarchical = bSrc?.parentId === b.to || bDst?.parentId === b.from;

      // For expanded nodes, prioritize hierarchical edges (parent-child) first
      // These are typically shorter and less likely to cross
      if (hasExpandedNodes) {
        if (aIsHierarchical && !bIsHierarchical) return -1;
        if (!aIsHierarchical && bIsHierarchical) return 1;
      }

      // For same type, prefer shorter edges (estimate by node distance)
      // This helps establish good paths early and reduces crossings
      const aSrcNode = positioned.get(a.from);
      const aDstNode = positioned.get(a.to);
      const bSrcNode = positioned.get(b.from);
      const bDstNode = positioned.get(b.to);

      if (aSrcNode && aDstNode && bSrcNode && bDstNode) {
        const aDist = Math.hypot(
          aSrcNode.bbox.x - aDstNode.bbox.x,
          aSrcNode.bbox.y - aDstNode.bbox.y
        );
        const bDist = Math.hypot(
          bSrcNode.bbox.x - bDstNode.bbox.x,
          bSrcNode.bbox.y - bDstNode.bbox.y
        );
        return aDist - bDist; // Shorter edges first
      }

      return 0;
    });

    for (const rel of relationshipsToRoute) {
      const src = positioned.get(rel.from as any);
      const dst = positioned.get(rel.to as any);
      if (!src || !dst) continue;

      // Filter obstacles: Exclude source, target, and their ancestors (otherwise routing fails starting inside a parent)
      const getAncestors = (id: string | undefined): Set<string> => {
        const ancestors = new Set<string>();
        let curr = id;
        while (curr) {
          ancestors.add(curr);
          const parent = positioned.get(curr as any)?.parent;
          curr = parent?.id;
        }
        return ancestors;
      };

      const srcAncestors = getAncestors(src.parent?.id);
      const dstAncestors = getAncestors(dst.parent?.id);

      const obstacles = [...positioned.values()]
        .filter(
          (n) =>
            n.id !== src.id && n.id !== dst.id && !srcAncestors.has(n.id) && !dstAncestors.has(n.id)
        )
        .map((n) => n.bbox);

      // Check if this is part of a bidirectional pair
      const forwardKey = `${rel.from}::${rel.to}`;
      const reverseKey = `${rel.to}::${rel.from}`;
      const isBidirectional =
        bidirectionalPairs.get(reverseKey) !== undefined ||
        (bidirectionalPairs.get(forwardKey) || 0) > 1;

      // Get count of how many edges we've processed for this pair
      const pairKey = rel.from < rel.to ? forwardKey : reverseKey;
      const processedCount = processedPairs.get(pairKey) || 0;
      processedPairs.set(pairKey, processedCount + 1);

      // For bidirectional edges, use different port sides to avoid crossings
      let sp, tp;
      if (isBidirectional && processedCount === 1) {
        // Second edge in bidirectional pair: use opposite sides to avoid crossing
        const baseSp = calculateBestPortWithObstacles(src.bbox, dst.bbox, obstacles, portUsage);
        const baseTp = calculateBestPortWithObstacles(dst.bbox, src.bbox, obstacles, portUsage);

        // Use opposite sides for the second edge
        const oppositeSide = (side: string): "north" | "south" | "east" | "west" => {
          if (side === "north") return "south";
          if (side === "south") return "north";
          if (side === "east") return "west";
          return "east";
        };

        const oppositeSpSide = oppositeSide(baseSp.side);
        const oppositeTpSide = oppositeSide(baseTp.side);

        // Recalculate positions for opposite sides
        const srcRight = src.bbox.x + src.bbox.width;
        const srcBottom = src.bbox.y + src.bbox.height;
        const dstRight = dst.bbox.x + dst.bbox.width;
        const dstBottom = dst.bbox.y + dst.bbox.height;
        const srcCenter = {
          x: src.bbox.x + src.bbox.width / 2,
          y: src.bbox.y + src.bbox.height / 2,
        };
        const dstCenter = {
          x: dst.bbox.x + dst.bbox.width / 2,
          y: dst.bbox.y + dst.bbox.height / 2,
        };

        // Calculate source port position
        let spPos: Point;
        let spAngle: number;
        if (oppositeSpSide === "east") {
          spPos = { x: srcRight, y: srcCenter.y };
          spAngle = 0;
        } else if (oppositeSpSide === "west") {
          spPos = { x: src.bbox.x, y: srcCenter.y };
          spAngle = 180;
        } else if (oppositeSpSide === "south") {
          spPos = { x: srcCenter.x, y: srcBottom };
          spAngle = 90;
        } else {
          spPos = { x: srcCenter.x, y: src.bbox.y };
          spAngle = 270;
        }

        // Calculate target port position
        let tpPos: Point;
        let tpAngle: number;
        if (oppositeTpSide === "east") {
          tpPos = { x: dstRight, y: dstCenter.y };
          tpAngle = 0;
        } else if (oppositeTpSide === "west") {
          tpPos = { x: dst.bbox.x, y: dstCenter.y };
          tpAngle = 180;
        } else if (oppositeTpSide === "south") {
          tpPos = { x: dstCenter.x, y: dstBottom };
          tpAngle = 90;
        } else {
          tpPos = { x: dstCenter.x, y: dst.bbox.y };
          tpAngle = 270;
        }

        sp = { side: oppositeSpSide, position: spPos, angle: spAngle };
        tp = { side: oppositeTpSide, position: tpPos, angle: tpAngle };
      } else {
        // Use obstacle-aware port selection for better routing
        // Pass port usage to distribute edges across all sides
        // For expanded nodes, be more aggressive about using all 4 sides
        if (hasExpandedNodes) {
          // For expanded nodes, try to balance port usage across all sides
          // This helps reduce congestion and crossings
          sp = calculateBestPortWithObstacles(src.bbox, dst.bbox, obstacles, portUsage);
          tp = calculateBestPortWithObstacles(dst.bbox, src.bbox, obstacles, portUsage);

          // If this node already has many edges on the selected side, try to use a different side
          const srcPortKey = `${src.bbox.x},${src.bbox.y}:${sp.side}`;
          const srcUsage = portUsage.get(srcPortKey) || 0;
          const dstPortKey = `${dst.bbox.x},${dst.bbox.y}:${tp.side}`;
          const dstUsage = portUsage.get(dstPortKey) || 0;

          // If either side is heavily used (3+ edges), try alternative sides
          if (srcUsage >= 3 || dstUsage >= 3) {
            const allSides: Array<"north" | "south" | "east" | "west"> = [
              "north",
              "south",
              "east",
              "west",
            ];

            // Find less-used sides
            const srcSideUsage = allSides
              .map((side) => ({
                side,
                usage: portUsage.get(`${src.bbox.x},${src.bbox.y}:${side}`) || 0,
              }))
              .sort((a, b) => a.usage - b.usage);

            const dstSideUsage = allSides
              .map((side) => ({
                side,
                usage: portUsage.get(`${dst.bbox.x},${dst.bbox.y}:${side}`) || 0,
              }))
              .sort((a, b) => a.usage - b.usage);

            // Try the least-used side that still makes sense directionally
            if (srcUsage >= 3) {
              for (const candidate of srcSideUsage) {
                if (candidate.usage < srcUsage) {
                  const newSp = calculatePortForSide(src.bbox, dst.bbox, candidate.side);
                  if (newSp) {
                    sp = newSp;
                    break;
                  }
                }
              }
            }

            if (dstUsage >= 3) {
              for (const candidate of dstSideUsage) {
                if (candidate.usage < dstUsage) {
                  const newTp = calculatePortForSide(dst.bbox, src.bbox, candidate.side);
                  if (newTp) {
                    tp = newTp;
                    break;
                  }
                }
              }
            }
          }
        } else {
          sp = calculateBestPortWithObstacles(src.bbox, dst.bbox, obstacles, portUsage);
          tp = calculateBestPortWithObstacles(dst.bbox, src.bbox, obstacles, portUsage);
        }
      }
      const routePref = rel.preferredRoute ?? options.edgeRouting.algorithm;
      if (routePref === "curved" || routePref === "splines") {
        const { points: p, controlPoints } = routeSpline(sp, tp);
        const mid = pathMidpoint([p[0], p[1]]);
        const angle = arrowAngle(p[0], p[1]);
        edges.push({
          relationshipId: rel.id,
          sourceId: rel.from as any,
          targetId: rel.to as any,
          points: p,
          controlPoints,
          segmentTypes: ["arc"],
          labelPosition: mid,
          labelAngle: 0,
          labelBounds: { x: mid.x - 50, y: mid.y - 10, width: 100, height: 20 },
          arrowEnd: p[p.length - 1],
          arrowAngle: angle,
          length: pathLength(p),
          bendCount: 0,
          crossesBoundaries: false,
        });
      } else {
        // Obstacles already calculated above for port selection
        // Increased routing padding to spread edges and reduce congestion
        // For dense graphs and expanded nodes, use even more padding to minimize crossings
        let routingPadding = options.strategy === "l1-context" ? 40 : 15;
        if (needsEnhancedRouting) {
          // Use even more padding for expanded nodes to handle hierarchical structures
          // Expanded nodes need significant padding to route around parent boundaries
          routingPadding = hasExpandedNodes
            ? options.strategy === "l1-context"
              ? 100
              : 60 // Very high padding for expanded nodes
            : options.strategy === "l1-context"
              ? 60
              : 30;
        }

        // Try multiple routing attempts with different padding to minimize crossings
        let bestPath: Point[] | null = null;
        let minCrossings = Infinity;

        if (needsEnhancedRouting && placedEdges.length > 0) {
          // For dense graphs and expanded nodes, try multiple routing strategies with different paddings
          // Also try different port combinations to find the best path
          // For expanded nodes, prioritize paths with fewer crossings more aggressively
          const paddingMultipliers = hasExpandedNodes
            ? [1.0, 1.5, 2.0] // Higher multipliers for expanded nodes to route around parents
            : [1.0, 1.2, 1.5, 1.8];
          const paddingOptions = paddingMultipliers.map((m) => routingPadding * m);

          // Try current ports first, prioritizing paths with zero or minimal crossings
          for (const testPadding of paddingOptions) {
            const testPath = routeOrthogonalAvoid(sp, tp, obstacles, testPadding);
            // Count crossings with already-placed edges
            let crossings = 0;
            for (const placed of placedEdges) {
              if (
                pathsCross(testPath, placed.path, src.id, dst.id, placed.sourceId, placed.targetId)
              ) {
                crossings++;
              }
            }
            if (crossings < minCrossings) {
              minCrossings = crossings;
              bestPath = testPath;
              // For expanded nodes, aggressively prefer paths with zero crossings
              if (hasExpandedNodes && crossings === 0) break;
            }
          }

          // If still many crossings, try alternative port combinations from all 4 sides
          // For expanded nodes, be more aggressive in trying all combinations
          const minCrossingsThreshold = hasExpandedNodes ? 1 : 2;
          if (minCrossings >= minCrossingsThreshold) {
            // Try all 4 sides for both source and target to find the best combination
            // This gives us 16 possible combinations (4x4), but we'll prioritize the most promising ones
            const allSides: Array<"north" | "south" | "east" | "west"> = [
              "north",
              "south",
              "east",
              "west",
            ];

            // Generate port positions for all sides
            const srcRight = src.bbox.x + src.bbox.width;
            const srcBottom = src.bbox.y + src.bbox.height;
            const dstRight = dst.bbox.x + dst.bbox.width;
            const dstBottom = dst.bbox.y + dst.bbox.height;
            const sc = { x: src.bbox.x + src.bbox.width / 2, y: src.bbox.y + src.bbox.height / 2 };
            const tc = { x: dst.bbox.x + dst.bbox.width / 2, y: dst.bbox.y + dst.bbox.height / 2 };

            const createPort = (side: "north" | "south" | "east" | "west", isSource: boolean) => {
              const rect = isSource ? src.bbox : dst.bbox;
              const right = isSource ? srcRight : dstRight;
              const bottom = isSource ? srcBottom : dstBottom;
              const targetCenter = isSource ? tc : sc;

              let position: Point;
              let angle: number;
              if (side === "east") {
                position = { x: right, y: Math.max(rect.y, Math.min(bottom, targetCenter.y)) };
                angle = 0;
              } else if (side === "west") {
                position = { x: rect.x, y: Math.max(rect.y, Math.min(bottom, targetCenter.y)) };
                angle = 180;
              } else if (side === "south") {
                position = { x: Math.max(rect.x, Math.min(right, targetCenter.x)), y: bottom };
                angle = 90;
              } else {
                // north
                position = { x: Math.max(rect.x, Math.min(right, targetCenter.x)), y: rect.y };
                angle = 270;
              }
              return { side, position, angle };
            };

            // For expanded nodes with many crossings, try ALL 16 combinations (4x4) to find the best
            // For others, prioritize promising combinations
            const maxCombinations =
              hasExpandedNodes && minCrossings > 5 ? 16 : hasExpandedNodes ? 12 : 6;
            const combinations: Array<{
              sp: ReturnType<typeof createPort>;
              tp: ReturnType<typeof createPort>;
            }> = [];

            // Prioritize combinations: start with best ports, then try all sides
            const bestSp = calculateBestPortWithObstacles(src.bbox, dst.bbox, obstacles, portUsage);
            const bestTp = calculateBestPortWithObstacles(dst.bbox, src.bbox, obstacles, portUsage);

            // Add best combination first
            combinations.push({ sp: bestSp, tp: bestTp });

            // For expanded nodes with many crossings, try ALL 16 combinations
            if (hasExpandedNodes && minCrossings > 5) {
              // Try all 4x4 = 16 combinations
              for (const srcSide of allSides) {
                for (const dstSide of allSides) {
                  if (combinations.length < maxCombinations) {
                    combinations.push({
                      sp: createPort(srcSide, true),
                      tp: createPort(dstSide, false),
                    });
                  }
                }
              }
            } else {
              // Then try combinations where one side is varied
              for (const altSide of allSides) {
                if (altSide !== bestSp.side && combinations.length < maxCombinations) {
                  combinations.push({ sp: createPort(altSide, true), tp: bestTp });
                }
                if (altSide !== bestTp.side && combinations.length < maxCombinations) {
                  combinations.push({ sp: bestSp, tp: createPort(altSide, false) });
                }
              }

              // For expanded nodes, also try some opposite side combinations
              if (hasExpandedNodes) {
                const oppositeSide = (side: "north" | "south" | "east" | "west") => {
                  if (side === "north") return "south";
                  if (side === "south") return "north";
                  if (side === "east") return "west";
                  return "east";
                };
                if (combinations.length < maxCombinations) {
                  combinations.push({
                    sp: createPort(oppositeSide(bestSp.side), true),
                    tp: createPort(oppositeSide(bestTp.side), false),
                  });
                }
              }
            }

            // Try each combination, prioritizing those with fewer crossings
            // Sort combinations by how promising they look (prefer less-used ports)
            // Use precise port position in key to differentiate ports on same side
            const sortedCombinations = combinations
              .map((combo) => {
                const srcPortKey = `${src.bbox.x},${src.bbox.y}:${combo.sp.side}:${Math.round(combo.sp.position.x)},${Math.round(combo.sp.position.y)}`;
                const dstPortKey = `${dst.bbox.x},${dst.bbox.y}:${combo.tp.side}:${Math.round(combo.tp.position.x)},${Math.round(combo.tp.position.y)}`;
                const srcUsage = portUsage.get(srcPortKey) || 0;
                const dstUsage = portUsage.get(dstPortKey) || 0;
                return { combo, priority: srcUsage + dstUsage }; // Lower usage = higher priority
              })
              .sort((a, b) => a.priority - b.priority)
              .map((item) => item.combo);

            for (const combo of sortedCombinations) {
              if (combo.sp.side === sp.side && combo.tp.side === tp.side) continue; // Skip if same as current

              const altPaddingOptions = hasExpandedNodes
                ? [routingPadding, routingPadding * 1.2, routingPadding * 1.5, routingPadding * 1.8]
                : [routingPadding, routingPadding * 1.2];

              for (const testPadding of altPaddingOptions) {
                const testPath = routeOrthogonalAvoid(combo.sp, combo.tp, obstacles, testPadding);
                let crossings = 0;
                for (const placed of placedEdges) {
                  if (pathsCross(testPath, placed.path, rel.from, rel.to, placed.from, placed.to)) {
                    crossings++;
                  }
                }
                // Use if it reduces crossings (even by 1 for expanded nodes)
                const improvementThreshold = hasExpandedNodes ? 0 : 1;
                if (crossings < minCrossings - improvementThreshold) {
                  minCrossings = crossings;
                  bestPath = testPath;
                  sp = combo.sp;
                  tp = combo.tp;
                  // For expanded nodes, be more aggressive - accept any improvement
                  if (minCrossings === 0 || (hasExpandedNodes && minCrossings < 3)) break; // Found good path
                }
              }
              // For expanded nodes, stop early if we found a good path
              if (minCrossings === 0 || (hasExpandedNodes && minCrossings < 3)) break;
            }
          }
        }

        const pts = bestPath || routeOrthogonalAvoid(sp, tp, obstacles, routingPadding);

        // Store this edge path for future crossing checks
        placedEdges.push({
          from: rel.from,
          to: rel.to,
          path: pts,
          sourceId: src.id,
          targetId: dst.id,
        });

        const mid = pathMidpoint(pts);
        const angle = arrowAngle(pts[pts.length - 2] ?? pts[0], pts[pts.length - 1]);
        edges.push({
          relationshipId: rel.id,
          sourceId: rel.from as any,
          targetId: rel.to as any,
          points: pts,
          segmentTypes: pts.slice(1).map(() => "orthogonal"),
          labelPosition: mid,
          labelAngle: 0,
          labelBounds: { x: mid.x - 50, y: mid.y - 10, width: 100, height: 20 },
          arrowEnd: pts[pts.length - 1],
          arrowAngle: angle,
          length: pathLength(pts),
          bendCount: pts.length - 2,
          crossesBoundaries: false,
        });
      }
    }
  }
  phases.push({ name: "routeEdges", durationMs: Date.now() - e0, nodesProcessed: edges.length });

  // Edge bundling: group parallel edges and fan out at endpoints
  // Use more aggressive bundling to reduce congestion
  const bundle0 = Date.now();
  const edgesForBundling = edges.map((e) => ({
    id: e.relationshipId,
    sourceId: e.sourceId,
    targetId: e.targetId,
    points: [...e.points] as Point[],
  }));
  const { bundles, adjustedPoints } = bundleEdges(edgesForBundling, {
    angleTolerance: 5,
    positionTolerance: 20,
    fanOutSpacing: 15,
    minBundleSize: 2,
  });

  // Apply bundled points back to edges
  for (const edge of edges) {
    const adjusted = adjustedPoints.get(edge.relationshipId);
    if (adjusted) {
      (edge as any).points = adjusted;
    }
  }
  phases.push({
    name: "bundleEdges",
    durationMs: Date.now() - bundle0,
    nodesProcessed: bundles.length,
  });

  let nodesOut = new Map<string, PositionedC4Node>();
  // Calculate z-index based on depth: deeper nodes (children) should have higher z-index
  // This ensures children render on top of parents, but parents are still visible
  // Root nodes: z-index 0, each level deeper: +10
  // This creates proper stacking: parents behind, children in front
  for (const [id, n] of positioned) {
    // Z-index: depth-based, but ensure parents are lower than children
    // For React Flow: higher z-index = renders on top
    // We want: root nodes (0) < children (10) < grandchildren (20), etc.
    // But also ensure parents are always lower than their direct children
    const baseZIndex = (n.depth || 0) * 10;
    // Additional: if node has children, ensure it's lower than them
    const hasChildren = n.children && n.children.length > 0;
    const zIndex = hasChildren ? baseZIndex : baseZIndex + 5;
    nodesOut.set(id, {
      nodeId: id,
      bbox: n.bbox,
      contentBox: n.bbox,
      labelBox: n.bbox,
      parentId: n.parent?.id,
      childrenIds: n.children.map((c) => c.id),
      depth: n.depth,
      level: n.node.level,
      collapsed: !!n.node.collapseChildren,
      visible: !n.node.hidden,
      zIndex,
      ports: [],
    });
  }

  // Label placement: avoid collisions between edge labels and nodes
  const label0 = Date.now();
  const nodeMap = new Map<string, NodeForLabeling>();
  for (const [id, n] of nodesOut) {
    nodeMap.set(id, { id, bbox: n.bbox });
  }
  const edgesForLabeling = edges.map((e) => ({
    id: e.relationshipId,
    label: graph.relationships.find((r) => r.id === e.relationshipId)?.label,
    points: e.points,
  }));
  // Use increased padding for better label placement to avoid overlaps
  // Use more aggressive label placement for dense graphs and expanded nodes
  const isDenseOrExpanded = needsEnhancedRouting;
  // For expanded nodes with many edges, use even more padding and iterations to reduce overlaps
  const edgeCount = edgesForLabeling.length;
  const labelPadding = hasExpandedNodes ? (edgeCount > 20 ? 60 : 50) : isDenseOrExpanded ? 40 : 25;
  const labelIterations = hasExpandedNodes
    ? edgeCount > 20
      ? 50
      : 40
    : isDenseOrExpanded
      ? 30
      : 18;
  // Increased padding and iterations for expanded/hierarchical diagrams to avoid overlaps
  const finalLabelPadding = hasExpandedNodes ? Math.max(labelPadding, 40) : labelPadding;
  const finalLabelIterations = hasExpandedNodes ? Math.max(labelIterations, 30) : labelIterations;

  const labelPlacements = placeLabels(edgesForLabeling, nodeMap, {
    padding: finalLabelPadding,
    maxShiftIterations: finalLabelIterations,
    allowRotation: false,
  });

  // Helper to verify and enforce strict parent containment
  function enforceParentContainment(
    nodes: Map<string, PositionedC4Node>
  ): Map<string, PositionedC4Node> {
    // Process nodes by depth (deepest first) to propagate size changes up
    // Calculate max depth
    let maxDepth = 0;
    for (const node of nodes.values()) {
      maxDepth = Math.max(maxDepth, node.depth);
    }

    // Iterate from maxDepth down to 0
    for (let d = maxDepth; d >= 0; d--) {
      const layerNodes = Array.from(nodes.values()).filter((n) => n.depth === d);

      for (const node of layerNodes) {
        if (!node.childrenIds || node.childrenIds.length === 0) continue;

        let minX = Infinity;
        let minY = Infinity;
        let maxX = -Infinity;
        let maxY = -Infinity;
        let hasChildren = false;

        for (const childId of node.childrenIds) {
          const child = nodes.get(childId);
          if (child) {
            hasChildren = true;
            // Child coordinates are absolute
            minX = Math.min(minX, child.bbox.x);
            minY = Math.min(minY, child.bbox.y);
            maxX = Math.max(maxX, child.bbox.x + child.bbox.width);
            maxY = Math.max(maxY, child.bbox.y + child.bbox.height);
          }
        }

        if (hasChildren) {
          // Calculate required padding
          // Use safer padding logic consistent with sizing.ts
          // Better: we don't have kind strictly typed here, but we can infer or use default
          const basePadding = 20;

          // Ensure parent bbox covers all children
          const currentBBox = node.bbox;

          // Check if containment is violated
          const SAFETY_MARGIN = 24; // Extra internal padding
          const requiredLeft = minX - SAFETY_MARGIN - basePadding;
          const requiredTop = minY - SAFETY_MARGIN - basePadding; // Header height? We don't have it easily here

          // Note: We can't easily know header height here without re-measuring.
          // But we can ensure at least the bbox contains children.
          // We preserve the header area by assuming top padding covers it or existing top is valid

          // If child is to the left/top of parent, expand parent left/top
          const newX = Math.min(currentBBox.x, requiredLeft);
          const newY = Math.min(currentBBox.y, requiredTop);

          // If child is to right/bottom, expand parent width/height
          const requiredRight = maxX + SAFETY_MARGIN + basePadding;
          const requiredBottom = maxY + SAFETY_MARGIN + basePadding;

          const currentRight = currentBBox.x + currentBBox.width;
          const currentBottom = currentBBox.y + currentBBox.height;

          const newRight = Math.max(currentRight, requiredRight);
          const newBottom = Math.max(currentBottom, requiredBottom);

          const newWidth = newRight - newX;
          const newHeight = newBottom - newY;

          // Only update if changed
          if (
            newX !== currentBBox.x ||
            newY !== currentBBox.y ||
            newWidth !== currentBBox.width ||
            newHeight !== currentBBox.height
          ) {
            nodes.set(node.nodeId, {
              ...node,
              bbox: { x: newX, y: newY, width: newWidth, height: newHeight },
              contentBox: { ...node.contentBox, x: newX + basePadding, y: newY + basePadding }, // Approximate
            });
          }
        }
      }
    }

    return nodes;
  }

  // Apply label placements back to edges
  for (const edge of edges) {
    const placement = labelPlacements.get(edge.relationshipId);
    if (placement) {
      (edge as any).labelPosition = placement.position;
      (edge as any).labelAngle = placement.rotation;
      (edge as any).labelBounds = placement.bounds;
    }
  }
  phases.push({
    name: "placeLabels",
    durationMs: Date.now() - label0,
    nodesProcessed: labelPlacements.size,
  });

  const b0 = Date.now();
  beautify(new Map([...positioned]), {
    alignNodes: options.beautify.alignNodes,
    gridSize: view.gridSize,
    snapToGrid: view.snapToGrid,
  });
  applyPostProcessors(nodesOut as any, edges as any);
  phases.push({ name: "beautify", durationMs: Date.now() - b0, nodesProcessed: positioned.size });

  // Calculate current bounds before viewport expansion
  let minX = Infinity,
    minY = Infinity,
    maxX = -Infinity,
    maxY = -Infinity;
  for (const n of nodesOut.values()) {
    minX = Math.min(minX, n.bbox.x);
    minY = Math.min(minY, n.bbox.y);
    maxX = Math.max(maxX, n.bbox.x + n.bbox.width);
    maxY = Math.max(maxY, n.bbox.y + n.bbox.height);
  }

  // Viewport expansion: expand diagram to better utilize viewport space (improves viewport utilization score)
  // Use default viewport if not provided (typical web viewport)
  const defaultViewport = { width: 1920, height: 1080 };
  const viewport = defaultViewport;
  const currentWidth = maxX - minX;
  const currentHeight = maxY - minY;

  // Only expand if diagram uses less than 50% of viewport (very compact)
  if (currentWidth < viewport.width * 0.5 || currentHeight < viewport.height * 0.5) {
    const expanded = expandToViewport(nodesOut, viewport, {
      targetUtilization: 0.7, // Target 70% utilization (was 0.75, slightly lower for safety)
      preserveAspectRatio: true,
      minExpansion: 1.0,
      maxExpansion: 1.8, // Limit expansion to avoid excessive scaling
    });

    // Update nodes if expansion occurred and is beneficial
    if (expanded.size === nodesOut.size) {
      // Recalculate bounds after expansion
      let expandedMinX = Infinity,
        expandedMinY = Infinity;
      let expandedMaxX = -Infinity,
        expandedMaxY = -Infinity;
      for (const n of expanded.values()) {
        expandedMinX = Math.min(expandedMinX, n.bbox.x);
        expandedMinY = Math.min(expandedMinY, n.bbox.y);
        expandedMaxX = Math.max(expandedMaxX, n.bbox.x + n.bbox.width);
        expandedMaxY = Math.max(expandedMaxY, n.bbox.y + n.bbox.height);
      }
      const expandedWidth = expandedMaxX - expandedMinX;
      const expandedHeight = expandedMaxY - expandedMinY;

      // Use expanded if it improves utilization without excessive scaling
      const widthImprovement = expandedWidth / currentWidth;
      const heightImprovement = expandedHeight / currentHeight;
      if (
        (widthImprovement > 1.1 || heightImprovement > 1.1) &&
        widthImprovement < 2.0 &&
        heightImprovement < 2.0
      ) {
        nodesOut = expanded;
        minX = expandedMinX;
        minY = expandedMinY;
        maxX = expandedMaxX;
        maxY = expandedMaxY;
      }
    }
  }

  // Phase 7: Post-Processing & Validation
  // Ensure strict containment of children within parents
  // This must happen LAST to catch any movements from previous phases
  const container0 = Date.now();
  if (nodesOut) {
    nodesOut = enforceParentContainment(nodesOut);
  } else {
    // Fallback if nodesOut not defined (should be positioned)
    // Looking at code flow, positioned seems to be the main var until late
  }
  phases.push({
    name: "enforceContainment",
    durationMs: Date.now() - container0,
    nodesProcessed: nodesOut ? nodesOut.size : 0,
  });

  const bbox = { x: minX, y: minY, width: maxX - minX, height: maxY - minY };
  const center = { x: bbox.x + bbox.width / 2, y: bbox.y + bbox.height / 2 };
  const metrics = calculateMetrics(
    // metrics calc line 1644
    new Map(
      [...nodesOut].map(([id, n]) => [
        id,
        { x: n.bbox.x, y: n.bbox.y, size: { width: n.bbox.width, height: n.bbox.height } },
      ])
    ),
    edges.map((e) => ({ ...e, points: [...e.points] }))
  );
  const end = Date.now();
  return {
    nodes: nodesOut,
    relationships: edges,
    bbox,
    center,
    metrics,
    debug: { layoutTimeMs: end - start, phases, warnings: [] },
  };
}

export async function layoutAsync(
  graph: C4Graph,
  view: C4ViewState,
  options: C4LayoutOptions = InteractivePreset
): Promise<C4LayoutResult> {
  return layout(graph, view, options);
}
