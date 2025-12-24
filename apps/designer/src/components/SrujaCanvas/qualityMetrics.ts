/**
 * Quality Metrics for Graphviz Layout Analysis
 *
 * This module analyzes Graphviz JSON output to measure layout quality
 * and provide feedback for iterative refinement.
 *
 * Architecture:
 * - Measures actual layout quality from Graphviz JSON (not heuristics)
 * - Provides quantitative metrics for refinement decisions
 * - Supports iterative improvement cycles
 */

import type { GraphvizResult } from "./types";

/**
 * Layout quality metrics
 */
export interface LayoutQuality {
  /** Number of edge crossings (lower is better) */
  edgeCrossings: number;
  /** Number of node overlaps (should be 0) */
  nodeOverlaps: number;
  /** Number of label overlaps with nodes (should be 0) */
  labelOverlaps: number;
  /** Average edge length */
  avgEdgeLength: number;
  /** Variance in edge lengths (lower is better for consistency) */
  edgeLengthVariance: number;
  /** Rank alignment score (0.0-1.0, higher is better) */
  rankAlignment: number;
  /** Cluster balance score (0.0-1.0, higher is better) */
  clusterBalance: number;
  /** Spacing consistency score (0.0-1.0, higher is better) - measures node spacing uniformity */
  spacingConsistency: number;
  /** Overall quality score (0.0-1.0, higher is better) */
  score: number;
}

/**
 * Node bounding box for collision detection
 */
interface BoundingBox {
  id: string;
  x: number;
  y: number;
  width: number;
  height: number;
  centerX: number;
  centerY: number;
}

/**
 * Edge segment for crossing detection
 */
interface EdgeSegment {
  source: string;
  target: string;
  points: Array<[number, number]>;
}

/**
 * Measure layout quality from Graphviz result
 */
export function measureQuality(result: GraphvizResult, _rawGraphvizJson?: any): LayoutQuality {
  const nodes = result.nodes;
  const edges = result.edges;

  // Build bounding boxes for nodes
  const nodeBoxes = nodes.map((node) => ({
    id: node.id,
    x: node.x,
    y: node.y,
    width: node.width,
    height: node.height,
    centerX: node.x + node.width / 2,
    centerY: node.y + node.height / 2,
  }));

  // Count edge crossings
  const edgeCrossings = countEdgeCrossings(edges, nodeBoxes);

  // Count node overlaps
  const nodeOverlaps = countNodeOverlaps(nodeBoxes);

  // Count label overlaps (simplified - would need label positions from Graphviz)
  const labelOverlaps = estimateLabelOverlaps(edges, nodeBoxes);

  // Calculate edge length statistics
  const edgeLengths = calculateEdgeLengths(edges, nodeBoxes);
  const avgEdgeLength =
    edgeLengths.length > 0 ? edgeLengths.reduce((a, b) => a + b, 0) / edgeLengths.length : 0;
  const edgeLengthVariance = calculateVariance(edgeLengths, avgEdgeLength);

  // Measure rank alignment (how well nodes align horizontally/vertically)
  const rankAlignment = measureRankAlignment(nodeBoxes, result);

  // Measure cluster balance (placeholder - would need cluster info)
  const clusterBalance = 0.9; // Assume reasonable balance for now

  // Measure spacing consistency (LikeC4 requires uniform spacing)
  const spacingConsistency = measureSpacingConsistency(nodeBoxes);

  // Calculate overall score
  const score = calculateScore({
    edgeCrossings,
    nodeOverlaps,
    labelOverlaps,
    avgEdgeLength,
    edgeLengthVariance,
    rankAlignment,
    clusterBalance,
    spacingConsistency,
  });

  return {
    edgeCrossings,
    nodeOverlaps,
    labelOverlaps,
    avgEdgeLength,
    edgeLengthVariance,
    rankAlignment,
    clusterBalance,
    spacingConsistency,
    score,
  };
}

/**
 * Count edge crossings using line segment intersection
 */
function countEdgeCrossings(edges: GraphvizResult["edges"], nodeBoxes: BoundingBox[]): number {
  if (edges.length < 2) return 0;

  const nodeMap = new Map(nodeBoxes.map((b) => [b.id, b]));
  const segments: EdgeSegment[] = [];

  // Build edge segments
  for (const edge of edges) {
    const sourceBox = nodeMap.get(edge.source);
    const targetBox = nodeMap.get(edge.target);

    if (!sourceBox || !targetBox) continue;

    // If edge has spline points, use them; otherwise use straight line
    if (edge.points && edge.points.length > 0) {
      segments.push({
        source: edge.source,
        target: edge.target,
        points: edge.points,
      });
    } else {
      // Straight line from center to center
      segments.push({
        source: edge.source,
        target: edge.target,
        points: [
          [sourceBox.centerX, sourceBox.centerY],
          [targetBox.centerX, targetBox.centerY],
        ],
      });
    }
  }

  // Check all pairs of edges for crossings
  let crossings = 0;
  for (let i = 0; i < segments.length; i++) {
    for (let j = i + 1; j < segments.length; j++) {
      if (segmentsShareNode(segments[i], segments[j])) {
        continue; // Edges sharing a node don't count as crossings
      }

      if (doEdgesCross(segments[i], segments[j])) {
        crossings++;
      }
    }
  }

  return crossings;
}

/**
 * Check if two edges share a common node
 */
function segmentsShareNode(seg1: EdgeSegment, seg2: EdgeSegment): boolean {
  return (
    seg1.source === seg2.source ||
    seg1.source === seg2.target ||
    seg1.target === seg2.source ||
    seg1.target === seg2.target
  );
}

/**
 * Check if two edge segments cross
 * Uses line segment intersection algorithm
 */
function doEdgesCross(seg1: EdgeSegment, seg2: EdgeSegment): boolean {
  // For bezier splines, approximate with line segments
  // For simplicity, check if the main path crosses
  const points1 = seg1.points;
  const points2 = seg2.points;

  // Check each segment of edge1 against each segment of edge2
  for (let i = 0; i < points1.length - 1; i++) {
    for (let j = 0; j < points2.length - 1; j++) {
      if (lineSegmentsIntersect(points1[i], points1[i + 1], points2[j], points2[j + 1])) {
        return true;
      }
    }
  }

  return false;
}

/**
 * Check if two line segments intersect
 * Uses cross product method
 */
function lineSegmentsIntersect(
  p1: [number, number],
  p2: [number, number],
  p3: [number, number],
  p4: [number, number]
): boolean {
  const [x1, y1] = p1;
  const [x2, y2] = p2;
  const [x3, y3] = p3;
  const [x4, y4] = p4;

  // Calculate cross products
  const d1 = crossProduct(x3 - x1, y3 - y1, x2 - x1, y2 - y1);
  const d2 = crossProduct(x4 - x1, y4 - y1, x2 - x1, y2 - y1);
  const d3 = crossProduct(x1 - x3, y1 - y3, x4 - x3, y4 - y3);
  const d4 = crossProduct(x2 - x3, y2 - y3, x4 - x3, y4 - y3);

  // Check if segments straddle each other
  return ((d1 > 0 && d2 < 0) || (d1 < 0 && d2 > 0)) && ((d3 > 0 && d4 < 0) || (d3 < 0 && d4 > 0));
}

/**
 * Cross product helper
 */
function crossProduct(x1: number, y1: number, x2: number, y2: number): number {
  return x1 * y2 - x2 * y1;
}

/**
 * Count node overlaps
 */
function countNodeOverlaps(nodeBoxes: BoundingBox[]): number {
  let overlaps = 0;

  for (let i = 0; i < nodeBoxes.length; i++) {
    for (let j = i + 1; j < nodeBoxes.length; j++) {
      if (boxesOverlap(nodeBoxes[i], nodeBoxes[j])) {
        overlaps++;
      }
    }
  }

  return overlaps;
}

/**
 * Check if two bounding boxes overlap
 */
function boxesOverlap(box1: BoundingBox, box2: BoundingBox): boolean {
  return !(
    box1.x + box1.width < box2.x ||
    box2.x + box2.width < box1.x ||
    box1.y + box1.height < box2.y ||
    box2.y + box2.height < box1.y
  );
}

/**
 * Estimate label overlaps using parsed label positions
 */
function estimateLabelOverlaps(edges: GraphvizResult["edges"], nodeBoxes: BoundingBox[]): number {
  let overlaps = 0;
  const labelBoxes: BoundingBox[] = [];

  // 1. Build bounding boxes for labels that have positions
  // We estimate size since Graphviz JSON doesn't always include label dimensions explicitly in the edge object
  // (though _ldraw_ might have it, we keep it simple for now)
  const ESTIMATED_WIDTH = 100;
  const ESTIMATED_HEIGHT = 24;

  for (const edge of edges) {
    if (edge.labelPos) {
      labelBoxes.push({
        id: `label-${edge.id}`,
        x: edge.labelPos.x - ESTIMATED_WIDTH / 2,
        y: edge.labelPos.y - ESTIMATED_HEIGHT / 2,
        width: ESTIMATED_WIDTH,
        height: ESTIMATED_HEIGHT,
        centerX: edge.labelPos.x,
        centerY: edge.labelPos.y,
      });
    }
  }

  // 2. Check overlap with nodes
  for (const label of labelBoxes) {
    let isOverlapping = false;
    for (const node of nodeBoxes) {
      if (boxesOverlap(label, node)) {
        isOverlapping = true;
        break; // One overlap per label is enough to count
      }
    }

    // 3. Check overlap with other labels
    if (!isOverlapping) {
      for (const otherLabel of labelBoxes) {
        if (label.id !== otherLabel.id && boxesOverlap(label, otherLabel)) {
          isOverlapping = true;
          break;
        }
      }
    }

    if (isOverlapping) {
      overlaps++;
    }
  }

  return overlaps;
}

/**
 * Calculate edge lengths
 */
function calculateEdgeLengths(edges: GraphvizResult["edges"], nodeBoxes: BoundingBox[]): number[] {
  const nodeMap = new Map(nodeBoxes.map((b) => [b.id, b]));
  const lengths: number[] = [];

  for (const edge of edges) {
    const sourceBox = nodeMap.get(edge.source);
    const targetBox = nodeMap.get(edge.target);

    if (!sourceBox || !targetBox) continue;

    let length = 0;

    if (edge.points && edge.points.length > 1) {
      // Calculate length along spline
      for (let i = 0; i < edge.points.length - 1; i++) {
        const [x1, y1] = edge.points[i];
        const [x2, y2] = edge.points[i + 1];
        length += Math.sqrt((x2 - x1) ** 2 + (y2 - y1) ** 2);
      }
    } else {
      // Straight line distance
      length = Math.sqrt(
        (targetBox.centerX - sourceBox.centerX) ** 2 + (targetBox.centerY - sourceBox.centerY) ** 2
      );
    }

    lengths.push(length);
  }

  return lengths;
}

/**
 * Calculate variance
 */
function calculateVariance(values: number[], mean: number): number {
  if (values.length === 0) return 0;

  const squaredDiffs = values.map((v) => (v - mean) ** 2);
  return squaredDiffs.reduce((a, b) => a + b, 0) / values.length;
}

/**
 * Measure rank alignment
 * Checks how well nodes align within ranks (horizontal or vertical)
 * LikeC4 requires very strict alignment - tolerance reduced for better aesthetics
 */
function measureRankAlignment(nodeBoxes: BoundingBox[], _result: GraphvizResult): number {
  if (nodeBoxes.length < 2) return 1.0;

  // Group nodes by approximate rank (Y position for TB, X position for LR)
  // For simplicity, assume TB layout (rank by Y)
  // LikeC4 uses stricter tolerance (10px instead of 20px)
  const tolerance = 10; // Pixels - stricter for LikeC4-level quality

  // Group nodes by Y position (ranks)
  const rankGroups = new Map<number, BoundingBox[]>();
  for (const box of nodeBoxes) {
    const rankY = Math.round(box.centerY / tolerance) * tolerance;
    if (!rankGroups.has(rankY)) {
      rankGroups.set(rankY, []);
    }
    rankGroups.get(rankY)!.push(box);
  }

  // Calculate alignment score for each rank
  let totalAlignment = 0;
  let rankCount = 0;

  for (const boxes of rankGroups.values()) {
    if (boxes.length < 2) {
      totalAlignment += 1.0; // Single node ranks are perfectly aligned
      rankCount++;
      continue;
    }

    // Check how well nodes align (should have similar Y positions)
    const yPositions = boxes.map((b) => b.centerY);
    const minY = Math.min(...yPositions);
    const maxY = Math.max(...yPositions);
    const ySpread = maxY - minY;

    // Stricter alignment: LikeC4 requires <5px spread for perfect alignment
    // Alignment score: 1.0 if spread < 5px, decreases linearly
    const perfectThreshold = 5;
    const maxAcceptableSpread = tolerance * 2;
    let alignment: number;
    if (ySpread <= perfectThreshold) {
      alignment = 1.0;
    } else if (ySpread <= maxAcceptableSpread) {
      alignment =
        1.0 - ((ySpread - perfectThreshold) / (maxAcceptableSpread - perfectThreshold)) * 0.3;
    } else {
      alignment = Math.max(0, 0.7 - (ySpread - maxAcceptableSpread) / tolerance);
    }

    totalAlignment += alignment;
    rankCount++;
  }

  return rankCount > 0 ? totalAlignment / rankCount : 1.0;
}

/**
 * Measure spacing consistency
 * LikeC4 requires uniform spacing between nodes for professional appearance
 */
function measureSpacingConsistency(nodeBoxes: BoundingBox[]): number {
  if (nodeBoxes.length < 3) return 1.0;

  // Calculate distances between all pairs of nodes
  const distances: number[] = [];
  for (let i = 0; i < nodeBoxes.length; i++) {
    for (let j = i + 1; j < nodeBoxes.length; j++) {
      const dx = nodeBoxes[i].centerX - nodeBoxes[j].centerX;
      const dy = nodeBoxes[i].centerY - nodeBoxes[j].centerY;
      const distance = Math.sqrt(dx * dx + dy * dy);
      distances.push(distance);
    }
  }

  if (distances.length === 0) return 1.0;

  // Calculate mean and standard deviation
  const mean = distances.reduce((a, b) => a + b, 0) / distances.length;
  const variance = distances.reduce((sum, d) => sum + (d - mean) ** 2, 0) / distances.length;
  const stdDev = Math.sqrt(variance);

  // Consistency score: lower stdDev relative to mean = better consistency
  // LikeC4 requires stdDev < 30% of mean for good consistency
  const coefficientOfVariation = stdDev / mean;
  if (coefficientOfVariation <= 0.3) {
    return 1.0;
  } else if (coefficientOfVariation <= 0.5) {
    return 1.0 - ((coefficientOfVariation - 0.3) / 0.2) * 0.2;
  } else {
    return Math.max(0, 0.8 - (coefficientOfVariation - 0.5) * 0.4);
  }
}

/**
 * Calculate overall quality score
 * More strict scoring for LikeC4-level aesthetics
 */
function calculateScore(metrics: Omit<LayoutQuality, "score">): number {
  let score = 1.0;

  // Penalize edge crossings (each crossing reduces score by 0.08, max 0.6 reduction)
  if (metrics.edgeCrossings > 0) {
    const penalty = Math.min(metrics.edgeCrossings * 0.08, 0.6);
    score -= penalty;
  }

  // Penalize node overlaps heavily (each overlap reduces score by 0.3, max 0.8 reduction)
  if (metrics.nodeOverlaps > 0) {
    const penalty = Math.min(metrics.nodeOverlaps * 0.3, 0.8);
    score -= penalty;
  }

  // Penalize label overlaps (each overlap reduces score by 0.15, max 0.4 reduction)
  if (metrics.labelOverlaps > 0) {
    const penalty = Math.min(metrics.labelOverlaps * 0.15, 0.4);
    score -= penalty;
  }

  // Penalize poor rank alignment more strictly (reduces score by up to 0.3)
  // LikeC4 requires near-perfect alignment (95%+)
  if (metrics.rankAlignment < 0.95) {
    score -= (0.95 - metrics.rankAlignment) * 0.3;
  }

  // Penalize poor cluster balance (reduces score by up to 0.15)
  if (metrics.clusterBalance < 0.85) {
    score -= (0.85 - metrics.clusterBalance) * 0.15;
  }

  // Penalize poor spacing consistency (reduces score by up to 0.3)
  // LikeC4 requires uniform spacing for professional appearance
  if (metrics.spacingConsistency < 0.9) {
    score -= (0.9 - metrics.spacingConsistency) * 0.3;
  }

  // Penalize high edge length variance (inconsistent spacing looks unprofessional)
  // Normalize variance: if variance > 50% of mean, penalize
  if (metrics.avgEdgeLength > 0) {
    const normalizedVariance = metrics.edgeLengthVariance / metrics.avgEdgeLength;
    if (normalizedVariance > 0.5) {
      score -= Math.min((normalizedVariance - 0.5) * 0.2, 0.3);
    }
  }

  // Ensure score is in valid range
  return Math.max(0.0, Math.min(1.0, score));
}

/**
 * Check if layout quality needs refinement
 * More strict thresholds for LikeC4-level aesthetics
 */
export function needsRefinement(quality: LayoutQuality): boolean {
  // Refine if:
  // 1. Score is below 0.85 (LikeC4-level quality threshold)
  // 2. Any edge crossings (LikeC4 minimizes crossings)
  // 3. Any node overlaps (unacceptable)
  // 4. Rank alignment below 95% (LikeC4 requires near-perfect alignment)
  return (
    quality.score < 0.85 ||
    quality.edgeCrossings > 0 ||
    quality.nodeOverlaps > 0 ||
    quality.rankAlignment < 0.95
  );
}

/**
 * Get refinement suggestions based on quality metrics
 */
export function getRefinementSuggestions(quality: LayoutQuality): string[] {
  const suggestions: string[] = [];

  if (quality.nodeOverlaps > 0) {
    suggestions.push(`Increase node spacing (${quality.nodeOverlaps} overlaps detected)`);
  }

  if (quality.edgeCrossings > 5) {
    suggestions.push(`Optimize edge routing (${quality.edgeCrossings} crossings detected)`);
  }

  if (quality.rankAlignment < 0.9) {
    suggestions.push(
      `Improve rank alignment (current: ${(quality.rankAlignment * 100).toFixed(0)}%)`
    );
  }

  if (quality.labelOverlaps > 0) {
    suggestions.push(`Adjust label placement (${quality.labelOverlaps} overlaps detected)`);
  }

  return suggestions;
}
