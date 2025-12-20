/**
 * Edge Routing Phase - Enhanced
 *
 * Routes edges between nodes with:
 * - Smart port selection with usage diversity
 * - Obstacle avoidance
 * - Bidirectional pair detection
 * - Global crossing minimization
 * - Corner rounding for visual polish
 *
 * Migrated from legacy c4-layout.ts and algorithms/edge-router.ts
 */

import type {
  LayoutPhase,
  LayoutContext,
  LayoutEdge,
  LayoutNode,
  Point,
  Rect,
} from "../core/types";

// Import sophisticated routing algorithms from legacy
import {
  calculateBestPortWithObstacles,
  routeOrthogonalAvoid,
  pathMidpoint,
  arrowAngle,
  pathLength,
  applyCornerRounding,
} from "../algorithms/edge-router";

import { routeWithLayers } from "../algorithms/crossing-minimizer";
import { bundleEdges } from "../algorithms/edge-bundler";

// ============================================================================
// PHASE DEFINITION
// ============================================================================

export function createEdgeRoutingPhase(): LayoutPhase {
  return {
    name: "edge-routing",
    description: "Route edges with obstacle avoidance and crossing minimization",
    dependencies: ["layout"],
    execute: (context: LayoutContext): LayoutContext => {
      const edges = new Map<string, LayoutEdge>();

      // Build port usage map for edge distribution
      const portUsage = new Map<string, number>();

      // Step 1: Detect bidirectional pairs for special handling
      const bidirectionalPairs = detectBidirectionalPairs(context);

      // Step 2: Build obstacles list and node depth map
      const nodeDepths = buildNodeDepthMap(context.nodes);

      // Step 3: Check if we need enhanced routing (dense or expanded graphs)
      const totalEdges = context.graph.relationships.length;
      const totalNodes = context.nodes.size;
      const isDenseGraph = totalEdges > 10 || totalEdges / totalNodes > 1.5;
      const hasExpandedNodes = Array.from(context.nodes.values()).some(
        (n) => n.parent && n.visible
      );
      const needsEnhancedRouting = isDenseGraph || hasExpandedNodes;

      // Step 4: Sort edges by routing priority
      const sortedRelationships = sortByRoutingPriority(context, bidirectionalPairs);

      // Step 5: Route edges with appropriate strategy
      if (needsEnhancedRouting && totalEdges > 8) {
        // Use global crossing minimization for complex graphs
        const routedEdges = routeWithGlobalMinimization(
          context,
          sortedRelationships,
          nodeDepths,
          portUsage,
          bidirectionalPairs
        );
        for (const edge of routedEdges) {
          edges.set(edge.id, edge);
        }
      } else {
        // Standard routing for simpler graphs with incremental crossing avoidance
        const placedEdges: Array<{ sourceId: string; targetId: string; path: Point[] }> = [];
        for (const relationship of sortedRelationships) {
          const edge = routeEdge(
            relationship,
            context,
            portUsage,
            bidirectionalPairs,
            placedEdges
          );
          if (edge) {
            edges.set(relationship.id, edge);
            // Track placed edge for subsequent routing
            placedEdges.push({
              sourceId: edge.source.id,
              targetId: edge.target.id,
              path: [...edge.points],
            });
          }
        }
      }

      // Step 6: Apply edge bundling to group parallel edges
      applyEdgeBundling(edges);

      // Step 7: Apply corner rounding for visual polish
      if (context.options.edgeRouting.algorithm !== "splines") {
        for (const [id, edge] of edges) {
          // Skip rounding for curved edges or those explicitly marked as direct/curved
          const preferred = (edge.original as any).preferredRoute;
          if (preferred === 'curved' || preferred === 'splines' || preferred === 'direct') {
            continue;
          }

          const roundedEdge = applyRounding(edge, context);
          edges.set(id, roundedEdge);
        }
      }

      return {
        ...context,
        edges,
      };
    },
  };
}

// ============================================================================
// BIDIRECTIONAL PAIR DETECTION
// ============================================================================

interface BidirectionalPair {
  forwardId: string;
  reverseId: string;
  count: number;
}

function detectBidirectionalPairs(
  context: LayoutContext
): Map<string, BidirectionalPair> {
  const pairs = new Map<string, BidirectionalPair>();

  // CRITICAL: Sort relationships for deterministic processing order
  const sortedRelationships = [...context.graph.relationships].sort((a, b) => 
    a.id.localeCompare(b.id)
  );

  for (const rel of sortedRelationships) {
    const forwardKey = `${rel.from}::${rel.to}`;
    const reverseKey = `${rel.to}::${rel.from}`;

    if (pairs.has(reverseKey)) {
      // Found a bidirectional pair
      const existing = pairs.get(reverseKey)!;
      existing.count++;
      pairs.set(forwardKey, {
        forwardId: rel.id,
        reverseId: existing.forwardId,
        count: existing.count,
      });
    } else {
      pairs.set(forwardKey, {
        forwardId: rel.id,
        reverseId: "",
        count: 1,
      });
    }
  }

  return pairs;
}

// ============================================================================
// NODE DEPTH MAP
// ============================================================================

function buildNodeDepthMap(nodes: Map<string, LayoutNode>): Map<string, number> {
  const depths = new Map<string, number>();

  // CRITICAL: Sort nodes for deterministic processing order
  const sortedNodes = Array.from(nodes.values()).sort((a, b) => a.id.localeCompare(b.id));
  for (const node of sortedNodes) {
    depths.set(node.id, node.depth);
  }

  return depths;
}

// ============================================================================
// ROUTING PRIORITY SORTING
// ============================================================================

function sortByRoutingPriority(
  context: LayoutContext,
  bidirectionalPairs: Map<string, BidirectionalPair>
): typeof context.graph.relationships {
  const relationships = [...context.graph.relationships];

  return relationships.sort((a, b) => {
    // 1. Parent-child edges have highest priority (route them first)
    const aSource = context.nodes.get(a.from);
    const aTarget = context.nodes.get(a.to);
    const bSource = context.nodes.get(b.from);
    const bTarget = context.nodes.get(b.to);

    if (!aSource || !aTarget || !bSource || !bTarget) return 0;

    const aIsParentChild =
      aSource.parent?.id === aTarget.id || aTarget.parent?.id === aSource.id;
    const bIsParentChild =
      bSource.parent?.id === bTarget.id || bTarget.parent?.id === bSource.id;

    if (aIsParentChild && !bIsParentChild) return -1;
    if (!aIsParentChild && bIsParentChild) return 1;

    // 2. Bidirectional pairs get routed together
    const aBidi = bidirectionalPairs.has(`${a.from}::${a.to}`);
    const bBidi = bidirectionalPairs.has(`${b.from}::${b.to}`);
    if (aBidi && !bBidi) return -1;
    if (!aBidi && bBidi) return 1;

    // 3. Shorter edges before longer ones
    const aDist = Math.hypot(
      aTarget.bbox.x - aSource.bbox.x,
      aTarget.bbox.y - aSource.bbox.y
    );
    const bDist = Math.hypot(
      bTarget.bbox.x - bSource.bbox.x,
      bTarget.bbox.y - bSource.bbox.y
    );

    return aDist - bDist;
  });
}

// ============================================================================
// GLOBAL CROSSING MINIMIZATION
// ============================================================================

function routeWithGlobalMinimization(
  context: LayoutContext,
  relationships: typeof context.graph.relationships,
  nodeDepths: Map<string, number>,
  portUsage: Map<string, number>,
  _bidirectionalPairs: Map<string, BidirectionalPair>
): LayoutEdge[] {
  // Prepare edges for global minimization
  // CRITICAL: Sort relationships for deterministic processing order
  const sortedRelationships = [...relationships].sort((a, b) => a.id.localeCompare(b.id));
  const edgesForMinimization = sortedRelationships
    .map((rel) => {
      const source = context.nodes.get(rel.from);
      const target = context.nodes.get(rel.to);
      if (!source || !target || !source.visible || !target.visible) return null;

      // Build obstacles list (all nodes except source, target, and their ancestors)
      const srcAncestors = getAncestorIds(source, context.nodes);
      const dstAncestors = getAncestorIds(target, context.nodes);

      // CRITICAL: Sort obstacles for deterministic processing order
      const obstacles = Array.from(context.nodes.values())
        .filter(
          (n) =>
            n.id !== source.id &&
            n.id !== target.id &&
            !srcAncestors.has(n.id) &&
            !dstAncestors.has(n.id) &&
            n.visible
        )
        .sort((a, b) => a.id.localeCompare(b.id))
        .map((n) => n.bbox);

      return {
        id: rel.id,
        sourceId: rel.from,
        targetId: rel.to,
        source: {
          id: source.id,
          bbox: source.bbox,
          parentId: source.parent?.id,
        },
        target: {
          id: target.id,
          bbox: target.bbox,
          parentId: target.parent?.id,
        },
        obstacles,
        getPorts: (
          sourceSide?: "north" | "south" | "east" | "west",
          targetSide?: "north" | "south" | "east" | "west"
        ) => {
          const srcPort = sourceSide
            ? calculatePortForSide(source.bbox, target.bbox, sourceSide)
            : calculateBestPortWithObstacles(
              source.bbox,
              target.bbox,
              obstacles,
              portUsage
            );
          const tgtPort = targetSide
            ? calculatePortForSide(target.bbox, source.bbox, targetSide)
            : calculateBestPortWithObstacles(
              target.bbox,
              source.bbox,
              obstacles,
              portUsage
            );
          return { sourcePort: srcPort, targetPort: tgtPort };
        },
      };
    })
    .filter((e): e is NonNullable<typeof e> => e !== null);

  // Use layer-based routing which internally calls global minimization
  const optimizedRoutes = routeWithLayers(edgesForMinimization, nodeDepths, pathsCross);

  // Convert to LayoutEdge format
  // CRITICAL: Sort routes for deterministic processing order
  const sortedRoutes = [...optimizedRoutes].sort((a, b) => a.id.localeCompare(b.id));
  return sortedRoutes.map((route) => {
    const rel = context.graph.relationships.find((r) => r.id === route.id)!;
    const source = context.nodes.get(rel.from)!;
    const target = context.nodes.get(rel.to)!;

    const midPoint = pathMidpoint(route.path);
    const arrowEnd = route.path[route.path.length - 1];
    const arrowStart =
      route.path.length > 1 ? route.path[route.path.length - 2] : route.path[0];

    return createLayoutEdge(
      rel,
      source,
      target,
      route.path,
      midPoint,
      arrowAngle(arrowStart, arrowEnd),
      route.sourcePort,
      route.targetPort
    );
  });
}

// ============================================================================
// STANDARD EDGE ROUTING
// ============================================================================

function routeEdge(
  relationship: { id: string; from: string; to: string; label?: string },
  context: LayoutContext,
  portUsage: Map<string, number>,
  bidirectionalPairs: Map<string, BidirectionalPair>,
  placedEdges: Array<{ sourceId: string; targetId: string; path: Point[] }> = []
): LayoutEdge | null {
  const source = context.nodes.get(relationship.from);
  const target = context.nodes.get(relationship.to);

  if (!source || !target || !source.visible || !target.visible) return null;

  // Check relationship preference
  const preferredRoute = (relationship as any).preferredRoute;

  // DIRECT ROUTING
  if (preferredRoute === 'direct') {
    const bestSourcePort = calculateBestPortWithObstacles(source.bbox, target.bbox, [], portUsage);
    const bestTargetPort = calculateBestPortWithObstacles(target.bbox, source.bbox, [], portUsage);

    const path = [bestSourcePort.position, bestTargetPort.position];
    const midPoint = pathMidpoint(path);

    return {
      ...createLayoutEdge(
        relationship,
        source,
        target,
        path,
        midPoint,
        arrowAngle(path[0], path[1]),
        bestSourcePort,
        bestTargetPort
      ),
      segmentTypes: ['line']
    };
  }

  // CURVED ROUTING (Splines)
  if (preferredRoute === 'curved' || preferredRoute === 'splines') {
    const bestSourcePort = calculateBestPortWithObstacles(source.bbox, target.bbox, [], portUsage);
    const bestTargetPort = calculateBestPortWithObstacles(target.bbox, source.bbox, [], portUsage);

    // Basic spline routing - populate control points
    // In a real implementation this would call routeSpline from algorithms
    // For now we will create control points for a smooth curve
    const path = [bestSourcePort.position, bestTargetPort.position];
    const midPoint = pathMidpoint(path);

    const dx = bestTargetPort.position.x - bestSourcePort.position.x;
    const dy = bestTargetPort.position.y - bestSourcePort.position.y;

    // Simple control points: move out 30% of distance in direction of travel
    const cp1 = {
      x: bestSourcePort.position.x + dx * 0.3,
      y: bestSourcePort.position.y + dy * 0.1
    };

    const cp2 = {
      x: bestTargetPort.position.x - dx * 0.3,
      y: bestTargetPort.position.y - dy * 0.1
    };

    return {
      ...createLayoutEdge(
        relationship,
        source,
        target,
        path,
        midPoint,
        arrowAngle(path[0], path[1]),
        bestSourcePort,
        bestTargetPort
      ),
      controlPoints: [cp1, cp2],
      segmentTypes: ['arc']
    };
  }

  // Build obstacles list
  const srcAncestors = getAncestorIds(source, context.nodes);
  const dstAncestors = getAncestorIds(target, context.nodes);

  // CRITICAL: Sort obstacles for deterministic processing order
  const obstacles = Array.from(context.nodes.values())
    .filter(
      (n) =>
        n.id !== source.id &&
        n.id !== target.id &&
        !srcAncestors.has(n.id) &&
        !dstAncestors.has(n.id) &&
        n.visible
    )
    .sort((a, b) => a.id.localeCompare(b.id))
    .map((n) => n.bbox);

  // Handle bidirectional pairs - offset one edge slightly
  const pairKey = `${relationship.from}::${relationship.to}`;
  const isBidirectional = bidirectionalPairs.has(pairKey);
  let offsetAmount = 0;
  if (isBidirectional) {
    const pair = bidirectionalPairs.get(pairKey)!;
    if (pair.reverseId && relationship.id !== pair.forwardId) {
      offsetAmount = 8; // Offset the reverse edge
    }
  }

  const basePadding = context.options.edgeRouting.padding || 20;

  // Try to find the best path with minimum crossings
  let bestPath: Point[] | null = null;
  let bestSourcePort = calculateBestPortWithObstacles(source.bbox, target.bbox, obstacles, portUsage);
  let bestTargetPort = calculateBestPortWithObstacles(target.bbox, source.bbox, obstacles, portUsage);
  let minCrossings = Infinity;

  // If we have placed edges, try multiple routing strategies
  if (placedEdges.length > 0) {
    // Try multiple padding values
    const paddingMultipliers = [1.0, 1.3, 1.6, 2.0];

    for (const multiplier of paddingMultipliers) {
      const testPadding = basePadding * multiplier + offsetAmount;
      const testPath = routeOrthogonalAvoid(
        { position: bestSourcePort.position, side: bestSourcePort.side },
        { position: bestTargetPort.position, side: bestTargetPort.side },
        obstacles,
        testPadding
      );

      const crossings = countCrossingsWithPlaced(testPath, source.id, target.id, placedEdges);
      if (crossings < minCrossings) {
        minCrossings = crossings;
        bestPath = testPath;
        if (crossings === 0) break; // Found perfect path
      }
    }

    // If still have crossings, try all 16 port combinations
    if (minCrossings > 0) {
      const allSides: Array<"north" | "south" | "east" | "west"> = ["north", "south", "east", "west"];

      for (const srcSide of allSides) {
        for (const tgtSide of allSides) {
          const srcPort = createPortForSide(source.bbox, target.bbox, srcSide);
          const tgtPort = createPortForSide(target.bbox, source.bbox, tgtSide);

          for (const multiplier of [1.0, 1.5]) {
            const testPadding = basePadding * multiplier + offsetAmount;
            const testPath = routeOrthogonalAvoid(
              { position: srcPort.position, side: srcPort.side },
              { position: tgtPort.position, side: tgtPort.side },
              obstacles,
              testPadding
            );

            const crossings = countCrossingsWithPlaced(testPath, source.id, target.id, placedEdges);
            if (crossings < minCrossings) {
              minCrossings = crossings;
              bestPath = testPath;
              bestSourcePort = srcPort;
              bestTargetPort = tgtPort;
              if (crossings === 0) break;
            }
          }
          if (minCrossings === 0) break;
        }
        if (minCrossings === 0) break;
      }
    }
  }

  // If no best path found, use default routing
  if (!bestPath) {
    bestPath = routeOrthogonalAvoid(
      { position: bestSourcePort.position, side: bestSourcePort.side },
      { position: bestTargetPort.position, side: bestTargetPort.side },
      obstacles,
      basePadding + offsetAmount
    );
  }

  // Calculate midpoint and arrow angle
  const midPoint = pathMidpoint(bestPath);
  const arrowEnd = bestPath[bestPath.length - 1];
  const arrowStart = bestPath.length > 1 ? bestPath[bestPath.length - 2] : bestPath[0];

  return createLayoutEdge(
    relationship,
    source,
    target,
    bestPath,
    midPoint,
    arrowAngle(arrowStart, arrowEnd),
    bestSourcePort,
    bestTargetPort
  );
}

function countCrossingsWithPlaced(
  path: Point[],
  sourceId: string,
  targetId: string,
  placedEdges: Array<{ sourceId: string; targetId: string; path: Point[] }>
): number {
  let crossings = 0;
  for (const placed of placedEdges) {
    if (pathsCross(path, placed.path, sourceId, targetId, placed.sourceId, placed.targetId)) {
      crossings++;
    }
  }
  return crossings;
}

function createPortForSide(
  source: Rect,
  target: Rect,
  side: "north" | "south" | "east" | "west"
): { side: "north" | "south" | "east" | "west"; position: Point; angle: number } {
  const targetCenterX = target.x + target.width / 2;
  const targetCenterY = target.y + target.height / 2;

  switch (side) {
    case "north":
      return {
        side,
        position: { x: Math.max(source.x, Math.min(source.x + source.width, targetCenterX)), y: source.y },
        angle: 270,
      };
    case "south":
      return {
        side,
        position: { x: Math.max(source.x, Math.min(source.x + source.width, targetCenterX)), y: source.y + source.height },
        angle: 90,
      };
    case "east":
      return {
        side,
        position: { x: source.x + source.width, y: Math.max(source.y, Math.min(source.y + source.height, targetCenterY)) },
        angle: 0,
      };
    case "west":
      return {
        side,
        position: { x: source.x, y: Math.max(source.y, Math.min(source.y + source.height, targetCenterY)) },
        angle: 180,
      };
  }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

function getAncestorIds(
  node: LayoutNode,
  _allNodes: Map<string, LayoutNode>
): Set<string> {
  const ancestors = new Set<string>();
  let current = node.parent;
  while (current) {
    ancestors.add(current.id);
    current = current.parent;
  }
  return ancestors;
}

function calculatePortForSide(
  source: Rect,
  _target: Rect,
  side: "north" | "south" | "east" | "west"
): { side: "north" | "south" | "east" | "west"; position: Point; angle: number } {
  const centerX = source.x + source.width / 2;
  const centerY = source.y + source.height / 2;

  switch (side) {
    case "north":
      return { side, position: { x: centerX, y: source.y }, angle: 270 };
    case "south":
      return {
        side,
        position: { x: centerX, y: source.y + source.height },
        angle: 90,
      };
    case "east":
      return {
        side,
        position: { x: source.x + source.width, y: centerY },
        angle: 0,
      };
    case "west":
      return { side, position: { x: source.x, y: centerY }, angle: 180 };
  }
}

function createLayoutEdge(
  relationship: { id: string; from: string; to: string; label?: string },
  source: LayoutNode,
  target: LayoutNode,
  path: Point[],
  midPoint: Point,
  arrowAng: number,
  sourcePort?: { side: "north" | "south" | "east" | "west"; position: Point; angle: number },
  targetPort?: { side: "north" | "south" | "east" | "west"; position: Point; angle: number }
): LayoutEdge {
  return {
    id: relationship.id,
    original: relationship as any,
    source,
    target,
    points: path,
    segmentTypes: path.map(() => "orthogonal" as const),
    sourcePort: sourcePort
      ? { id: `${source.id}-${sourcePort.side}`, ...sourcePort, usage: 1, capacity: 10 }
      : undefined,
    targetPort: targetPort
      ? { id: `${target.id}-${targetPort.side}`, ...targetPort, usage: 1, capacity: 10 }
      : undefined,
    labelPosition: midPoint,
    labelAngle: 0,
    labelBounds: {
      x: midPoint.x - 50,
      y: midPoint.y - 10,
      width: 100,
      height: 20,
    },
    arrowEnd: path[path.length - 1],
    arrowAngle: arrowAng,
    length: pathLength(path),
    bendCount: path.length - 2,
    crossesBoundaries: source.parent?.id !== target.parent?.id,
  };
}

function applyRounding(edge: LayoutEdge, _context: LayoutContext): LayoutEdge {
  try {
    const rounded = applyCornerRounding([...edge.points], {
      cornerRadius: 8,
      style: "rounded",
      minSegmentLength: 20,
    });

    return {
      ...edge,
      points: rounded.points,
      controlPoints: rounded.cornerControlPoints.map((c) => c.control),
      segmentTypes: rounded.segmentTypes,
    };
  } catch {
    // If rounding fails, return original edge
    return edge;
  }
}

/**
 * Check if two paths cross each other
 * Used by crossing minimization algorithm
 */
function pathsCross(
  path1: Point[],
  path2: Point[],
  from1?: string,
  to1?: string,
  from2?: string,
  to2?: string
): boolean {
  // Don't count as crossing if edges share a node
  if (from1 === from2 || from1 === to2 || to1 === from2 || to1 === to2) {
    return false;
  }

  for (let i = 0; i < path1.length - 1; i++) {
    for (let j = 0; j < path2.length - 1; j++) {
      if (
        lineSegmentsIntersect(path1[i], path1[i + 1], path2[j], path2[j + 1])
      ) {
        return true;
      }
    }
  }
  return false;
}

function lineSegmentsIntersect(
  p1: Point,
  p2: Point,
  p3: Point,
  p4: Point
): boolean {
  const det = (p2.x - p1.x) * (p4.y - p3.y) - (p4.x - p3.x) * (p2.y - p1.y);
  if (Math.abs(det) < 1e-10) return false;

  const lambda =
    ((p4.y - p3.y) * (p4.x - p1.x) + (p3.x - p4.x) * (p4.y - p1.y)) / det;
  const gamma =
    ((p1.y - p2.y) * (p4.x - p1.x) + (p2.x - p1.x) * (p4.y - p1.y)) / det;

  return 0 < lambda && lambda < 1 && 0 < gamma && gamma < 1;
}

// ============================================================================
// EDGE BUNDLING
// ============================================================================

function applyEdgeBundling(edges: Map<string, LayoutEdge>): void {
  if (edges.size < 2) return;

  // Prepare edges for bundling
  // CRITICAL: Sort edges for deterministic processing order
  const edgesForBundling = Array.from(edges.values())
    .sort((a, b) => a.id.localeCompare(b.id))
    .map((e) => ({
      id: e.id,
      sourceId: e.source.id,
      targetId: e.target.id,
      points: [...e.points] as Point[],
    }));

  // Enhanced bundling: bundle parallel edges (same source/target tier) more aggressively
  // This reduces crossings by grouping similar edges together
  const { adjustedPoints } = bundleEdges(edgesForBundling, {
    angleTolerance: 8, // Increased from 5 to bundle more similar edges
    positionTolerance: 30, // Increased from 20 to catch more parallel edges
    fanOutSpacing: 12, // Slightly reduced for tighter bundling
    minBundleSize: 2, // Bundle any pair of parallel edges
  });

  // Update edge points with bundled positions
  for (const [id, edge] of edges) {
    const adjusted = adjustedPoints.get(id);
    if (adjusted && adjusted.length > 0) {
      edges.set(id, {
        ...edge,
        points: adjusted,
        // Recalculate dependent properties
        labelPosition: pathMidpoint(adjusted),
        arrowEnd: adjusted[adjusted.length - 1],
        arrowAngle: arrowAngle(
          adjusted.length > 1 ? adjusted[adjusted.length - 2] : adjusted[0],
          adjusted[adjusted.length - 1]
        ),
        length: pathLength(adjusted),
        bendCount: adjusted.length - 2,
      });
    }
  }
}
