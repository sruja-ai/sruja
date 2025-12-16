/**
 * Clutter Detection Algorithm
 *
 * Computes a clutter score based on:
 * - Node proximity (nodes too close together)
 * - Edge crossings
 * - Edge routing complexity (bends, length)
 * - Layout density
 */

import type { Point } from "../geometry/point";
import type { Rect } from "../geometry/rect";
import { CLUTTER_THRESHOLD, NO_ENTRY_MARGIN } from "../constants";

export interface ClutterScore {
  /** 0-1: How close nodes are to each other (0 = well spaced, 1 = overlapping) */
  nodeProximity: number;
  /** Number of edge crossings normalized by total edges */
  edgeCrossings: number;
  /** Average bends per edge normalized */
  routingComplexity: number;
  /** Ratio of used space to total bounding box (higher = denser) */
  density: number;
  /** Combined weighted score 0-1 */
  total: number;
  /** Whether the score exceeds the optimization threshold */
  needsOptimization: boolean;
}

export interface NodeBox {
  id: string;
  bbox: Rect;
}

export interface EdgePath {
  id: string;
  points: readonly Point[];
}

/**
 * Calculate the minimum distance between two rectangles
 */
function rectDistance(a: Rect, b: Rect): number {
  // Optimized: cache boundaries to avoid repeated calculations
  const aRight = a.x + a.width;
  const aBottom = a.y + a.height;
  const bRight = b.x + b.width;
  const bBottom = b.y + b.height;

  const left = bRight < a.x;
  const right = aRight < b.x;
  const top = bBottom < a.y;
  const bottom = aBottom < b.y;

  if (left && top) {
    // Optimized: use squared distance calculation (same ordering, faster)
    const dx = a.x - bRight;
    const dy = a.y - bBottom;
    return Math.sqrt(dx * dx + dy * dy);
  }
  if (left && bottom) {
    const dx = a.x - bRight;
    const dy = b.y - aBottom;
    return Math.sqrt(dx * dx + dy * dy);
  }
  if (right && top) {
    const dx = b.x - aRight;
    const dy = a.y - bBottom;
    return Math.sqrt(dx * dx + dy * dy);
  }
  if (right && bottom) {
    const dx = b.x - aRight;
    const dy = b.y - aBottom;
    return Math.sqrt(dx * dx + dy * dy);
  }
  if (left) return a.x - bRight;
  if (right) return b.x - aRight;
  if (top) return a.y - bBottom;
  if (bottom) return b.y - aBottom;

  // Overlapping
  return 0;
}

/**
 * Check if two line segments intersect
 */
function segmentsIntersect(a1: Point, a2: Point, b1: Point, b2: Point): boolean {
  const ccw = (p1: Point, p2: Point, p3: Point) =>
    (p3.y - p1.y) * (p2.x - p1.x) > (p2.y - p1.y) * (p3.x - p1.x);

  return ccw(a1, b1, b2) !== ccw(a2, b1, b2) && ccw(a1, a2, b1) !== ccw(a1, a2, b2);
}

/**
 * Count intersections between two edge paths
 */
function countEdgeCrossings(edges: EdgePath[]): number {
  let crossings = 0;
  // Optimized: cache edges length
  const edgesLength = edges.length;

  for (let i = 0; i < edgesLength; i++) {
    const edgeA = edges[i];
    const pathA = edgeA.points;
    const pathALength = pathA.length;

    for (let j = i + 1; j < edgesLength; j++) {
      const edgeB = edges[j];
      const pathB = edgeB.points;
      const pathBLength = pathB.length;

      // Optimized: cache path lengths
      for (let a = 0; a < pathALength - 1; a++) {
        const pointA1 = pathA[a];
        const pointA2 = pathA[a + 1];

        for (let b = 0; b < pathBLength - 1; b++) {
          if (segmentsIntersect(pointA1, pointA2, pathB[b], pathB[b + 1])) {
            crossings++;
          }
        }
      }
    }
  }

  return crossings;
}

/**
 * Calculate bounding box of all nodes
 */
function calculateBoundingBox(nodes: NodeBox[]): Rect {
  if (nodes.length === 0) {
    return { x: 0, y: 0, width: 0, height: 0 };
  }

  let minX = Infinity,
    minY = Infinity;
  let maxX = -Infinity,
    maxY = -Infinity;

  // Optimized: cache bbox references to avoid repeated property access
  for (const node of nodes) {
    const bbox = node.bbox;
    const right = bbox.x + bbox.width;
    const bottom = bbox.y + bbox.height;

    minX = Math.min(minX, bbox.x);
    minY = Math.min(minY, bbox.y);
    maxX = Math.max(maxX, right);
    maxY = Math.max(maxY, bottom);
  }

  return { x: minX, y: minY, width: maxX - minX, height: maxY - minY };
}

/**
 * Compute the clutter score for a layout
 */
export function computeClutterScore(
  nodes: NodeBox[],
  edges: EdgePath[],
  options: {
    idealNodeSpacing?: number;
    maxEdgeCrossings?: number;
    threshold?: number;
  } = {}
): ClutterScore {
  const {
    idealNodeSpacing = NO_ENTRY_MARGIN * 4,
    maxEdgeCrossings = Math.max(10, edges.length * 2),
    threshold = CLUTTER_THRESHOLD,
  } = options;

  // 1. Node proximity score
  let proximitySum = 0;
  let proximityCount = 0;
  // Optimized: cache nodes length and idealNodeSpacing inverse
  const nodesLength = nodes.length;
  const invIdealSpacing = 1 / idealNodeSpacing;

  for (let i = 0; i < nodesLength; i++) {
    const nodeA = nodes[i];
    const bboxA = nodeA.bbox;

    for (let j = i + 1; j < nodesLength; j++) {
      const dist = rectDistance(bboxA, nodes[j].bbox);
      // Score: 1 if overlapping/touching, 0 if >= idealSpacing
      // Optimized: use multiplication instead of division
      const score = dist < idealNodeSpacing ? 1 - dist * invIdealSpacing : 0;
      proximitySum += score;
      proximityCount++;
    }
  }

  const nodeProximity = proximityCount > 0 ? Math.min(1, proximitySum / proximityCount) : 0;

  // 2. Edge crossings score
  const crossings = countEdgeCrossings(edges);
  const edgeCrossings = Math.min(1, crossings / maxEdgeCrossings);

  // 3. Routing complexity score (average bends per edge)
  let totalBends = 0;
  for (const edge of edges) {
    totalBends += Math.max(0, edge.points.length - 2);
  }
  const avgBends = edges.length > 0 ? totalBends / edges.length : 0;
  const routingComplexity = Math.min(1, avgBends / 4); // 4+ bends = max complexity

  // 4. Density score
  const bbox = calculateBoundingBox(nodes);
  const totalArea = bbox.width * bbox.height;
  // Optimized: cache bbox references in reduce
  const usedArea = nodes.reduce((sum, n) => {
    const bbox = n.bbox;
    return sum + bbox.width * bbox.height;
  }, 0);
  const invTotalArea = totalArea > 0 ? 1 / totalArea : 0;
  const density = totalArea > 0 ? Math.min(1, usedArea * invTotalArea) : 0;

  // Weighted total (weights sum to 1)
  const total =
    nodeProximity * 0.35 + edgeCrossings * 0.3 + routingComplexity * 0.2 + density * 0.15;

  return {
    nodeProximity,
    edgeCrossings,
    routingComplexity,
    density,
    total,
    needsOptimization: total > threshold,
  };
}

/**
 * Quick check if layout needs optimization
 */
export function shouldReoptimize(score: ClutterScore, threshold = CLUTTER_THRESHOLD): boolean {
  return score.total > threshold;
}

/**
 * Get optimization suggestions based on clutter score
 */
export function getOptimizationSuggestions(score: ClutterScore): string[] {
  const suggestions: string[] = [];

  if (score.nodeProximity > 0.5) {
    suggestions.push("Increase node spacing or expand grid");
  }
  if (score.edgeCrossings > 0.3) {
    suggestions.push("Reorder nodes to reduce edge crossings");
  }
  if (score.routingComplexity > 0.5) {
    suggestions.push("Consider curved edges or repositioning nodes");
  }
  if (score.density > 0.6) {
    suggestions.push("Expand layout boundaries for more breathing room");
  }

  return suggestions;
}
