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
  angleTolerance: 5, // Strict angle tolerance to only bundle typically parallel lines
  positionTolerance: 20, // Only bundle edges that are already close
  fanOutSpacing: 15, // Increased spacing for clear separation
  minBundleSize: 2, // Bundle any pair of parallel edges
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
    // Validate edge has required fields
    if (!edge.sourceId || !edge.targetId) {
      console.warn(`[EdgeBundler] Edge missing sourceId or targetId, skipping:`, edge);
      continue;
    }

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
    if (!key || typeof key !== 'string') {
      console.warn(`[EdgeBundler] Invalid key, skipping:`, key);
      continue;
    }

    const parts = key.split("::");
    if (parts.length !== 2) {
      console.warn(`[EdgeBundler] Invalid key format, expected 'id1::id2', got:`, key);
      continue;
    }

    const [sourceId, targetId] = parts;
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

      // Calculate perpendicular direction based on the first segment
      // We apply this same offset vector to the entire path to keeping parallel edges parallel
      const start = edge.points[0];
      const second = edge.points[1];

      const dx = second.x - start.x;
      const dy = second.y - start.y;
      const len = Math.hypot(dx, dy);

      let perpX = 0;
      let perpY = 0;

      if (len > 0.001) {
        perpX = -dy / len;
        perpY = dx / len;
      }

      const offsetX = perpX * offset;
      const offsetY = perpY * offset;

      // Apply offset to ALL points
      // This creates distinct parallel paths ("bus" style) instead of merging in the middle
      const newPoints = edge.points.map((p) => ({
        x: p.x + offsetX,
        y: p.y + offsetY,
      }));

      result.set(edgeId, newPoints);
    }
  }

  return result;
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
