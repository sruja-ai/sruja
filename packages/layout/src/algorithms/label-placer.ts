/**
 * Label Placement Algorithm
 *
 * Ensures edge labels don't overlap with each other or with nodes.
 * Uses spatial indexing for efficient collision detection.
 */

import type { Point } from "../geometry/point";
import type { Rect } from "../geometry/rect";

// ============================================================================
// TYPES
// ============================================================================

export interface LabelPlacement {
  edgeId: string;
  position: Point;
  rotation: number; // Angle to rotate label (follows edge direction)
  anchor: "start" | "middle" | "end";
  bounds: Rect; // Computed bounding box of label
}

export interface LabelOptions {
  defaultWidth: number; // Default label width estimate (chars * approx width)
  defaultHeight: number; // Default label height
  padding: number; // Min distance between labels
  maxShiftIterations: number;
  allowRotation: boolean;
}

export const DEFAULT_LABEL_OPTIONS: LabelOptions = {
  defaultWidth: 80,
  defaultHeight: 20,
  padding: 20, // Increased from 16 for better clearance
  maxShiftIterations: 15, // Increased from 10 to try more positions
  allowRotation: false,
};

export interface EdgeForLabeling {
  id: string;
  label?: string;
  points: readonly Point[];
  labelPosition?: Point;
}

export interface NodeForLabeling {
  id: string;
  bbox: Rect;
}

// ============================================================================
// GEOMETRY HELPERS
// ============================================================================

function rectOverlaps(a: Rect, b: Rect, padding: number = 0): boolean {
  return !(
    a.x + a.width + padding < b.x ||
    b.x + b.width + padding < a.x ||
    a.y + a.height + padding < b.y ||
    b.y + b.height + padding < a.y
  );
}

function pointAlongPath(points: readonly Point[], t: number): Point {
  if (points.length === 0) return { x: 0, y: 0 };
  if (points.length === 1) return { ...points[0] };
  if (t <= 0) return { ...points[0] };
  if (t >= 1) return { ...points[points.length - 1] };

  // Calculate total path length
  let totalLength = 0;
  for (let i = 1; i < points.length; i++) {
    totalLength += Math.hypot(points[i].x - points[i - 1].x, points[i].y - points[i - 1].y);
  }

  const targetDist = t * totalLength;
  let accum = 0;

  for (let i = 1; i < points.length; i++) {
    const segLen = Math.hypot(points[i].x - points[i - 1].x, points[i].y - points[i - 1].y);
    if (accum + segLen >= targetDist) {
      const segT = (targetDist - accum) / segLen;
      return {
        x: points[i - 1].x + (points[i].x - points[i - 1].x) * segT,
        y: points[i - 1].y + (points[i].y - points[i - 1].y) * segT,
      };
    }
    accum += segLen;
  }

  return { ...points[points.length - 1] };
}

function angleAtPoint(points: readonly Point[], t: number): number {
  if (points.length < 2) return 0;

  // Find segment at position t
  let totalLength = 0;
  for (let i = 1; i < points.length; i++) {
    totalLength += Math.hypot(points[i].x - points[i - 1].x, points[i].y - points[i - 1].y);
  }

  const targetDist = t * totalLength;
  let accum = 0;

  for (let i = 1; i < points.length; i++) {
    const segLen = Math.hypot(points[i].x - points[i - 1].x, points[i].y - points[i - 1].y);
    if (accum + segLen >= targetDist || i === points.length - 1) {
      const dx = points[i].x - points[i - 1].x;
      const dy = points[i].y - points[i - 1].y;
      return Math.atan2(dy, dx) * (180 / Math.PI);
    }
    accum += segLen;
  }

  return 0;
}

function estimateLabelWidth(label: string | undefined, defaultWidth: number): number {
  if (!label) return 0;
  // Rough estimate: 7px per character
  return Math.max(defaultWidth, label.length * 7);
}

// ============================================================================
// COLLISION DETECTION
// ============================================================================

export interface CollisionResult {
  hasCollision: boolean;
  collidingWith: string[]; // IDs of colliding elements
}

export function detectLabelCollisions(
  placements: Map<string, LabelPlacement>,
  nodes: Map<string, NodeForLabeling>,
  padding: number
): Map<string, CollisionResult> {
  const results = new Map<string, CollisionResult>();
  const allPlacements = [...placements.entries()];

  for (const [id, placement] of allPlacements) {
    const collisions: string[] = [];

    // Check against other labels
    for (const [otherId, otherPlacement] of allPlacements) {
      if (id === otherId) continue;
      if (rectOverlaps(placement.bounds, otherPlacement.bounds, padding)) {
        collisions.push(`label:${otherId}`);
      }
    }

    // Check against nodes
    for (const [, node] of nodes) {
      if (rectOverlaps(placement.bounds, node.bbox, padding)) {
        collisions.push(`node:${node.id}`);
      }
    }

    results.set(id, {
      hasCollision: collisions.length > 0,
      collidingWith: collisions,
    });
  }

  return results;
}

// ============================================================================
// LABEL PLACEMENT ALGORITHM
// ============================================================================

/**
 * Place labels on edges avoiding collisions with nodes and other labels.
 *
 * Algorithm:
 * 1. Initial placement at edge midpoint (t=0.5)
 * 2. Detect collisions
 * 3. For colliding labels, try alternate positions (t=0.25, t=0.75, etc.)
 * 4. Repeat until no collisions or max iterations reached
 */
export function placeLabels(
  edges: EdgeForLabeling[],
  nodes: Map<string, NodeForLabeling>,
  options: Partial<LabelOptions> = {}
): Map<string, LabelPlacement> {
  const opts = { ...DEFAULT_LABEL_OPTIONS, ...options };
  const placements = new Map<string, LabelPlacement>();

  // Skip edges without labels
  const labeledEdges = edges.filter((e) => e.label && e.label.length > 0);

  // Step 1: Initial placement at midpoint
  for (const edge of labeledEdges) {
    const position = edge.labelPosition ?? pointAlongPath(edge.points, 0.5);
    const width = estimateLabelWidth(edge.label, opts.defaultWidth);

    placements.set(edge.id, {
      edgeId: edge.id,
      position,
      rotation: 0,
      anchor: "middle",
      bounds: {
        x: position.x - width / 2,
        y: position.y - opts.defaultHeight / 2,
        width,
        height: opts.defaultHeight,
      },
    });
  }

  // Step 2-4: Iteratively resolve collisions
  // More candidate positions for better label placement, especially for dense graphs
  const candidatePositions = [
    0.5, 0.35, 0.65, 0.25, 0.75, 0.2, 0.8, 0.15, 0.85, 0.3, 0.7, 0.4, 0.6,
  ];

  // Track which positions each label has tried to avoid infinite loops
  const triedPositions = new Map<string, Set<number>>();
  labeledEdges.forEach((e) => triedPositions.set(e.id, new Set()));

  for (let iteration = 0; iteration < opts.maxShiftIterations; iteration++) {
    const collisions = detectLabelCollisions(placements, nodes, opts.padding);

    let anyCollision = false;
    for (const [id, result] of collisions) {
      if (result.hasCollision) {
        anyCollision = true;

        // Try next candidate position
        const edge = labeledEdges.find((e) => e.id === id);
        if (!edge) continue;

        const currentPlacement = placements.get(id)!;
        const currentT = findCurrentT(edge.points, currentPlacement.position);
        const tried = triedPositions.get(id)!;

        // Find best untried position that minimizes collisions
        let bestT = currentT;
        let bestCollisions = result.collidingWith.length;

        // Try all candidate positions and pick the one with fewest collisions
        for (const candidateT of candidatePositions) {
          if (tried.has(candidateT)) continue;

          const candidatePos = pointAlongPath(edge.points, candidateT);
          const width = estimateLabelWidth(edge.label, opts.defaultWidth);
          const candidateBounds = {
            x: candidatePos.x - width / 2,
            y: candidatePos.y - opts.defaultHeight / 2,
            width,
            height: opts.defaultHeight,
          };

          // Count collisions for this candidate position
          let candidateCollisions = 0;
          for (const [otherId, otherPlacement] of placements) {
            if (otherId === id) continue;
            if (rectOverlaps(candidateBounds, otherPlacement.bounds, opts.padding)) {
              candidateCollisions++;
            }
          }
          for (const [, node] of nodes) {
            if (rectOverlaps(candidateBounds, node.bbox, opts.padding)) {
              candidateCollisions++;
            }
          }

          if (candidateCollisions < bestCollisions) {
            bestCollisions = candidateCollisions;
            bestT = candidateT;
          }
        }

        // If we found a better position, use it
        if (bestT !== currentT || iteration === 0) {
          tried.add(bestT);
          const newPosition = pointAlongPath(edge.points, bestT);
          const width = estimateLabelWidth(edge.label, opts.defaultWidth);

          placements.set(id, {
            edgeId: id,
            position: newPosition,
            rotation: opts.allowRotation ? angleAtPoint(edge.points, bestT) : 0,
            anchor: bestT < 0.4 ? "start" : bestT > 0.6 ? "end" : "middle",
            bounds: {
              x: newPosition.x - width / 2,
              y: newPosition.y - opts.defaultHeight / 2,
              width,
              height: opts.defaultHeight,
            },
          });
        } else {
          // If no better position found, try offsetting perpendicular to edge
          if (edge.points.length >= 2) {
            const segmentIdx = Math.floor(bestT * (edge.points.length - 1));
            const p1 = edge.points[Math.min(segmentIdx, edge.points.length - 2)];
            const p2 = edge.points[Math.min(segmentIdx + 1, edge.points.length - 1)];
            const dx = p2.x - p1.x;
            const dy = p2.y - p1.y;
            const len = Math.hypot(dx, dy);
            if (len > 0) {
              // Perpendicular offset (normalized)
              const perpX = -dy / len;
              const perpY = dx / len;
              const offset = ((iteration % 3) + 1) * 25; // Try offsets: 25, 50, 75

              const offsetPosition = {
                x: currentPlacement.position.x + perpX * offset,
                y: currentPlacement.position.y + perpY * offset,
              };
              const width = estimateLabelWidth(edge.label, opts.defaultWidth);

              placements.set(id, {
                edgeId: id,
                position: offsetPosition,
                rotation: currentPlacement.rotation,
                anchor: currentPlacement.anchor,
                bounds: {
                  x: offsetPosition.x - width / 2,
                  y: offsetPosition.y - opts.defaultHeight / 2,
                  width,
                  height: opts.defaultHeight,
                },
              });
            }
          }
        }
      }
    }

    if (!anyCollision) break;
  }

  return placements;
}

function findCurrentT(points: readonly Point[], position: Point): number {
  if (points.length < 2) return 0.5;

  // Calculate total path length and find closest point
  let totalLength = 0;
  const segments: number[] = [];

  for (let i = 1; i < points.length; i++) {
    const len = Math.hypot(points[i].x - points[i - 1].x, points[i].y - points[i - 1].y);
    segments.push(len);
    totalLength += len;
  }

  // Find which segment the position is closest to
  let minDist = Infinity;
  let closestT = 0.5;
  let accum = 0;

  for (let i = 0; i < segments.length; i++) {
    const midT = (accum + segments[i] / 2) / totalLength;
    const midPoint = pointAlongPath(points, midT);
    const dist = Math.hypot(position.x - midPoint.x, position.y - midPoint.y);

    if (dist < minDist) {
      minDist = dist;
      closestT = midT;
    }
    accum += segments[i];
  }

  return closestT;
}
