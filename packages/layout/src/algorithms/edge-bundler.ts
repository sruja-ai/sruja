/**
 * Edge Bundling Algorithm
 *
 * Groups parallel edges (edges with same or similar source/target)
 * to reduce visual clutter. Bundled edges share a common path and
 * fan out at their endpoints.
 */

import type { Point } from "../geometry/point";

// ============================================================================
// TYPES
// ============================================================================

export interface EdgeForBundling {
  id: string;
  sourceId: string;
  targetId: string;
  points: Point[];
}

export interface EdgeBundle {
  /** IDs of edges in this bundle */
  edgeIds: string[];
  /** Source node ID */
  sourceId: string;
  /** Target node ID */
  targetId: string;
  /** Whether bundle contains reverse direction edges */
  isBidirectional: boolean;
  /** Spread offset for each edge (perpendicular to path) */
  spreadOffsets: Map<string, number>;
}

export interface BundlingOptions {
  /** Maximum angle difference (degrees) between edges to bundle */
  angleTolerance: number;
  /** Maximum position difference to consider edges bundleable */
  positionTolerance: number;
  /** Spacing between bundled edges at fan-out points */
  fanOutSpacing: number;
  /** Minimum edges required to form a bundle */
  minBundleSize: number;
}

export const DEFAULT_BUNDLING_OPTIONS: BundlingOptions = {
  angleTolerance: 25, // Increased to bundle more edges and reduce congestion
  positionTolerance: 80, // Increased for better bundling of parallel edges
  fanOutSpacing: 8, // Reduced for tighter bundles, better congestion management
  minBundleSize: 2,
};

// ============================================================================
// BUNDLING LOGIC
// ============================================================================

/**
 * Identify edges that can be bundled together based on their endpoints.
 *
 * Two edges are bundleable if:
 * 1. They connect the same pair of nodes (same direction = parallel, opposite = bidirectional)
 * 2. Their paths are similar (within tolerance)
 */
export function identifyBundles(
  edges: EdgeForBundling[],
  options: Partial<BundlingOptions> = {}
): EdgeBundle[] {
  const opts = { ...DEFAULT_BUNDLING_OPTIONS, ...options };

  // Group by source-target pair
  const pairMap = new Map<string, EdgeForBundling[]>();

  for (const edge of edges) {
    // Create canonical key (lower ID first for bidirectional matching)
    const forward = `${edge.sourceId}::${edge.targetId}`;
    const reverse = `${edge.targetId}::${edge.sourceId}`;

    // Check if reverse already exists
    if (pairMap.has(reverse)) {
      pairMap.get(reverse)!.push({ ...edge, _isReverse: true } as any);
    } else {
      if (!pairMap.has(forward)) {
        pairMap.set(forward, []);
      }
      pairMap.get(forward)!.push(edge);
    }
  }

  // Convert pairs with enough edges into bundles
  const bundles: EdgeBundle[] = [];

  for (const [key, groupEdges] of pairMap) {
    if (groupEdges.length < opts.minBundleSize) continue;

    const [sourceId, targetId] = key.split("::");
    const hasReverse = groupEdges.some((e: any) => e._isReverse === true);

    // Calculate spread offsets for each edge
    const spreadOffsets = new Map<string, number>();
    const halfWidth = ((groupEdges.length - 1) * opts.fanOutSpacing) / 2;

    groupEdges.forEach((edge, index) => {
      const offset = -halfWidth + index * opts.fanOutSpacing;
      spreadOffsets.set(edge.id, offset);
    });

    bundles.push({
      edgeIds: groupEdges.map((e) => e.id),
      sourceId,
      targetId,
      isBidirectional: hasReverse,
      spreadOffsets,
    });
  }

  return bundles;
}

/**
 * Apply fan-out offsets to bundled edge points.
 * Modifies the edge points to spread them apart at endpoints.
 */
export function applyBundleOffsets(
  edges: EdgeForBundling[],
  bundles: EdgeBundle[]
): Map<string, Point[]> {
  const result = new Map<string, Point[]>();
  const bundledEdgeIds = new Set(bundles.flatMap((b) => b.edgeIds));

  // Copy non-bundled edges as-is
  for (const edge of edges) {
    if (!bundledEdgeIds.has(edge.id)) {
      result.set(edge.id, [...edge.points]);
    }
  }

  // Apply offsets to bundled edges
  for (const bundle of bundles) {
    for (const edgeId of bundle.edgeIds) {
      const edge = edges.find((e) => e.id === edgeId);
      if (!edge || edge.points.length < 2) continue;

      const offset = bundle.spreadOffsets.get(edgeId) ?? 0;
      if (Math.abs(offset) < 0.01) {
        // Center edge, no modification needed
        result.set(edgeId, [...edge.points]);
        continue;
      }

      // Calculate perpendicular direction at start and end
      const start = edge.points[0];
      const second = edge.points[1];
      const secondLast = edge.points[edge.points.length - 2];
      const end = edge.points[edge.points.length - 1];

      const startPerp = perpendicularOffset(start, second, offset);
      const endPerp = perpendicularOffset(end, secondLast, offset);

      // Apply offsets
      const newPoints = [...edge.points];
      newPoints[0] = startPerp;
      newPoints[newPoints.length - 1] = endPerp;

      result.set(edgeId, newPoints);
    }
  }

  return result;
}

/**
 * Calculate a point offset perpendicular to the line direction.
 */
function perpendicularOffset(point: Point, toward: Point, offset: number): Point {
  const dx = toward.x - point.x;
  const dy = toward.y - point.y;
  const len = Math.hypot(dx, dy);

  if (len < 0.001) return point;

  // Perpendicular unit vector (rotated 90 degrees)
  const perpX = -dy / len;
  const perpY = dx / len;

  return {
    x: point.x + perpX * offset,
    y: point.y + perpY * offset,
  };
}

// ============================================================================
// CONVENIENCE FUNCTION
// ============================================================================

/**
 * Bundle edges and return updated points map.
 */
export function bundleEdges(
  edges: EdgeForBundling[],
  options: Partial<BundlingOptions> = {}
): {
  bundles: EdgeBundle[];
  adjustedPoints: Map<string, Point[]>;
} {
  const bundles = identifyBundles(edges, options);
  const adjustedPoints = applyBundleOffsets(edges, bundles);

  return { bundles, adjustedPoints };
}
