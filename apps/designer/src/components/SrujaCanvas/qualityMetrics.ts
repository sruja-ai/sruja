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
import type { Node as RFNode } from "@xyflow/react";

/**
 * Parent-child relationship mapping
 */
export interface ParentChildRelationships {
  /** Map of child node ID to parent node ID */
  childToParent: Map<string, string>;
}

/**
 * Detailed information about a parent-child containment violation
 */
export interface ParentChildContainmentViolation {
  childId: string;
  parentId: string;
}

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
  /** Number of child nodes that are not contained within their parent's bounding box */
  parentChildContainment: number;
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
  isGroup?: boolean;
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
 * Returns both the quality metrics and detailed violation information
 */
export interface QualityMeasurementResult {
  quality: LayoutQuality;
  parentChildContainmentViolations: ParentChildContainmentViolation[];
}

/**
 * Measure layout quality from React Flow nodes (with compound structure)
 * This version accounts for parent-child relationships in compound nodes
 */
export function measureQualityFromNodes(
  nodes: RFNode[],
  edges: GraphvizResult["edges"],
  parentChildRelationships?: ParentChildRelationships
): QualityMeasurementResult {
  // Build a map of all nodes for quick lookup
  const nodeMap = new Map(nodes.map((n) => [n.id, n]));

  // Convert React Flow nodes to absolute positions
  // For nodes with parentId, convert relative position to absolute
  const nodeBoxes: BoundingBox[] = nodes.map((node) => {
    let absoluteX = node.position.x;
    let absoluteY = node.position.y;

    // If node has a parent, add parent's position to get absolute position
    if (node.parentId) {
      const parent = nodeMap.get(node.parentId);
      if (parent) {
        absoluteX = parent.position.x + node.position.x;
        absoluteY = parent.position.y + node.position.y;
      }
    }

    const width = (node.width as number) || 200;
    const height = (node.height as number) || 120;

    return {
      id: node.id,
      x: absoluteX,
      y: absoluteY,
      width,
      height,
      centerX: absoluteX + width / 2,
      centerY: absoluteY + height / 2,
      isGroup: node.type === "group" || node.data?._isParent === true,
    };
  });

  // Build node map for quick lookup
  const boxMap = new Map(nodeBoxes.map((box) => [box.id, box]));

  // Count edge crossings
  const edgeCrossings = countEdgeCrossings(edges, nodeBoxes);

  // Count node overlaps
  const nodeOverlaps = countNodeOverlaps(nodeBoxes);

  // Count label overlaps (simplified - would need label positions from Graphviz)
  const labelOverlaps = estimateLabelOverlaps(edges, nodeBoxes);

  // Count parent-child containment violations and get detailed violations
  const containmentResult = parentChildRelationships
    ? findParentChildContainmentViolations(nodeBoxes, boxMap, parentChildRelationships)
    : { count: 0, violations: [] };
  const parentChildContainment = containmentResult.count;

  // Calculate edge length statistics
  const edgeLengths = calculateEdgeLengths(edges, nodeBoxes);
  const avgEdgeLength =
    edgeLengths.length > 0 ? edgeLengths.reduce((a, b) => a + b, 0) / edgeLengths.length : 0;
  const edgeLengthVariance = calculateVariance(edgeLengths, avgEdgeLength);

  // Measure rank alignment (how well nodes align horizontally/vertically)
  // Create a minimal GraphvizResult for compatibility
  const mockResult: GraphvizResult = {
    nodes: nodeBoxes.map((box) => ({
      id: box.id,
      x: box.x,
      y: box.y,
      width: box.width,
      height: box.height,
    })),
    edges,
    width: 0,
    height: 0,
  };
  const rankAlignment = measureRankAlignment(nodeBoxes, mockResult);

  // Measure cluster balance (placeholder - would need cluster info)
  const clusterBalance = 0.9; // Assume reasonable balance for now

  // Measure spacing consistency (requires uniform spacing)
  const spacingConsistency = measureSpacingConsistency(nodeBoxes);

  // Calculate overall score
  const score = calculateScore({
    edgeCrossings,
    nodeOverlaps,
    labelOverlaps,
    parentChildContainment,
    avgEdgeLength,
    edgeLengthVariance,
    rankAlignment,
    clusterBalance,
    spacingConsistency,
  });

  const quality: LayoutQuality = {
    edgeCrossings,
    nodeOverlaps,
    labelOverlaps,
    parentChildContainment,
    avgEdgeLength,
    edgeLengthVariance,
    rankAlignment,
    clusterBalance,
    spacingConsistency,
    score,
  };

  return {
    quality,
    parentChildContainmentViolations: containmentResult.violations,
  };
}

/**
 * Measure layout quality from Graphviz result
 */
export function measureQuality(
  result: GraphvizResult,
  parentChildRelationships?: ParentChildRelationships,
  _rawGraphvizJson?: any
): QualityMeasurementResult {
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

  // Build node map for quick lookup
  const nodeMap = new Map(nodeBoxes.map((box) => [box.id, box]));

  // Count edge crossings
  const edgeCrossings = countEdgeCrossings(edges, nodeBoxes);

  // Count node overlaps
  const nodeOverlaps = countNodeOverlaps(nodeBoxes);

  // Count label overlaps (simplified - would need label positions from Graphviz)
  const labelOverlaps = estimateLabelOverlaps(edges, nodeBoxes);

  // Count parent-child containment violations and get detailed violations
  const containmentResult = parentChildRelationships
    ? findParentChildContainmentViolations(nodeBoxes, nodeMap, parentChildRelationships)
    : { count: 0, violations: [] };
  const parentChildContainment = containmentResult.count;

  // Calculate edge length statistics
  const edgeLengths = calculateEdgeLengths(edges, nodeBoxes);
  const avgEdgeLength =
    edgeLengths.length > 0 ? edgeLengths.reduce((a, b) => a + b, 0) / edgeLengths.length : 0;
  const edgeLengthVariance = calculateVariance(edgeLengths, avgEdgeLength);

  // Measure rank alignment (how well nodes align horizontally/vertically)
  const rankAlignment = measureRankAlignment(nodeBoxes, result);

  // Measure cluster balance (placeholder - would need cluster info)
  const clusterBalance = 0.9; // Assume reasonable balance for now

  // Measure spacing consistency (requires uniform spacing)
  const spacingConsistency = measureSpacingConsistency(nodeBoxes);

  // Calculate overall score
  const score = calculateScore({
    edgeCrossings,
    nodeOverlaps,
    labelOverlaps,
    parentChildContainment,
    avgEdgeLength,
    edgeLengthVariance,
    rankAlignment,
    clusterBalance,
    spacingConsistency,
  });

  const quality: LayoutQuality = {
    edgeCrossings,
    nodeOverlaps,
    labelOverlaps,
    parentChildContainment,
    avgEdgeLength,
    edgeLengthVariance,
    rankAlignment,
    clusterBalance,
    spacingConsistency,
    score,
  };

  return {
    quality,
    parentChildContainmentViolations: containmentResult.violations,
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
      // Skip overlap check if either node is a group/cluster
      // Valid parent-child containment (child inside group) should not count as overlap
      if (nodeBoxes[i].isGroup || nodeBoxes[j].isGroup) continue;

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
 * Check if a child bounding box is completely contained within a parent bounding box
 * A child is contained if all its corners are inside the parent's bounds
 */
function isBoxContained(child: BoundingBox, parent: BoundingBox): boolean {
  const childLeft = child.x;
  const childRight = child.x + child.width;
  const childTop = child.y;
  const childBottom = child.y + child.height;

  const parentLeft = parent.x;
  const parentRight = parent.x + parent.width;
  const parentTop = parent.y;
  const parentBottom = parent.y + parent.height;

  // Child is contained if all its bounds are within parent's bounds
  // Using <= to allow edge-touching (child can be exactly at parent's edge)
  return (
    childLeft >= parentLeft &&
    childRight <= parentRight &&
    childTop >= parentTop &&
    childBottom <= parentBottom
  );
}

/**
 * Find parent-child containment violations
 * Checks if child nodes are properly contained within their parent's bounding box
 * Returns both the count and detailed violation information
 */
function findParentChildContainmentViolations(
  _nodeBoxes: BoundingBox[], // Not used directly, but nodeMap is built from it
  nodeMap: Map<string, BoundingBox>,
  relationships: ParentChildRelationships
): { count: number; violations: ParentChildContainmentViolation[] } {
  const violations: ParentChildContainmentViolation[] = [];

  // Iterate through all child-to-parent mappings
  for (const [childId, parentId] of relationships.childToParent.entries()) {
    const childBox = nodeMap.get(childId);
    const parentBox = nodeMap.get(parentId);

    // Skip if either node is not in the layout (might be filtered out by view level)
    if (!childBox || !parentBox) {
      continue;
    }

    // Check if child is contained within parent
    if (!isBoxContained(childBox, parentBox)) {
      violations.push({ childId, parentId });
    }
  }

  return {
    count: violations.length,
    violations,
  };
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
      // Skip group/cluster nodes (labels are often inside clusters)
      if (node.isGroup) continue;

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
 * Improved to account for node size variations and be more lenient for natural hierarchies
 */
function measureRankAlignment(nodeBoxes: BoundingBox[], _result: GraphvizResult): number {
  if (nodeBoxes.length < 2) return 1.0;

  // Group nodes by approximate rank (Y position for TB, X position for LR)
  // For simplicity, assume TB layout (rank by Y)
  // Uses reasonable tolerance that accounts for node size variations
  const tolerance = 15; // Pixels - accounts for different node sizes (person vs system)

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

    // Account for node size variations - larger nodes naturally have more spread
    const avgHeight = boxes.reduce((sum, b) => sum + b.height, 0) / boxes.length;
    const normalizedSpread = ySpread / avgHeight;

    // More lenient alignment that accounts for node sizes
    // We focus ONLY on vertical alignment, ignoring horizontal spread completely
    // Perfect alignment: < 10% of average node height
    let alignment: number;
    if (normalizedSpread <= 0.1) {
      alignment = 1.0; // Perfect alignment
    } else if (normalizedSpread <= 0.2) {
      alignment = 1.0 - ((normalizedSpread - 0.1) / 0.1) * 0.1; // Excellent
    } else if (normalizedSpread <= 0.4) {
      alignment = 0.9 - ((normalizedSpread - 0.2) / 0.2) * 0.2; // Good
    } else {
      alignment = Math.max(0, 0.7 - (normalizedSpread - 0.4) * 0.5); // Fair to poor
    }

    totalAlignment += alignment;
    rankCount++;
  }

  return rankCount > 0 ? totalAlignment / rankCount : 1.0;
}

/**
 * Measure spacing consistency
 * Requires uniform spacing between nodes for professional appearance
 *
 * Improvements:
 * 1. Accounts for node sizes by using edge-to-edge distances
 * 2. Measures within-rank spacing separately (horizontal consistency)
 * 3. Measures between-rank spacing separately (vertical consistency)
 * 4. Normalizes for node size variations
 * 5. Uses adaptive tolerance based on node size variability
 * 6. Better handling of small sample sizes
 */
function measureSpacingConsistency(nodeBoxes: BoundingBox[]): number {
  if (nodeBoxes.length < 3) return 1.0;

  // Group nodes by rank (Y position) for rank-based spacing analysis
  // Use adaptive tolerance based on average node height
  const avgHeight = nodeBoxes.reduce((sum, b) => sum + b.height, 0) / nodeBoxes.length;
  const tolerance = Math.max(10, avgHeight * 0.1); // Adaptive tolerance

  const rankGroups = new Map<number, BoundingBox[]>();
  for (const box of nodeBoxes) {
    const rankY = Math.round(box.centerY / tolerance) * tolerance;
    if (!rankGroups.has(rankY)) {
      rankGroups.set(rankY, []);
    }
    rankGroups.get(rankY)!.push(box);
  }

  // Calculate spacing consistency within ranks (horizontal spacing)
  const withinRankSpacings: number[] = [];
  const withinRankConsistencies: number[] = [];

  for (const boxes of rankGroups.values()) {
    if (boxes.length < 2) continue;

    // Sort by X position
    const sorted = [...boxes].sort((a, b) => a.centerX - b.centerX);

    // Calculate edge-to-edge horizontal spacing between adjacent nodes in same rank
    for (let i = 0; i < sorted.length - 1; i++) {
      const left = sorted[i];
      const right = sorted[i + 1];

      // Edge-to-edge distance: right edge of left node to left edge of right node
      const leftRightEdge = left.x + left.width;
      const rightLeftEdge = right.x;
      const spacing = rightLeftEdge - leftRightEdge;

      // Normalize by average node size to account for size variations
      const avgWidth = (left.width + right.width) / 2;
      const normalizedSpacing = spacing / avgWidth;

      if (normalizedSpacing > 0) {
        withinRankSpacings.push(normalizedSpacing);
      }
    }

    // Calculate per-rank consistency (how uniform is spacing within this rank)
    if (sorted.length >= 3) {
      const rankSpacings: number[] = [];
      for (let i = 0; i < sorted.length - 1; i++) {
        const left = sorted[i];
        const right = sorted[i + 1];
        const spacing = right.x - (left.x + left.width);
        const normalized = spacing / ((left.width + right.width) / 2);
        if (normalized > 0) rankSpacings.push(normalized);
      }
      if (rankSpacings.length > 1) {
        const rankMean = rankSpacings.reduce((a, b) => a + b, 0) / rankSpacings.length;
        const rankVar =
          rankSpacings.reduce((sum, d) => sum + (d - rankMean) ** 2, 0) / rankSpacings.length;
        const rankCV = rankMean > 0 ? Math.sqrt(rankVar) / rankMean : 0;
        withinRankConsistencies.push(Math.max(0, 1 - rankCV));
      }
    }
  }

  // Calculate spacing consistency between ranks (vertical spacing)
  const betweenRankSpacings: number[] = [];
  const sortedRanks = Array.from(rankGroups.keys()).sort((a, b) => a - b);

  for (let i = 0; i < sortedRanks.length - 1; i++) {
    const upperRank = rankGroups.get(sortedRanks[i])!;
    const lowerRank = rankGroups.get(sortedRanks[i + 1])!;

    // Calculate vertical spacing between ranks
    // Use bottom edge of upper rank to top edge of lower rank
    const upperBottom = Math.max(...upperRank.map((b) => b.y + b.height));
    const lowerTop = Math.min(...lowerRank.map((b) => b.y));
    const verticalSpacing = lowerTop - upperBottom;

    // Normalize by average node height
    const avgUpperHeight = upperRank.reduce((sum, b) => sum + b.height, 0) / upperRank.length;
    const avgLowerHeight = lowerRank.reduce((sum, b) => sum + b.height, 0) / lowerRank.length;
    const avgHeightPair = (avgUpperHeight + avgLowerHeight) / 2;
    const normalizedSpacing = verticalSpacing / avgHeightPair;

    if (normalizedSpacing > 0) {
      betweenRankSpacings.push(normalizedSpacing);
    }
  }

  // Combine both spacing measurements
  const allSpacings = [...withinRankSpacings, ...betweenRankSpacings];

  if (allSpacings.length === 0) return 1.0;

  // Calculate overall consistency score
  const mean = allSpacings.reduce((a, b) => a + b, 0) / allSpacings.length;
  const variance = allSpacings.reduce((sum, d) => sum + (d - mean) ** 2, 0) / allSpacings.length;
  const stdDev = Math.sqrt(variance);

  // Consistency score: lower stdDev relative to mean = better consistency
  // Use adaptive thresholds based on number of samples
  const sampleCount = allSpacings.length;
  const cvMultiplier = sampleCount < 10 ? 1.2 : sampleCount < 20 ? 1.1 : 1.0;

  const coefficientOfVariation = stdDev / mean;
  const adjustedCV = coefficientOfVariation * cvMultiplier;

  // More lenient thresholds for better scoring
  if (adjustedCV <= 0.3) {
    return 1.0; // Excellent consistency
  } else if (adjustedCV <= 0.5) {
    return 1.0 - ((adjustedCV - 0.3) / 0.2) * 0.15; // Good consistency
  } else if (adjustedCV <= 0.75) {
    return 0.85 - ((adjustedCV - 0.5) / 0.25) * 0.2; // Fair consistency
  } else {
    return Math.max(0, 0.65 - (adjustedCV - 0.75) * 0.4); // Poor consistency
  }
}

/**
 * Calculate overall quality score
 * Balanced scoring that rewards improvements while penalizing major issues
 *
 * Scoring philosophy:
 * - 0 overlaps = no penalty (critical)
 * - Few edge crossings (<10) = small penalty, allows for non-zero scores
 * - Moderate crossings (10-30) = medium penalty
 * - Many crossings (>30) = large penalty but still allows some score
 * - Spacing/alignment variations = minor penalties
 */
function calculateScore(metrics: Omit<LayoutQuality, "score">): number {
  let score = 1.0;

  // DEBUG: Verify new scoring code is being used
  console.log("[SCORE_DEBUG_V2] New scoring formula active - crossings:", metrics.edgeCrossings);

  // Penalize edge crossings with a sliding scale
  // 1-5 crossings: small penalty (0.02 each)
  // 6-15 crossings: medium penalty (0.03 each)
  // 16-30 crossings: larger penalty (0.04 each)
  // 30+ crossings: max penalty of 0.5
  if (metrics.edgeCrossings > 0) {
    let penalty = 0;
    const crossings = metrics.edgeCrossings;

    if (crossings <= 5) {
      penalty = crossings * 0.02; // Max 0.10 for 5 crossings
    } else if (crossings <= 15) {
      penalty = 0.1 + (crossings - 5) * 0.03; // Max 0.40 for 15 crossings
    } else if (crossings <= 30) {
      penalty = 0.4 + (crossings - 15) * 0.02; // Max 0.70 for 30 crossings
    } else {
      penalty = 0.7 + Math.min((crossings - 30) * 0.005, 0.15); // Max 0.85
    }

    score -= Math.min(penalty, 0.85);
  }

  // Penalize node overlaps heavily (each overlap reduces score by 0.25, max 0.8 reduction)
  // Node overlaps are critical - they make diagrams unreadable
  if (metrics.nodeOverlaps > 0) {
    const penalty = Math.min(metrics.nodeOverlaps * 0.25, 0.8);
    score -= penalty;
  }

  // Penalize label overlaps (each overlap reduces score by 0.10, max 0.3 reduction)
  if (metrics.labelOverlaps > 0) {
    const penalty = Math.min(metrics.labelOverlaps * 0.1, 0.3);
    score -= penalty;
  }

  // Penalize parent-child containment violations (each violation reduces score by 0.20, max 0.6 reduction)
  // Critical for hierarchy but slightly less harsh than before
  if (metrics.parentChildContainment > 0) {
    const penalty = Math.min(metrics.parentChildContainment * 0.2, 0.6);
    score -= penalty;
  }

  // Penalize poor rank alignment (reduces score by up to 0.15)
  // More lenient: 85%+ is acceptable, 75%+ gets partial credit
  const rankAlignment = metrics.rankAlignment ?? 0.9;
  if (rankAlignment < 0.85) {
    score -= (0.85 - rankAlignment) * 0.15;
  }

  // Penalize poor cluster balance (reduces score by up to 0.10)
  const clusterBalance = metrics.clusterBalance ?? 0.85;
  if (clusterBalance < 0.8) {
    score -= (0.8 - clusterBalance) * 0.1;
  }

  // Penalize poor spacing consistency (reduces score by up to 0.15)
  // More lenient: 75%+ is acceptable
  const spacingConsistency = metrics.spacingConsistency ?? 0.85;
  if (spacingConsistency < 0.75) {
    score -= (0.75 - spacingConsistency) * 0.15;
  }

  // Penalize high edge length variance (inconsistent spacing)
  // Only penalize extreme variance
  if (metrics.avgEdgeLength > 0) {
    const normalizedVariance = metrics.edgeLengthVariance / metrics.avgEdgeLength;
    if (normalizedVariance > 0.7) {
      score -= Math.min((normalizedVariance - 0.7) * 0.15, 0.2);
    }
  }

  // DEBUG: Log final score explicitly
  console.log(
    `[SCORE_V2] crossings=${metrics.edgeCrossings}, labelOverlaps=${metrics.labelOverlaps}, containment=${metrics.parentChildContainment}, rankAlign=${metrics.rankAlignment?.toFixed(2)}, spacing=${metrics.spacingConsistency?.toFixed(2)}, clusterBal=${metrics.clusterBalance?.toFixed(2)}, finalScore=${score.toFixed(4)}`
  );

  // Ensure score is in valid range
  return Math.max(0.0, Math.min(1.0, score));
}

/**
 * Check if layout quality needs refinement
 * More strict thresholds for high-quality aesthetics
 */
export function needsRefinement(quality: LayoutQuality): boolean {
  // Refine if:
  // 1. Score is below 0.85 (high-quality threshold)
  // 2. Any edge crossings (should be minimized)
  // 3. Any node overlaps (unacceptable)
  // 4. Any parent-child containment violations (critical for hierarchy)
  // 5. Rank alignment below 95% (requires near-perfect alignment)
  return (
    quality.score < 0.85 ||
    quality.edgeCrossings > 0 ||
    quality.nodeOverlaps > 0 ||
    quality.parentChildContainment > 0 ||
    quality.rankAlignment < 0.95
  );
}

/**
 * Get refinement suggestions based on quality metrics
 */
export function getRefinementSuggestions(quality: LayoutQuality): string[] {
  const suggestions: string[] = [];

  if (quality.parentChildContainment > 0) {
    suggestions.push(
      `Fix parent-child containment (${quality.parentChildContainment} child nodes outside parent bounds)`
    );
  }

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
