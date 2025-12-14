// Diagram quality metrics and validation for "well-structured" diagrams
import type { Node, Edge } from "@xyflow/react";
import type { C4NodeData } from "../types";

export interface DiagramQualityMetrics {
  // Overlap detection
  overlappingNodes: OverlapViolation[];
  overlapScore: number; // 0-100, higher is better

  // Spacing
  minSpacing: number;
  averageSpacing: number;
  spacingViolations: SpacingViolation[];
  spacingScore: number;

  // Edge quality
  edgeCrossings: number;
  edgeOverlaps: number;
  edgesOverNodes: number; // Edges passing over/through nodes
  edgeBends: number; // Total number of bends in edges
  edgeLength: { min: number; max: number; average: number };
  edgeScore: number;

  // Hierarchy clarity
  parentChildContainment: ContainmentViolation[];
  hierarchyScore: number;

  // Viewport utilization
  viewportUtilization: number; // 0-1, how well diagram uses viewport
  viewportScore: number;

  // Consistency
  consistencyScore: number; // How consistent similar nodes are positioned

  // Aspect ratio and distortion
  aspectRatio: number; // width/height of diagram
  aspectRatioScore: number; // Penalty for extreme aspect ratios
  parentChildSizeViolations: ParentChildSizeViolation[]; // Parents too small for children

  // Overall score
  overallScore: number;
  weightedScore: number; // Score with custom weights
  grade: "A" | "B" | "C" | "D" | "F";

  // New metrics
  directionViolations: DirectionViolation[];
  directionScore: number;
  // Empty space metric
  emptySpace: number; // Percentage of bounding box that is empty
  emptySpaceScore: number;
  nodeBadness: Record<string, number>;

  // Label quality metrics
  edgeLabelOverlaps: number; // Edge labels overlapping with node labels/content
  edgeLabelOverlapScore: number;
  clippedNodeLabels: number; // Node labels cut off or not fully visible
  clippedLabelScore: number;
  edgeCongestionScore: number;
  crossingAngleScore: number;
  alignmentScore: number;
  detourScore: number;
}

export interface QualityWeights {
  overlap: number;
  spacing: number;
  edgeCrossings: number;
  edgesOverNodes: number;
  edgeBends: number;
  edgeLength: number;
  hierarchy: number;
  viewport: number;
  consistency: number;
  aspectRatio: number;
  direction: number;
  emptySpace: number;
  edgeLabelOverlaps: number;
  clippedLabels: number;
  edgeCongestion: number;
  crossingAngles: number;
  alignment: number;
  detour: number;
}

export interface OverlapViolation {
  node1: string;
  node2: string;
  overlapArea: number;
  overlapPercentage: number;
}

export interface SpacingViolation {
  node1: string;
  node2: string;
  distance: number;
  minRequired: number;
}

export interface ContainmentViolation {
  childId: string;
  parentId: string;
  violation: "outside" | "overlapping" | "too-close-to-edge";
  details: string;
}

export interface ParentChildSizeViolation {
  parentId: string;
  childId: string;
  parentWidth: number;
  parentHeight: number;
  childWidth: number;
  childHeight: number;
  requiredWidth: number;
  requiredHeight: number;
  violation: "width" | "height" | "both";
}

const MIN_NODE_SPACING = 30; // Minimum pixels between nodes (further increased for better quality)
const MIN_PARENT_PADDING = 80; // Minimum padding inside parent nodes (increased further to prevent any child overflow)

/**
 * Default weights for quality metrics
 * Higher weight = more important for overall score
 */
export const DEFAULT_QUALITY_WEIGHTS: QualityWeights = {
  overlap: 0.16, // Node overlaps are critical violations (reduced to make room)
  spacing: 0.07, // Good spacing is important but less critical (reduced)
  edgeCrossings: 0.16, // Edge crossings severely impact readability (reduced)
  edgesOverNodes: 0.1, // Edges over nodes are confusing (reduced)
  edgeBends: 0.04, // Bends are acceptable if they avoid crossings (reduced)
  edgeLength: 0.04, // Reasonable edge lengths (reduced)
  hierarchy: 0.15, // Parent-child containment is CRITICAL - increased weight
  viewport: 0.02, // Viewport utilization (less important, reduced)
  consistency: 0.02, // Consistency is nice-to-have
  aspectRatio: 0.03, // Aspect ratio weight (reduced)
  direction: 0.07, // Direction flow (reduced)
  emptySpace: 0.04, // Penalty for excessive empty space (reduced)
  edgeLabelOverlaps: 0.07,
  clippedLabels: 0.07,
  edgeCongestion: 0.05,
  crossingAngles: 0.06,
  alignment: 0.04,
  detour: 0.03,
};

/**
 * Per-level quality weights - different priorities for each C4 level
 * L0: Enterprise overview - focus on spacing and visual balance
 * L1: System context - focus on relationship clarity and direction flow
 * L2: Container view - focus on container organization and hierarchy
 * L3: Component view - focus on component flow and edge routing
 */
export const LEVEL_QUALITY_WEIGHTS: Record<string, QualityWeights> = {
  L0: {
    ...DEFAULT_QUALITY_WEIGHTS,
    spacing: 0.12, // Increased - proper spacing critical for overview
    hierarchy: 0.08, // Reduced - less hierarchy at L0
    aspectRatio: 0.1, // Increased - visual balance matters
    emptySpace: 0.08, // Increased - avoid wasted space
  },
  L1: {
    ...DEFAULT_QUALITY_WEIGHTS,
    direction: 0.12, // Increased - relationship direction is key
    edgeCrossings: 0.18, // Increased - crossing edges confuse relationships
    spacing: 0.1, // Increased - clear separation between systems
    hierarchy: 0.08, // Reduced - typically flat at L1
  },
  L2: {
    ...DEFAULT_QUALITY_WEIGHTS,
    hierarchy: 0.2, // Increased - containment is critical
    edgesOverNodes: 0.12, // Increased - edges passing through containers confuse
    clippedLabels: 0.1, // Increased - container labels must be readable
  },
  L3: {
    ...DEFAULT_QUALITY_WEIGHTS,
    direction: 0.14, // Increased - data/control flow matters
    edgeCrossings: 0.18, // Increased - crossings confuse component interactions
    alignment: 0.08, // Increased - aligned components look organized
    edgeCongestion: 0.08, // Increased - avoid edge bunching
  },
};

/**
 * Get quality weights for a specific C4 level
 */
export function getQualityWeightsForLevel(level?: string): QualityWeights {
  if (level && LEVEL_QUALITY_WEIGHTS[level]) {
    return LEVEL_QUALITY_WEIGHTS[level];
  }
  return DEFAULT_QUALITY_WEIGHTS;
}

// Optimal aspect ratio range (width/height)
const OPTIMAL_ASPECT_RATIO_MIN = 0.5; // Not too tall
const OPTIMAL_ASPECT_RATIO_MAX = 2.0; // Not too wide

/**
 * Detect edge labels overlapping with node labels/content
 * Edge labels should be positioned away from nodes to avoid visual confusion
 */
function detectEdgeLabelOverlaps(nodes: Node<C4NodeData>[], edges: Edge[]): number {
  let overlapCount = 0;
  const PADDING = 8; // Minimum padding around nodes for label clearance

  edges.forEach((edge) => {
    const sourceNode = nodes.find((n) => n.id === edge.source);
    const targetNode = nodes.find((n) => n.id === edge.target);
    if (!sourceNode || !targetNode) return;

    // Calculate edge label position (React Flow uses midpoint or layout position)
    let labelX: number;
    let labelY: number;

    const labelPosition = (edge.data as any)?.labelPosition;
    if (
      labelPosition &&
      typeof labelPosition.x === "number" &&
      typeof labelPosition.y === "number"
    ) {
      // Use explicit label position from layout (absolute coordinates)
      labelX = labelPosition.x;
      labelY = labelPosition.y;
    } else {
      // Calculate midpoint between source and target centers (React Flow default)
      const sourceCenterX = sourceNode.position.x + (sourceNode.width || 100) / 2;
      const sourceCenterY = sourceNode.position.y + (sourceNode.height || 100) / 2;
      const targetCenterX = targetNode.position.x + (targetNode.width || 100) / 2;
      const targetCenterY = targetNode.position.y + (targetNode.height || 100) / 2;

      labelX = (sourceCenterX + targetCenterX) / 2;
      labelY = (sourceCenterY + targetCenterY) / 2;
    }

    // Estimate label size (account for both label and technology tag if present)
    const edgeLabel = edge.label || (edge.data?.label as string) || "";
    const technology = (edge.data?.technology as string) || "";
    const hasTechnology = !!technology;

    // Edge labels can have both label and technology tag stacked
    const labelTextWidth = Math.max(
      String(edgeLabel).length * 7,
      technology.length * 6 // Technology uses smaller font
    );
    const labelWidth = labelTextWidth + 16; // padding
    const labelHeight = hasTechnology ? 44 : 24; // Stacked: ~22px each + gap

    // Check ALL nodes - edge labels should never be inside any node
    let edgeHasOverlap = false;
    for (const node of nodes) {
      // Account for parent positioning (absolute coordinates)
      let nodeLeft = node.position.x;
      let nodeTop = node.position.y;

      // If node has parent, coordinates are relative, need to calculate absolute
      if (node.parentId) {
        const parent = nodes.find((n) => n.id === node.parentId);
        if (parent) {
          nodeLeft = parent.position.x + node.position.x;
          nodeTop = parent.position.y + node.position.y;
        }
      }

      const nodeRight = nodeLeft + (node.width || 100);
      const nodeBottom = nodeTop + (node.height || 100);

      // Label bounds (centered at labelX, labelY)
      const labelLeft = labelX - labelWidth / 2;
      const labelRight = labelX + labelWidth / 2;
      const labelTop = labelY - labelHeight / 2;
      const labelBottom = labelY + labelHeight / 2;

      // Check if label overlaps with or is inside the node
      // Use padding to be more sensitive
      const nodeLeftWithPadding = nodeLeft - PADDING;
      const nodeRightWithPadding = nodeRight + PADDING;
      const nodeTopWithPadding = nodeTop - PADDING;
      const nodeBottomWithPadding = nodeBottom + PADDING;

      if (
        !(
          labelRight < nodeLeftWithPadding ||
          labelLeft > nodeRightWithPadding ||
          labelBottom < nodeTopWithPadding ||
          labelTop > nodeBottomWithPadding
        )
      ) {
        edgeHasOverlap = true;
        break; // Count once per edge
      }
    }

    if (edgeHasOverlap) {
      overlapCount++;
    }
  });

  return overlapCount;
}

/**
 * Calculate score for edge label overlaps
 */
function calculateEdgeLabelOverlapScore(overlapCount: number, edgeCount: number): number {
  if (edgeCount === 0) return 100;
  if (overlapCount === 0) return 100;

  // Each overlap significantly reduces score (more severe than edge crossings)
  const penalty = (overlapCount / edgeCount) * 50; // Reduced from 150 to be less draconian
  return Math.max(0, 100 - penalty);
}

/**
 * Detect node labels that are clipped or cut off
 * Labels should have adequate padding within node bounds
 */
function detectClippedNodeLabels(nodes: Node<C4NodeData>[]): number {
  let clippedCount = 0;
  const MIN_PADDING = 12; // Increased minimum padding required for labels (was 8)
  const TIGHT_THRESHOLD = 0.5; // Even lower threshold - if label area > 50% of node, it's cramped
  const CRAMPED_HEIGHT_RATIO = 0.35; // If label height is > 35% of node height, it's cramped

  nodes.forEach((node) => {
    if (!node.parentId) {
      // Only check root nodes (children use relative positioning)
      const label = node.data?.label || "";
      if (!label) return;

      const nodeWidth = node.width || 100;
      const nodeHeight = node.height || 100;

      // More accurate label dimension estimation
      // Account for icon spacing (person nodes have icons on left)
      const hasIcon = node.data?.type === "person";
      const iconSpace = hasIcon ? 40 : 0; // Icon takes ~40px

      // Also account for technology/description tags that might be below label
      const hasTechnology = !!node.data?.technology;
      const hasDescription = !!node.data?.description;
      const extraHeight = (hasTechnology ? 22 : 0) + (hasDescription ? 22 : 0);

      // Estimate label dimensions - more generous to catch clipping
      const labelWidth = label.length * 7 + 16;
      const labelHeight = 18; // Base label height

      // Available space accounting for icon and other content
      const availableWidth = nodeWidth - iconSpace - MIN_PADDING * 2;
      const availableHeight = nodeHeight - extraHeight - MIN_PADDING * 2;

      // Check if label is likely clipped - be more aggressive
      // If label text width exceeds available width (accounting for wrapping)
      const estimatedTextWidth = label.length * 7;
      if (estimatedTextWidth > availableWidth || labelHeight > availableHeight) {
        clippedCount++;
      } else {
        // Check if label height ratio is too high (cramped vertically)
        const totalContentHeight = labelHeight + extraHeight;
        const heightRatio = totalContentHeight / nodeHeight;
        if (heightRatio > CRAMPED_HEIGHT_RATIO) {
          clippedCount++;
        }

        // Check if label area ratio is too high (cramped overall)
        const labelArea = labelWidth * labelHeight;
        const nodeArea = nodeWidth * nodeHeight;
        const labelRatio = labelArea / nodeArea;

        if (labelRatio > TIGHT_THRESHOLD) {
          clippedCount++;
        }

        // Check if label bottom would be too close to node bottom
        // Account for icon at top (~10px), label below it (~30px start), height (~18px)
        const typicalLabelTop = hasIcon ? 30 : 20;
        const labelBottom = typicalLabelTop + totalContentHeight;
        const distanceFromBottom = nodeHeight - labelBottom;

        // More sensitive: if less than minimum padding, it's clipped
        if (distanceFromBottom < MIN_PADDING) {
          clippedCount++;
          return; // Already detected, move to next node
        }

        // Check if label would be too close to right edge
        const labelRight = iconSpace + estimatedTextWidth + MIN_PADDING;
        if (labelRight > nodeWidth - MIN_PADDING) {
          clippedCount++;
          return; // Already detected, move to next node
        }

        // Additional check: if node height is too small for content
        const minRequiredHeight = typicalLabelTop + totalContentHeight + MIN_PADDING;
        if (nodeHeight < minRequiredHeight) {
          clippedCount++;
          return;
        }
      }
    }
  });

  return clippedCount;
}

/**
 * Calculate score for clipped labels
 */
function calculateClippedLabelScore(clippedCount: number, nodeCount: number): number {
  if (nodeCount === 0) return 100;
  if (clippedCount === 0) return 100;

  // Each clipped label significantly impacts readability
  // More severe penalty: any clipped label should reduce score substantially
  // If 1 node out of 2 is clipped, that's 50% -> score should be much lower
  const penalty = (clippedCount / nodeCount) * 150; // 50% clipped = 25 score (was 40)
  return Math.max(0, 100 - penalty);
}

/**
 * Calculate comprehensive diagram quality metrics
 * @param level - Optional C4 level (L0, L1, L2, L3) to use level-specific weights
 */
export function calculateDiagramQuality(
  nodes: Node<C4NodeData>[],
  edges: Edge[],
  viewportSize: { width: number; height: number } = { width: 1920, height: 1080 },
  weights: QualityWeights = DEFAULT_QUALITY_WEIGHTS,
  level?: string
): DiagramQualityMetrics {
  // Use level-specific weights if level is provided and no custom weights given
  const effectiveWeights = level ? getQualityWeightsForLevel(level) : weights;
  const overlappingNodes = detectOverlaps(nodes);
  const overlapScore = calculateOverlapScore(nodes, overlappingNodes);

  const spacingViolations = detectSpacingViolations(nodes);
  const spacingMetrics = calculateSpacingMetrics(nodes);
  const spacingScore = calculateSpacingScore(spacingMetrics, spacingViolations);

  const edgeMetrics = calculateEdgeMetrics(nodes, edges);
  const edgesOverNodes = detectEdgesOverNodes(nodes, edges);
  const edgeBends = countEdgeBends(edges);
  const edgeScore = calculateEdgeScore(edgeMetrics, edgesOverNodes, edgeBends);

  const parentChildContainment = detectContainmentViolations(nodes);
  const hierarchyScore = calculateHierarchyScore(nodes, parentChildContainment);

  const viewportUtilization = calculateViewportUtilization(nodes, viewportSize);
  const viewportScore = viewportUtilization * 100;

  const consistencyScore = calculateConsistencyScore(nodes);

  // Direction score
  const directionViolations = detectDirectionViolations(nodes, edges);
  const directionScore = calculateDirectionScore(directionViolations, edges.length);

  // Aspect ratio and parent-child size checks
  const aspectRatio = calculateAspectRatio(nodes);
  const aspectRatioScore = calculateAspectRatioScore(aspectRatio);
  const parentChildSizeViolations = detectParentChildSizeViolations(nodes);

  // Empty space penalty
  const emptySpace = calculateEmptySpace(nodes);
  const emptySpaceScore = calculateEmptySpaceScore(emptySpace);

  // Label quality checks
  const edgeLabelOverlaps = detectEdgeLabelOverlaps(nodes, edges);
  const edgeLabelOverlapScore = calculateEdgeLabelOverlapScore(edgeLabelOverlaps, edges.length);
  const clippedNodeLabels = detectClippedNodeLabels(nodes);
  const clippedLabelScore = calculateClippedLabelScore(clippedNodeLabels, nodes.length);
  const edgeCongestionScore = calculateEdgeCongestionScore(nodes, edges);
  const crossingAngleScore = calculateCrossingAngleScore(nodes, edges);
  const alignmentScore = calculateAlignmentScore(nodes);
  const detourScore = calculateDetourScore(nodes, edges);

  // Debug logging for label issues
  if (edgeLabelOverlaps > 0 || clippedNodeLabels > 0) {
    console.log("[Quality] Label Issues Detected:", {
      edgeLabelOverlaps,
      clippedNodeLabels,
      edgeLabelOverlapScore: edgeLabelOverlapScore.toFixed(1),
      clippedLabelScore: clippedLabelScore.toFixed(1),
    });
  }

  // Calculate individual component scores
  const edgeCrossingScore = calculateEdgeCrossingScore(edgeMetrics.crossings, edges.length);
  const edgesOverNodesScore = calculateEdgesOverNodesScore(edgesOverNodes, edges.length);
  const edgeBendScore = calculateEdgeBendScore(edgeBends, edges.length);
  const edgeLengthScore = calculateEdgeLengthScore(edgeMetrics.length);

  // Weighted overall score using provided weights
  // Normalize weights if they don't sum to 1? Or just imply they do.
  // Sum of defaults = 0.2+0.1+0.2+0.15+0.05+0.05+0.15+0.03+0.02+0.05 + 0.10(new) = 1.10
  // We should normalize or adjust weights.
  // Let's adjust existing weights to make room for direction:
  // overlap: 0.15
  // spacing: 0.05
  // crossings: 0.15
  // hierarchy: 0.15
  // direction: 0.15
  // edgesOverNodes: 0.10
  // ...
  // Alternatively, just increase sum and Math.min(100) will cap it.

  const components = [
    { s: overlapScore, w: effectiveWeights.overlap },
    { s: spacingScore, w: effectiveWeights.spacing },
    { s: edgeCrossingScore, w: effectiveWeights.edgeCrossings },
    { s: edgesOverNodesScore, w: effectiveWeights.edgesOverNodes },
    { s: edgeBendScore, w: effectiveWeights.edgeBends },
    { s: edgeLengthScore, w: effectiveWeights.edgeLength },
    { s: hierarchyScore, w: effectiveWeights.hierarchy },
    { s: viewportScore, w: effectiveWeights.viewport },
    { s: consistencyScore, w: effectiveWeights.consistency },
    { s: aspectRatioScore, w: effectiveWeights.aspectRatio },
    { s: directionScore, w: effectiveWeights.direction || 0.1 },
    { s: emptySpaceScore, w: effectiveWeights.emptySpace || 0.05 },
    { s: edgeLabelOverlapScore, w: effectiveWeights.edgeLabelOverlaps || 0.1 },
    { s: clippedLabelScore, w: effectiveWeights.clippedLabels || 0.1 },
    { s: edgeCongestionScore, w: effectiveWeights.edgeCongestion || 0.05 },
    { s: crossingAngleScore, w: effectiveWeights.crossingAngles || 0.06 },
    { s: alignmentScore, w: effectiveWeights.alignment || 0.04 },
    { s: detourScore, w: effectiveWeights.detour || 0.03 },
  ];
  const totalWeight = components.reduce((acc, c) => acc + c.w, 0);
  const weightedSum = components.reduce((acc, c) => acc + c.s * c.w, 0);
  let weightedScore = Math.max(
    0,
    Math.min(100, totalWeight > 0 ? weightedSum / totalWeight : weightedSum)
  );

  // CRITICAL FAILURE CAP
  // If there are overlaps, containment violations, or label issues, the diagram is effectively broken.
  // Cap the score to ensure it doesn't get an "A" or "B".
  // Containment violations are especially critical - children outside parents break hierarchy
  const criticalViolations = overlappingNodes.length + parentChildContainment.length;
  const labelViolations = edgeLabelOverlaps + clippedNodeLabels;

  // Containment violations should have very strong penalty - they indicate broken layout
  const containmentViolationPenalty =
    parentChildContainment.length > 0 ? parentChildContainment.length * 25 : 0;

  // Label violations are also critical for readability
  if (labelViolations > 0 || criticalViolations > 0) {
    const totalViolations = criticalViolations + Math.ceil(labelViolations / 2); // Label violations count less
    // Containment violations are the most severe - cap even lower
    // 1 containment violation = max 60 (D)
    // 2+ containment violations = max 40 (F)
    // Other violations: 1 violation = max 85 (B), 3+ violations = max 70 (C)
    let cap: number;
    if (parentChildContainment.length > 0) {
      cap = Math.max(20, 60 - (parentChildContainment.length - 1) * 20);
    } else {
      // Relaxed cap for label issues - they are bad but shouldn't fail the diagram immediately
      cap = Math.max(60, 85 - totalViolations * 5);
    }
    // Force score down
    weightedScore = Math.min(weightedScore, cap);

    // Additional penalty for containment violations beyond the cap
    if (containmentViolationPenalty > 0) {
      weightedScore = Math.max(0, weightedScore - containmentViolationPenalty);
    }
  }

  // Also calculate unweighted average for comparison
  const overallScore = Math.max(
    0,
    Math.min(
      100,
      overlapScore * 0.15 +
        spacingScore * 0.1 +
        edgeScore * 0.1 +
        hierarchyScore * 0.15 +
        directionScore * 0.15 +
        viewportScore * 0.1 +
        consistencyScore * 0.1 +
        aspectRatioScore * 0.1
    )
  );

  const grade = getGrade(weightedScore);

  // Calculate node badness for heatmap
  // We construct a partial metrics object to satisfy the helper
  const metricsForBadness: DiagramQualityMetrics = {
    overlappingNodes,
    overlapScore,
    minSpacing: spacingMetrics.min,
    averageSpacing: spacingMetrics.average,
    spacingViolations,
    spacingScore,
    edgeCrossings: edgeMetrics.crossings,
    edgeOverlaps: edgeMetrics.overlaps,
    edgesOverNodes,
    edgeBends,
    edgeLength: edgeMetrics.length,
    edgeScore,
    parentChildContainment,
    hierarchyScore,
    viewportUtilization,
    viewportScore,
    consistencyScore,
    aspectRatio,
    aspectRatioScore,
    parentChildSizeViolations,
    overallScore,
    weightedScore,
    grade,
    directionViolations,
    directionScore,
    emptySpace,
    emptySpaceScore,
    edgeLabelOverlaps,
    edgeLabelOverlapScore,
    clippedNodeLabels,
    clippedLabelScore,
    edgeCongestionScore,
    crossingAngleScore,
    alignmentScore,
    detourScore,
    nodeBadness: {}, // placeholder
  };

  const nodeBadness = calculateNodeBadness(nodes, metricsForBadness);
  metricsForBadness.nodeBadness = nodeBadness;

  return metricsForBadness;
}

export function compositeObjective(metrics: DiagramQualityMetrics): number {
  let score = metrics.weightedScore;
  const containmentPenalty = metrics.parentChildContainment.length * 15;
  const overlapPenalty = metrics.overlappingNodes.length * 10;
  const crossingPenalty = Math.max(0, metrics.edgeCrossings - 3) * 5;
  const overNodesPenalty = Math.max(0, metrics.edgesOverNodes - 2) * 4;
  const labelPenalty = metrics.edgeLabelOverlaps * 6 + metrics.clippedNodeLabels * 6;
  const congestionPenalty = (100 - metrics.edgeCongestionScore) * 0.1;
  const anglePenalty = (100 - metrics.crossingAngleScore) * 0.1;
  const alignmentPenalty = (100 - metrics.alignmentScore) * 0.08;
  const detourPenalty = (100 - metrics.detourScore) * 0.06;
  let bonus = 0;
  if (metrics.viewportUtilization >= 0.6 && metrics.viewportUtilization <= 0.9) bonus += 3;
  if (metrics.directionScore >= 90) bonus += 3;
  if (metrics.spacingScore >= 85) bonus += 2;
  score = Math.max(
    0,
    Math.min(
      100,
      score -
        containmentPenalty -
        overlapPenalty -
        crossingPenalty -
        overNodesPenalty -
        labelPenalty -
        congestionPenalty -
        anglePenalty -
        alignmentPenalty -
        detourPenalty +
        bonus
    )
  );
  return score;
}

/**
 * Detect overlapping nodes
 */
function detectOverlaps(nodes: Node<C4NodeData>[]): OverlapViolation[] {
  const violations: OverlapViolation[] = [];

  for (let i = 0; i < nodes.length; i++) {
    for (let j = i + 1; j < nodes.length; j++) {
      const node1 = nodes[i];
      const node2 = nodes[j];

      // Skip if one is a child of the other (that's expected)
      if (node1.parentId === node2.id || node2.parentId === node1.id) {
        continue;
      }

      const overlap = calculateOverlap(node1, node2);
      if (overlap.area > 0) {
        const area1 = (node1.width || 100) * (node1.height || 100);
        const area2 = (node2.width || 100) * (node2.height || 100);
        const overlapPercentage = (overlap.area / Math.min(area1, area2)) * 100;

        violations.push({
          node1: node1.id,
          node2: node2.id,
          overlapArea: overlap.area,
          overlapPercentage,
        });
      }
    }
  }

  return violations;
}

function calculateOverlap(node1: Node, node2: Node): { area: number } {
  const w1 = node1.width || 100;
  const h1 = node1.height || 100;
  const w2 = node2.width || 100;
  const h2 = node2.height || 100;

  const x1 = node1.position.x;
  const y1 = node1.position.y;
  const x2 = node2.position.x;
  const y2 = node2.position.y;

  // Calculate bounding boxes
  const left1 = x1;
  const right1 = x1 + w1;
  const top1 = y1;
  const bottom1 = y1 + h1;

  const left2 = x2;
  const right2 = x2 + w2;
  const top2 = y2;
  const bottom2 = y2 + h2;

  // Check for overlap
  const overlapLeft = Math.max(left1, left2);
  const overlapRight = Math.min(right1, right2);
  const overlapTop = Math.max(top1, top2);
  const overlapBottom = Math.min(bottom1, bottom2);

  if (overlapLeft < overlapRight && overlapTop < overlapBottom) {
    const overlapWidth = overlapRight - overlapLeft;
    const overlapHeight = overlapBottom - overlapTop;
    return { area: overlapWidth * overlapHeight };
  }

  return { area: 0 };
}

function calculateOverlapScore(nodes: Node[], violations: OverlapViolation[]): number {
  if (nodes.length < 2) return 100;

  const totalPossiblePairs = (nodes.length * (nodes.length - 1)) / 2;
  const violationCount = violations.length;

  // Score based on percentage of non-overlapping pairs
  const score = ((totalPossiblePairs - violationCount) / totalPossiblePairs) * 100;
  return Math.max(0, Math.min(100, score));
}

/**
 * Detect spacing violations (nodes too close together)
 */
function detectSpacingViolations(nodes: Node<C4NodeData>[]): SpacingViolation[] {
  const violations: SpacingViolation[] = [];

  for (let i = 0; i < nodes.length; i++) {
    for (let j = i + 1; j < nodes.length; j++) {
      const node1 = nodes[i];
      const node2 = nodes[j];

      // Skip parent-child pairs (they have different spacing rules)
      if (node1.parentId === node2.id || node2.parentId === node1.id) {
        continue;
      }

      const distance = calculateNodeDistance(node1, node2);
      const minRequired = MIN_NODE_SPACING;

      if (distance < minRequired) {
        violations.push({
          node1: node1.id,
          node2: node2.id,
          distance,
          minRequired,
        });
      }
    }
  }

  return violations;
}

function calculateNodeDistance(node1: Node, node2: Node): number {
  const w1 = node1.width || 100;
  const h1 = node1.height || 100;
  const w2 = node2.width || 100;
  const h2 = node2.height || 100;

  const center1 = {
    x: node1.position.x + w1 / 2,
    y: node1.position.y + h1 / 2,
  };
  const center2 = {
    x: node2.position.x + w2 / 2,
    y: node2.position.y + h2 / 2,
  };

  // Calculate edge-to-edge distance
  const dx = Math.abs(center1.x - center2.x) - (w1 + w2) / 2;
  const dy = Math.abs(center1.y - center2.y) - (h1 + h2) / 2;

  if (dx < 0 && dy < 0) {
    // Overlapping - return negative distance
    return -Math.sqrt(dx * dx + dy * dy);
  }

  return Math.sqrt(Math.max(0, dx) ** 2 + Math.max(0, dy) ** 2);
}

function calculateSpacingMetrics(nodes: Node[]): { min: number; max: number; average: number } {
  if (nodes.length < 2) {
    return { min: Infinity, max: 0, average: 0 };
  }

  const distances: number[] = [];

  for (let i = 0; i < nodes.length; i++) {
    for (let j = i + 1; j < nodes.length; j++) {
      const node1 = nodes[i];
      const node2 = nodes[j];

      if (node1.parentId === node2.id || node2.parentId === node1.id) {
        continue;
      }

      const distance = calculateNodeDistance(node1, node2);
      if (distance > 0) {
        distances.push(distance);
      }
    }
  }

  if (distances.length === 0) {
    return { min: 0, max: 0, average: 0 };
  }

  return {
    min: Math.min(...distances),
    max: Math.max(...distances),
    average: distances.reduce((a, b) => a + b, 0) / distances.length,
  };
}

function calculateSpacingScore(
  metrics: { min: number; average: number },
  violations: SpacingViolation[]
): number {
  // Penalize violations
  const violationPenalty = violations.length * 5;

  // Reward good average spacing (optimal around 100-200px)
  const spacingBonus = metrics.average > 50 ? Math.min(20, (metrics.average - 50) / 10) : 0;

  const baseScore = 100 - violationPenalty + spacingBonus;
  return Math.max(0, Math.min(100, baseScore));
}

/**
 * Calculate edge metrics
 */
function calculateEdgeMetrics(
  nodes: Node[],
  edges: Edge[]
): {
  crossings: number;
  overlaps: number;
  length: { min: number; max: number; average: number };
} {
  const edgeLengths: number[] = [];
  let crossings = 0;
  let overlaps = 0;

  edges.forEach((edge) => {
    const sourceNode = nodes.find((n) => n.id === edge.source);
    const targetNode = nodes.find((n) => n.id === edge.target);

    if (!sourceNode || !targetNode) return;

    const length = Math.sqrt(
      Math.pow(targetNode.position.x - sourceNode.position.x, 2) +
        Math.pow(targetNode.position.y - sourceNode.position.y, 2)
    );
    edgeLengths.push(length);
  });

  // Improved crossing detection - use actual edge paths if available
  for (let i = 0; i < edges.length; i++) {
    for (let j = i + 1; j < edges.length; j++) {
      const e1 = edges[i];
      const e2 = edges[j];

      const s1 = nodes.find((n) => n.id === e1.source);
      const t1 = nodes.find((n) => n.id === e1.target);
      const s2 = nodes.find((n) => n.id === e2.source);
      const t2 = nodes.find((n) => n.id === e2.target);

      if (!s1 || !t1 || !s2 || !t2) continue;

      // Check if edges share both endpoints (same edge, skip)
      if (s1.id === s2.id && t1.id === t2.id) continue;
      if (s1.id === t2.id && t1.id === s2.id) continue;

      // Get actual edge paths if available (for routed edges with points)
      const e1Points = (e1.data as any)?.points as Array<{ x: number; y: number }> | undefined;
      const e2Points = (e2.data as any)?.points as Array<{ x: number; y: number }> | undefined;

      if (e1Points && e1Points.length > 1 && e2Points && e2Points.length > 1) {
        // Check if routed paths intersect
        let pathsCross = false;
        for (let k = 0; k < e1Points.length - 1 && !pathsCross; k++) {
          for (let l = 0; l < e2Points.length - 1 && !pathsCross; l++) {
            if (
              linesIntersectProperly(e1Points[k], e1Points[k + 1], e2Points[l], e2Points[l + 1])
            ) {
              pathsCross = true;
            }
          }
        }
        if (pathsCross) {
          crossings++;
        }
      } else {
        // Fallback to node-based crossing detection
        if (doEdgesCross(s1, t1, s2, t2)) {
          crossings++;
        }
      }
    }
  }

  return {
    crossings,
    overlaps,
    length:
      edgeLengths.length > 0
        ? {
            min: Math.min(...edgeLengths),
            max: Math.max(...edgeLengths),
            average: edgeLengths.reduce((a, b) => a + b, 0) / edgeLengths.length,
          }
        : { min: 0, max: 0, average: 0 },
  };
}

function doEdgesCross(s1: Node, t1: Node, s2: Node, t2: Node): boolean {
  // More accurate line segment intersection check
  // Get actual edge paths if available (for routed edges)
  // For now, use center-to-center as approximation, but check if they share endpoints
  const shareSource = s1.id === s2.id;
  const shareTarget = t1.id === t2.id;
  const shareEndpoints = shareSource || shareTarget;

  // If edges share an endpoint, they don't cross (they meet at a node)
  if (shareEndpoints) {
    // Check if it's the same edge (both endpoints same)
    if (shareSource && shareTarget) return false;

    // If they share one endpoint, check if the other endpoints are on opposite sides
    // This would indicate a crossing, but for edges from same node, it's usually fine
    // Only count as crossing if the paths actually intersect in the middle
    const p1 = {
      x: s1.position.x + (s1.width || 100) / 2,
      y: s1.position.y + (s1.height || 100) / 2,
    };
    const p2 = {
      x: t1.position.x + (t1.width || 100) / 2,
      y: t1.position.y + (t1.height || 100) / 2,
    };
    const p3 = {
      x: s2.position.x + (s2.width || 100) / 2,
      y: s2.position.y + (s2.height || 100) / 2,
    };
    const p4 = {
      x: t2.position.x + (t2.width || 100) / 2,
      y: t2.position.y + (t2.height || 100) / 2,
    };

    // Use line segment intersection formula
    return linesIntersectProperly(p1, p2, p3, p4);
  }

  // For edges with different endpoints, use bounding box as approximation
  const x1min = Math.min(s1.position.x, t1.position.x);
  const x1max = Math.max(s1.position.x, t1.position.x);
  const y1min = Math.min(s1.position.y, t1.position.y);
  const y1max = Math.max(s1.position.y, t1.position.y);

  const x2min = Math.min(s2.position.x, t2.position.x);
  const x2max = Math.max(s2.position.x, t2.position.x);
  const y2min = Math.min(s2.position.y, t2.position.y);
  const y2max = Math.max(s2.position.y, t2.position.y);

  // Check if bounding boxes overlap
  const bboxOverlap = !(x1max < x2min || x2max < x1min || y1max < y2min || y2max < y1min);

  if (!bboxOverlap) return false;

  // If bounding boxes overlap, do more precise check
  const p1 = {
    x: s1.position.x + (s1.width || 100) / 2,
    y: s1.position.y + (s1.height || 100) / 2,
  };
  const p2 = {
    x: t1.position.x + (t1.width || 100) / 2,
    y: t1.position.y + (t1.height || 100) / 2,
  };
  const p3 = {
    x: s2.position.x + (s2.width || 100) / 2,
    y: s2.position.y + (s2.height || 100) / 2,
  };
  const p4 = {
    x: t2.position.x + (t2.width || 100) / 2,
    y: t2.position.y + (t2.height || 100) / 2,
  };

  return linesIntersectProperly(p1, p2, p3, p4);
}

function linesIntersectProperly(
  p1: { x: number; y: number },
  p2: { x: number; y: number },
  p3: { x: number; y: number },
  p4: { x: number; y: number }
): boolean {
  // Line segment intersection using cross product
  const ccw = (
    A: { x: number; y: number },
    B: { x: number; y: number },
    C: { x: number; y: number }
  ) => {
    return (C.y - A.y) * (B.x - A.x) > (B.y - A.y) * (C.x - A.x);
  };

  // Two line segments (p1-p2) and (p3-p4) intersect if:
  // ccw(p1,p3,p4) != ccw(p2,p3,p4) && ccw(p1,p2,p3) != ccw(p1,p2,p4)
  return ccw(p1, p3, p4) !== ccw(p2, p3, p4) && ccw(p1, p2, p3) !== ccw(p1, p2, p4);
}

function calculateEdgeScore(
  metrics: ReturnType<typeof calculateEdgeMetrics>,
  edgesOverNodes: number,
  edgeBends: number
): number {
  let score = 100;

  // Penalize crossings (each crossing reduces score by 2)
  score -= metrics.crossings * 2;

  // Penalize edges over nodes (each reduces score by 3)
  score -= edgesOverNodes * 3;

  // Penalize bends (each bend reduces score by 1)
  score -= edgeBends * 1;

  // Reward reasonable edge lengths (not too short, not too long)
  const avgLength = metrics.length.average;
  if (avgLength < 50) {
    score -= 10; // Too short
  } else if (avgLength > 1000) {
    score -= 10; // Too long
  }

  return Math.max(0, Math.min(100, score));
}

/**
 * Calculate score specifically for edge crossings
 * For complex diagrams with many edges, some crossings are unavoidable
 * The formula scales based on edge count to be more forgiving for dense graphs
 */
function calculateEdgeCrossingScore(crossings: number, edgeCount: number): number {
  if (edgeCount === 0) return 100;
  if (crossings === 0) return 100;

  // For complex diagrams (many edges), crossings are harder to avoid
  // Use a logarithmic scale for penalty and adjust based on edge density
  const normalizedCrossings = crossings / edgeCount;

  // Scale penalty based on edge count:
  // - Few edges (< 10): crossings are very bad, harsh penalty
  // - Medium edges (10-20): moderate penalty
  // - Many edges (> 20): some crossings acceptable, lenient penalty
  let penaltyMultiplier: number;
  if (edgeCount <= 10) {
    penaltyMultiplier = 150; // Harsh for simple diagrams
  } else if (edgeCount <= 20) {
    penaltyMultiplier = 100; // Moderate for medium diagrams
  } else {
    // For complex diagrams, use logarithmic scaling
    // More edges = more potential crossings = lower penalty per crossing
    penaltyMultiplier = Math.max(50, 120 - (edgeCount - 20) * 2);
  }

  // Also consider absolute crossing count - 50+ crossings is always bad
  const absolutePenalty = crossings > 50 ? (crossings - 50) * 0.5 : 0;

  const score = Math.max(0, 100 - normalizedCrossings * penaltyMultiplier - absolutePenalty);
  return score;
}

/**
 * Calculate score for edges passing over nodes
 */
function calculateEdgesOverNodesScore(edgesOverNodes: number, edgeCount: number): number {
  if (edgeCount === 0) return 100;

  const normalized = edgesOverNodes / edgeCount;
  // Score: 100 - (normalized * 150)
  // 0 = 100, 0.33 = 50, 0.67+ = 0
  const score = Math.max(0, 100 - normalized * 150);
  return score;
}

/**
 * Calculate score for edge bends
 */
function calculateEdgeBendScore(edgeBends: number, edgeCount: number): number {
  if (edgeCount === 0) return 100;

  const avgBendsPerEdge = edgeBends / edgeCount;
  // Score: 100 - (avg_bends * 20)
  // 0 bends = 100, 2.5 bends/edge = 50, 5+ = 0
  const score = Math.max(0, 100 - avgBendsPerEdge * 20);
  return score;
}

/**
 * Calculate score for edge length
 */
function calculateEdgeLengthScore(length: { min: number; max: number; average: number }): number {
  if (length.average === 0) return 100;

  let score = 100;

  // Optimal length is 100-300px
  const optimalMin = 100;
  const optimalMax = 300;

  if (length.average < optimalMin) {
    // Too short - penalize
    score -= (optimalMin - length.average) / 5;
  } else if (length.average > optimalMax) {
    // Too long - penalize
    score -= (length.average - optimalMax) / 20;
  }

  // Also check for extreme outliers
  if (length.max > 2000) {
    score -= 20; // Very long edges are bad
  }
  if (length.min < 20) {
    score -= 10; // Very short edges might indicate overlap
  }

  return Math.max(0, Math.min(100, score));
}

function calculateEdgeCongestionScore(nodes: Node<C4NodeData>[], edges: Edge[]): number {
  if (edges.length === 0 || nodes.length === 0) return 100;
  const positions = nodes.map((n) => ({
    x: n.position.x,
    y: n.position.y,
    w: n.width || 100,
    h: n.height || 100,
  }));
  const minX = Math.min(...positions.map((p) => p.x));
  const maxX = Math.max(...positions.map((p) => p.x + p.w));
  const minY = Math.min(...positions.map((p) => p.y));
  const maxY = Math.max(...positions.map((p) => p.y + p.h));
  const width = Math.max(1, maxX - minX);
  const height = Math.max(1, maxY - minY);
  // Use finer grid for better congestion detection
  const cols = 8,
    rows = 8;
  const counts = Array.from({ length: cols * rows }, () => 0);
  edges.forEach((edge) => {
    const s = nodes.find((n) => n.id === edge.source);
    const t = nodes.find((n) => n.id === edge.target);
    if (!s || !t) return;
    const cx = (s.position.x + (s.width || 100) / 2 + t.position.x + (t.width || 100) / 2) / 2;
    const cy = (s.position.y + (s.height || 100) / 2 + t.position.y + (t.height || 100) / 2) / 2;
    const ix = Math.max(0, Math.min(cols - 1, Math.floor(((cx - minX) / width) * cols)));
    const iy = Math.max(0, Math.min(rows - 1, Math.floor(((cy - minY) / height) * rows)));
    counts[iy * cols + ix]++;
  });
  const maxCell = Math.max(...counts);
  const ratio = maxCell / edges.length;
  // More lenient scoring for complex diagrams - some congestion is unavoidable
  // Scale penalty based on edge count: more edges = more lenient
  let penaltyMultiplier = 100;
  if (edges.length > 20) {
    penaltyMultiplier = 80; // More lenient for complex diagrams
  }
  if (edges.length > 40) {
    penaltyMultiplier = 60; // Even more lenient for very complex diagrams
  }
  const score = Math.max(0, 100 - ratio * penaltyMultiplier);
  return score;
}

function calculateCrossingAngleScore(nodes: Node<C4NodeData>[], edges: Edge[]): number {
  let angles: number[] = [];
  for (let i = 0; i < edges.length; i++) {
    for (let j = i + 1; j < edges.length; j++) {
      const e1 = edges[i];
      const e2 = edges[j];
      const s1 = nodes.find((n) => n.id === e1.source);
      const t1 = nodes.find((n) => n.id === e1.target);
      const s2 = nodes.find((n) => n.id === e2.source);
      const t2 = nodes.find((n) => n.id === e2.target);
      if (!s1 || !t1 || !s2 || !t2) continue;
      const l1 = { x1: s1.position.x, y1: s1.position.y, x2: t1.position.x, y2: t1.position.y };
      const l2 = { x1: s2.position.x, y1: s2.position.y, x2: t2.position.x, y2: t2.position.y };
      if (!linesIntersect(l1, l2)) continue;
      const v1x = t1.position.x - s1.position.x;
      const v1y = t1.position.y - s1.position.y;
      const v2x = t2.position.x - s2.position.x;
      const v2y = t2.position.y - s2.position.y;
      const dot = Math.abs(v1x * v2x + v1y * v2y);
      const len1 = Math.sqrt(v1x * v1x + v1y * v1y);
      const len2 = Math.sqrt(v2x * v2x + v2y * v2y);
      if (len1 === 0 || len2 === 0) continue;
      const cos = Math.min(1, Math.max(-1, dot / (len1 * len2)));
      const angle = Math.acos(cos) * (180 / Math.PI);
      angles.push(angle);
    }
  }
  if (angles.length === 0) return 100;
  const avgSeverity =
    angles.reduce((a, b) => a + Math.max(0, 45 - Math.min(90, b)) / 45, 0) / angles.length;
  const score = Math.max(0, 100 - avgSeverity * 100);
  return score;
}

function calculateAlignmentScore(nodes: Node<C4NodeData>[]): number {
  if (nodes.length < 3) return 100;
  const groups = new Map<string, Node<C4NodeData>[]>();
  nodes.forEach((n) => {
    const key = n.parentId || "root";
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key)!.push(n);
  });
  let scores: number[] = [];
  groups.forEach((list) => {
    if (list.length < 3) return;
    const tol = 8;
    const rows: number[] = [];
    const cols: number[] = [];
    list.forEach((n) => {
      const y = n.parentId ? n.position.y : n.position.y;
      const x = n.parentId ? n.position.x : n.position.x;
      let r = rows.find((v) => Math.abs(v - y) <= tol);
      if (r === undefined) rows.push(y);
      let c = cols.find((v) => Math.abs(v - x) <= tol);
      if (c === undefined) cols.push(x);
    });
    const rowBins = rows.map((rv) => list.filter((n) => Math.abs(n.position.y - rv) <= tol).length);
    const colBins = cols.map((cv) => list.filter((n) => Math.abs(n.position.x - cv) <= tol).length);
    const maxRow = rowBins.length ? Math.max(...rowBins) : 1;
    const maxCol = colBins.length ? Math.max(...colBins) : 1;
    const best = Math.max(maxRow, maxCol);
    scores.push(Math.max(0, Math.min(100, (best / list.length) * 100)));
  });
  if (scores.length === 0) return 100;
  return scores.reduce((a, b) => a + b, 0) / scores.length;
}

function calculateDetourScore(nodes: Node<C4NodeData>[], edges: Edge[]): number {
  if (edges.length === 0) return 100;
  let ratios: number[] = [];
  edges.forEach((edge) => {
    const s = nodes.find((n) => n.id === edge.source);
    const t = nodes.find((n) => n.id === edge.target);
    if (!s || !t) return;
    const straight = Math.hypot(t.position.x - s.position.x, t.position.y - s.position.y);
    let pathLen = straight;
    const pts = (edge.data as any)?.points;
    if (Array.isArray(pts) && pts.length >= 2) {
      pathLen = 0;
      for (let i = 1; i < pts.length; i++) {
        const p0 = pts[i - 1];
        const p1 = pts[i];
        pathLen += Math.hypot((p1.x ?? 0) - (p0.x ?? 0), (p1.y ?? 0) - (p0.y ?? 0));
      }
    }
    if (straight > 0) ratios.push(pathLen / straight);
  });
  if (ratios.length === 0) return 100;
  const avg = ratios.reduce((a, b) => a + b, 0) / ratios.length;
  const excess = Math.max(0, avg - 1);
  const penalty = Math.min(100, excess * 80);
  return Math.max(0, 100 - penalty);
}

/**
 * Detect edges passing over/through nodes
 */
function detectEdgesOverNodes(nodes: Node<C4NodeData>[], edges: Edge[]): number {
  let count = 0;

  edges.forEach((edge) => {
    const sourceNode = nodes.find((n) => n.id === edge.source);
    const targetNode = nodes.find((n) => n.id === edge.target);

    if (!sourceNode || !targetNode) return;

    // Check if edge passes through any intermediate nodes
    const edgePath = {
      x1: sourceNode.position.x + (sourceNode.width || 100) / 2,
      y1: sourceNode.position.y + (sourceNode.height || 100) / 2,
      x2: targetNode.position.x + (targetNode.width || 100) / 2,
      y2: targetNode.position.y + (targetNode.height || 100) / 2,
    };

    // Check each node (excluding source and target)
    nodes.forEach((node) => {
      if (node.id === edge.source || node.id === edge.target) return;
      if (node.parentId === edge.source || node.parentId === edge.target) return; // Skip children

      const nodeRect = {
        x: node.position.x,
        y: node.position.y,
        width: node.width || 100,
        height: node.height || 100,
      };

      // Check if edge line intersects node rectangle
      if (lineIntersectsRect(edgePath, nodeRect)) {
        count++;
      }
    });
  });

  return count;
}

/**
 * Check if a line segment intersects a rectangle
 */
function lineIntersectsRect(
  line: { x1: number; y1: number; x2: number; y2: number },
  rect: { x: number; y: number; width: number; height: number }
): boolean {
  // Check if line endpoints are inside rect
  if (
    (line.x1 >= rect.x &&
      line.x1 <= rect.x + rect.width &&
      line.y1 >= rect.y &&
      line.y1 <= rect.y + rect.height) ||
    (line.x2 >= rect.x &&
      line.x2 <= rect.x + rect.width &&
      line.y2 >= rect.y &&
      line.y2 <= rect.y + rect.height)
  ) {
    return true;
  }

  // Check intersection with rectangle edges
  const rectEdges = [
    { x1: rect.x, y1: rect.y, x2: rect.x + rect.width, y2: rect.y }, // top
    { x1: rect.x + rect.width, y1: rect.y, x2: rect.x + rect.width, y2: rect.y + rect.height }, // right
    { x1: rect.x + rect.width, y1: rect.y + rect.height, x2: rect.x, y2: rect.y + rect.height }, // bottom
    { x1: rect.x, y1: rect.y + rect.height, x2: rect.x, y2: rect.y }, // left
  ];

  for (const edge of rectEdges) {
    if (linesIntersect(line, edge)) {
      return true;
    }
  }

  return false;
}

/**
 * Check if two line segments intersect
 */
function linesIntersect(
  line1: { x1: number; y1: number; x2: number; y2: number },
  line2: { x1: number; y1: number; x2: number; y2: number }
): boolean {
  const denom =
    (line1.x1 - line1.x2) * (line2.y1 - line2.y2) - (line1.y1 - line1.y2) * (line2.x1 - line2.x2);
  if (Math.abs(denom) < 0.0001) return false; // Parallel lines

  const t =
    ((line1.x1 - line2.x1) * (line2.y1 - line2.y2) -
      (line1.y1 - line2.y1) * (line2.x1 - line2.x2)) /
    denom;
  const u =
    -(
      (line1.x1 - line1.x2) * (line1.y1 - line2.y1) -
      (line1.y1 - line1.y2) * (line1.x1 - line2.x1)
    ) / denom;

  return t >= 0 && t <= 1 && u >= 0 && u <= 1;
}

/**
 * Count total number of bends in edges
 */
function countEdgeBends(edges: Edge[]): number {
  let totalBends = 0;

  edges.forEach((edge) => {
    // Check if edge has points (bends) in its data
    const points = (edge.data as any)?.points;
    if (points && Array.isArray(points)) {
      // Number of bends = points.length - 2 (excluding start and end)
      totalBends += Math.max(0, points.length - 2);
    } else {
      // If no explicit points, check if it's a straight line
      // For now, assume straight lines have 0 bends
      // This could be enhanced to detect actual routing
    }
  });

  return totalBends;
}

/**
 * Detect parent-child containment violations
 * React Flow uses relative positioning for children (child.position is relative to parent's (0,0))
 */
function detectContainmentViolations(nodes: Node<C4NodeData>[]): ContainmentViolation[] {
  const violations: ContainmentViolation[] = [];
  const nodeMap = new Map(nodes.map((n) => [n.id, n]));

  nodes.forEach((child) => {
    if (!child.parentId) return;

    const parent = nodeMap.get(child.parentId);
    if (!parent) return;

    // Child positions are RELATIVE to parent's top-left corner (0,0)
    // Parent's coordinate system: [0, 0] to [parent.width, parent.height]
    const childLeft = child.position.x;
    const childTop = child.position.y;
    const childWidth = child.width || 100;
    const childHeight = child.height || 100;
    const childRight = childLeft + childWidth;
    const childBottom = childTop + childHeight;

    const parentWidth = parent.width || 200;
    const parentHeight = parent.height || 200;

    // Check if child extends beyond parent bounds (with padding requirement)
    const padding = MIN_PARENT_PADDING;
    let violation: "outside" | "overlapping" | "too-close-to-edge" | undefined;
    let details = "";

    // Check each boundary
    const overflowLeft = Math.min(0, childLeft - padding);
    const overflowTop = Math.min(0, childTop - padding);
    const overflowRight = Math.max(0, childRight - (parentWidth - padding));
    const overflowBottom = Math.max(0, childBottom - (parentHeight - padding));

    if (overflowLeft < 0 || overflowTop < 0 || overflowRight > 0 || overflowBottom > 0) {
      // Determine violation type
      if (childLeft < 0 || childTop < 0 || childRight > parentWidth || childBottom > parentHeight) {
        violation = "outside";
        details = `Child completely outside parent bounds`;
      } else if (
        childLeft < padding ||
        childTop < padding ||
        childRight > parentWidth - padding ||
        childBottom > parentHeight - padding
      ) {
        violation = "too-close-to-edge";
        details = `Child too close to parent edge (padding violation)`;
      }

      // Add detailed overflow information
      const overflows: string[] = [];
      if (overflowLeft < 0) overflows.push(`Left: ${Math.abs(overflowLeft).toFixed(1)}px`);
      if (overflowTop < 0) overflows.push(`Top: ${Math.abs(overflowTop).toFixed(1)}px`);
      if (overflowRight > 0) overflows.push(`Right: ${overflowRight.toFixed(1)}px`);
      if (overflowBottom > 0) overflows.push(`Bottom: ${overflowBottom.toFixed(1)}px`);

      if (overflows.length > 0) {
        details += ` (${overflows.join(", ")})`;
      }

      violations.push({
        childId: child.id,
        parentId: parent.id,
        violation: violation || "outside",
        details: details,
      });
    }
  });

  return violations;
}

function calculateHierarchyScore(nodes: Node[], violations: ContainmentViolation[]): number {
  const totalChildren = nodes.filter((n) => n.parentId).length;
  if (totalChildren === 0) return 100;

  const violationCount = violations.length;

  // Strong penalty: each containment violation severely impacts hierarchy score
  // Base score calculation
  let score = ((totalChildren - violationCount) / totalChildren) * 100;

  // Additional penalty: any violation should heavily penalize
  // Even one violation is a critical issue
  if (violationCount > 0) {
    // For violations, apply severe penalty
    // 1 violation = max 50, 2+ = max 30, 3+ = 0
    score = Math.max(0, 50 - (violationCount - 1) * 20);
  }

  return Math.max(0, Math.min(100, score));
}

/**
 * Calculate viewport utilization
 */
function calculateViewportUtilization(
  nodes: Node[],
  viewportSize: { width: number; height: number }
): number {
  if (nodes.length === 0) return 0;

  const positions = nodes.map((n) => ({
    x: n.position.x,
    y: n.position.y,
    width: n.width || 100,
    height: n.height || 100,
  }));

  const minX = Math.min(...positions.map((p) => p.x));
  const maxX = Math.max(...positions.map((p) => p.x + p.width));
  const minY = Math.min(...positions.map((p) => p.y));
  const maxY = Math.max(...positions.map((p) => p.y + p.height));

  const diagramWidth = maxX - minX;
  const diagramHeight = maxY - minY;

  const utilizationX = Math.min(1, diagramWidth / viewportSize.width);
  const utilizationY = Math.min(1, diagramHeight / viewportSize.height);

  // Optimal utilization is around 70-90%
  const optimalRange = { min: 0.7, max: 0.9 };
  if (
    utilizationX >= optimalRange.min &&
    utilizationX <= optimalRange.max &&
    utilizationY >= optimalRange.min &&
    utilizationY <= optimalRange.max
  ) {
    return 1.0;
  }

  // Penalize if too small or too large
  const avgUtilization = (utilizationX + utilizationY) / 2;
  if (avgUtilization < optimalRange.min) {
    return avgUtilization / optimalRange.min;
  } else {
    return 1 - (avgUtilization - optimalRange.max) / (1 - optimalRange.max);
  }
}

/**
 * Calculate empty space percentage within the diagram bounding box
 */
function calculateEmptySpace(nodes: Node<C4NodeData>[]): number {
  if (nodes.length < 2) return 0;

  // Calculate diagram bounding box
  let minX = Infinity,
    minY = Infinity;
  let maxX = -Infinity,
    maxY = -Infinity;
  let totalNodeArea = 0;

  nodes.forEach((node) => {
    // Update global bounds
    const x = node.position.x;
    const y = node.position.y;
    const w = node.width || 100;
    const h = node.height || 100;

    minX = Math.min(minX, x);
    minY = Math.min(minY, y);
    maxX = Math.max(maxX, x + w);
    maxY = Math.max(maxY, y + h);

    if (!node.parentId) {
      totalNodeArea += w * h;
    }
  });

  if (minX === Infinity || maxX === -Infinity) return 0;

  const boundingBoxWidth = maxX - minX;
  const boundingBoxHeight = maxY - minY;

  // The strict bounding box area:
  const boundingBoxArea = boundingBoxWidth * boundingBoxHeight;

  if (boundingBoxArea <= 0) return 0;

  const density = totalNodeArea / boundingBoxArea;
  const emptySpace = 1 - density;

  return Math.max(0, emptySpace);
}

function calculateEmptySpaceScore(emptySpace: number): number {
  // We expect some empty space for spacing (padding, gaps).
  // Ideal density is perhaps around 40-60%.
  // So ideal empty space is 0.4 - 0.6.
  // If empty space > 0.8 (80%), it's too sparse.
  // If empty space < 0.2 (20%), it's too crowded.

  if (emptySpace > 0.8) {
    // Linearly penalize from 100 down to 0 as it goes from 0.8 to 1.0
    // (emptySpace - 0.8) / 0.2
    const penalty = ((emptySpace - 0.8) / 0.2) * 100;
    return Math.max(0, 100 - penalty);
  }

  if (emptySpace < 0.2) {
    // Linearly penalize from 100 down to 0 as it goes from 0.2 to 0.0
    // (0.2 - emptySpace) / 0.2
    const penalty = ((0.2 - emptySpace) / 0.2) * 100;
    return Math.max(0, 100 - penalty);
  }

  // Optimal range: 0.2 - 0.8 (40-60% density)
  return 100;
}

/**
 * Calculate consistency score (similar nodes positioned consistently)
 */
function calculateConsistencyScore(nodes: Node<C4NodeData>[]): number {
  // Group nodes by type
  const nodesByType = new Map<string, Node[]>();
  nodes.forEach((node) => {
    const type = node.data.type || "unknown";
    if (!nodesByType.has(type)) {
      nodesByType.set(type, []);
    }
    nodesByType.get(type)!.push(node);
  });

  let totalVariance = 0;
  let typeCount = 0;

  nodesByType.forEach((typeNodes) => {
    if (typeNodes.length < 2) return;

    // Calculate variance in X and Y positions
    const xPositions = typeNodes.map((n) => n.position.x);
    const yPositions = typeNodes.map((n) => n.position.y);

    const xMean = xPositions.reduce((a, b) => a + b, 0) / xPositions.length;
    const yMean = yPositions.reduce((a, b) => a + b, 0) / yPositions.length;

    const xVariance =
      xPositions.reduce((sum, x) => sum + Math.pow(x - xMean, 2), 0) / xPositions.length;
    const yVariance =
      yPositions.reduce((sum, y) => sum + Math.pow(y - yMean, 2), 0) / yPositions.length;

    // Lower variance = more consistent = higher score
    const avgVariance = (xVariance + yVariance) / 2;
    totalVariance += avgVariance;
    typeCount++;
  });

  // Calculate variance
  if (typeCount > 0) {
    // Average variance per group
    // Weighted by group size? Or just average?
    // Let's use average variance normalized by max possible

    // Normalize variance: 0 = perfect, 1000+ = bad
    // Score = 100 - (variance / 10)
    const avgVariance = totalVariance / typeCount;
    const score = Math.max(0, 100 - avgVariance / 50);
    return score;
  }

  return 100;
}

/**
 * Detect direction violations (edges flowing against the rank direction)
 * Assumes 'DOWN' layout (TB) roughly correlates with +Y
 */
function detectDirectionViolations(nodes: Node<C4NodeData>[], edges: Edge[]): DirectionViolation[] {
  const violations: DirectionViolation[] = [];

  edges.forEach((edge) => {
    const source = nodes.find((n) => n.id === edge.source);
    const target = nodes.find((n) => n.id === edge.target);

    if (!source || !target) return;

    // Skip self-loops
    if (source.id === target.id) return;

    // Skip if parent/child relationship (handled by containment)
    if (source.parentId === target.id || target.parentId === source.id) return;

    // Check Y coordinate
    // In TB layout, target should be below source (y > y)
    // Allow some threshold (e.g. same rank is fine)
    const threshold = 10;

    if (target.position.y < source.position.y - threshold) {
      violations.push({
        edgeId: edge.id,
        sourceId: source.id,
        targetId: target.id,
        deltaY: source.position.y - target.position.y,
      });
    }
  });

  return violations;
}

function calculateDirectionScore(violations: DirectionViolation[], edgeCount: number): number {
  if (edgeCount === 0) return 100;

  // Normalize by edge count
  const violationRatio = violations.length / edgeCount;

  // 0 violations = 100
  // 10% violations = 80
  // 50% violations = 0
  return Math.max(0, 100 - violationRatio * 200);
}

/**
 * Calculate per-node badness score (0-1) for heatmap
 */
export function calculateNodeBadness(
  nodes: Node<C4NodeData>[],
  metrics: DiagramQualityMetrics
): Record<string, number> {
  const badness: Record<string, number> = {};

  // Initialize
  nodes.forEach((n) => (badness[n.id] = 0));

  // 1. Overlaps (Critical: 1.0)
  metrics.overlappingNodes.forEach((v) => {
    badness[v.node1] = Math.max(badness[v.node1] || 0, 1.0);
    badness[v.node2] = Math.max(badness[v.node2] || 0, 1.0);
  });

  // 2. Spacing Violations (Moderate: 0.5+)
  metrics.spacingViolations.forEach((v) => {
    // badness proportional to how close they are
    // required = 100, actual = 10 -> very bad (0.9)
    const severity = Math.min(1.0, (v.minRequired - v.distance) / v.minRequired);
    // Weight: 0.6
    const addedBadness = severity * 0.6;
    badness[v.node1] = Math.min(1.0, (badness[v.node1] || 0) + addedBadness);
    badness[v.node2] = Math.min(1.0, (badness[v.node2] || 0) + addedBadness);
  });

  // 3. Edges Over Nodes (High: 0.8)
  // We need to know WHICH node was crossed.
  // The metric 'edgesOverNodes' is a count.
  // We need 'detectEdgesOverNodes' to return the list of crossed nodes to map it accurately.
  // For now, we'll skip direct mapping unless we update detectEdgesOverNodes.
  // Let's rely on overlap/spacing for now.

  // 4. Containment Violations (Critical: 1.0)
  metrics.parentChildContainment.forEach((v) => {
    badness[v.childId] = 1.0;
    badness[v.parentId] = Math.max(badness[v.parentId] || 0, 0.5); // Parent is partially to blame?
  });

  return badness;
}

export interface DirectionViolation {
  edgeId: string;
  sourceId: string;
  targetId: string;
  deltaY: number;
}

/**
 * Calculate diagram aspect ratio (width/height)
 */
function calculateAspectRatio(nodes: Node<C4NodeData>[]): number {
  if (nodes.length === 0) return 1;

  const positions = nodes.map((n) => ({
    x: n.position.x,
    y: n.position.y,
    width: n.width || 100,
    height: n.height || 100,
  }));

  const minX = Math.min(...positions.map((p) => p.x));
  const maxX = Math.max(...positions.map((p) => p.x + p.width));
  const minY = Math.min(...positions.map((p) => p.y));
  const maxY = Math.max(...positions.map((p) => p.y + p.height));

  const diagramWidth = maxX - minX;
  const diagramHeight = maxY - minY;

  if (diagramHeight === 0) return 1;
  return diagramWidth / diagramHeight;
}

/**
 * Calculate score for aspect ratio (penalize extreme ratios)
 */
function calculateAspectRatioScore(aspectRatio: number): number {
  if (aspectRatio >= OPTIMAL_ASPECT_RATIO_MIN && aspectRatio <= OPTIMAL_ASPECT_RATIO_MAX) {
    return 100; // Optimal range
  }

  // Penalize deviation from optimal range
  if (aspectRatio < OPTIMAL_ASPECT_RATIO_MIN) {
    // Too tall (aspect ratio too small)
    const deviation = OPTIMAL_ASPECT_RATIO_MIN - aspectRatio;
    return Math.max(0, 100 - deviation * 100);
  } else {
    // Too wide (aspect ratio too large)
    const deviation = aspectRatio - OPTIMAL_ASPECT_RATIO_MAX;
    return Math.max(0, 100 - deviation * 50);
  }
}

/**
 * Detect parent nodes that are too small for their children
 */
function detectParentChildSizeViolations(nodes: Node<C4NodeData>[]): ParentChildSizeViolation[] {
  const violations: ParentChildSizeViolation[] = [];
  const nodeMap = new Map(nodes.map((n) => [n.id, n]));

  // Group children by parent
  const childrenByParent = new Map<string, Node<C4NodeData>[]>();
  nodes.forEach((node) => {
    if (node.parentId) {
      if (!childrenByParent.has(node.parentId)) {
        childrenByParent.set(node.parentId, []);
      }
      childrenByParent.get(node.parentId)!.push(node);
    }
  });

  // Check each parent
  childrenByParent.forEach((children, parentId) => {
    const parent = nodeMap.get(parentId);
    if (!parent) return;

    // Calculate required size to contain all children with padding
    const padding = MIN_PARENT_PADDING * 2;
    let minChildX = Infinity;
    let maxChildX = -Infinity;
    let minChildY = Infinity;
    let maxChildY = -Infinity;

    children.forEach((child) => {
      // For children, position is relative to parent
      const childX = child.position.x;
      const childY = child.position.y;
      const childWidth = child.width || 100;
      const childHeight = child.height || 100;

      minChildX = Math.min(minChildX, childX);
      maxChildX = Math.max(maxChildX, childX + childWidth);
      minChildY = Math.min(minChildY, childY);
      maxChildY = Math.max(maxChildY, childY + childHeight);
    });

    const requiredWidth = maxChildX - minChildX + padding;
    const requiredHeight = maxChildY - minChildY + padding;

    const parentWidth = parent.width || 200;
    const parentHeight = parent.height || 200;

    // Check for violations
    const widthViolation = parentWidth < requiredWidth;
    const heightViolation = parentHeight < requiredHeight;

    if (widthViolation || heightViolation) {
      // Find the child that causes the violation
      const violatingChild =
        children.find((child) => {
          const childWidth = child.width || 100;
          const childHeight = child.height || 100;
          return (
            child.position.x + childWidth > parentWidth - MIN_PARENT_PADDING ||
            child.position.y + childHeight > parentHeight - MIN_PARENT_PADDING
          );
        }) || children[0];

      violations.push({
        parentId,
        childId: violatingChild.id,
        parentWidth,
        parentHeight,
        childWidth: violatingChild.width || 100,
        childHeight: violatingChild.height || 100,
        requiredWidth,
        requiredHeight,
        violation: widthViolation && heightViolation ? "both" : widthViolation ? "width" : "height",
      });
    }
  });

  return violations;
}

function getGrade(score: number): "A" | "B" | "C" | "D" | "F" {
  if (score >= 90) return "A";
  if (score >= 80) return "B";
  if (score >= 70) return "C";
  if (score >= 60) return "D";
  return "F";
}

/**
 * Generate quality report
 */
export function generateQualityReport(metrics: DiagramQualityMetrics, context?: string): string {
  const lines: string[] = [];

  if (context) {
    lines.push(`\n=== Diagram Quality Report: ${context} ===\n`);
  } else {
    lines.push("\n=== Diagram Quality Report ===\n");
  }

  lines.push(`Weighted Score: ${metrics.weightedScore.toFixed(1)}/100 (Grade: ${metrics.grade})`);
  lines.push(`Unweighted Score: ${metrics.overallScore.toFixed(1)}/100`);
  lines.push("");
  lines.push("Component Scores:");
  lines.push(
    `  Overlap Score: ${metrics.overlapScore.toFixed(1)}/100 (${metrics.overlappingNodes.length} overlaps)`
  );
  lines.push(
    `  Spacing Score: ${metrics.spacingScore.toFixed(1)}/100 (min: ${metrics.minSpacing.toFixed(1)}px, avg: ${metrics.averageSpacing.toFixed(1)}px)`
  );
  lines.push(
    `  Edge Crossings: ${metrics.edgeCrossings} (score: ${calculateEdgeCrossingScore(metrics.edgeCrossings, metrics.edgeCrossings + 1).toFixed(1)})`
  );
  lines.push(
    `  Edges Over Nodes: ${metrics.edgesOverNodes} (score: ${calculateEdgesOverNodesScore(metrics.edgesOverNodes, metrics.edgesOverNodes + 1).toFixed(1)})`
  );
  lines.push(
    `  Edge Bends: ${metrics.edgeBends} (score: ${calculateEdgeBendScore(metrics.edgeBends, metrics.edgeBends + 1).toFixed(1)})`
  );
  lines.push(
    `  Edge Length: min=${metrics.edgeLength.min.toFixed(0)}px, max=${metrics.edgeLength.max.toFixed(0)}px, avg=${metrics.edgeLength.average.toFixed(0)}px`
  );
  lines.push(`  Edge Score: ${metrics.edgeScore.toFixed(1)}/100`);
  lines.push(
    `  Hierarchy Score: ${metrics.hierarchyScore.toFixed(1)}/100 (${metrics.parentChildContainment.length} violations)`
  );
  lines.push(
    `  Viewport Score: ${metrics.viewportScore.toFixed(1)}/100 (${(metrics.viewportUtilization * 100).toFixed(1)}% utilization)`
  );
  lines.push(`  Consistency Score: ${metrics.consistencyScore.toFixed(1)}/100`);
  lines.push(
    `  Aspect Ratio: ${metrics.aspectRatio.toFixed(2)} (score: ${metrics.aspectRatioScore.toFixed(1)}/100)`
  );
  lines.push(
    `  Empty Space: ${(metrics.emptySpace * 100).toFixed(1)}% (score: ${metrics.emptySpaceScore.toFixed(1)}/100)`
  );
  lines.push(
    `  Edge Label Overlaps: ${metrics.edgeLabelOverlaps} (score: ${metrics.edgeLabelOverlapScore.toFixed(1)}/100)`
  );
  lines.push(
    `  Clipped Node Labels: ${metrics.clippedNodeLabels} (score: ${metrics.clippedLabelScore.toFixed(1)}/100)`
  );
  lines.push(`  Parent-Child Size Violations: ${metrics.parentChildSizeViolations.length}`);

  if (metrics.overlappingNodes.length > 0) {
    lines.push("\nOverlapping Nodes:");
    metrics.overlappingNodes.slice(0, 5).forEach((v) => {
      lines.push(`  - ${v.node1} & ${v.node2}: ${v.overlapPercentage.toFixed(1)}% overlap`);
    });
  }

  if (metrics.spacingViolations.length > 0) {
    lines.push("\nSpacing Violations:");
    metrics.spacingViolations.slice(0, 5).forEach((v) => {
      lines.push(
        `  - ${v.node1} & ${v.node2}: ${v.distance.toFixed(1)}px (min: ${v.minRequired}px)`
      );
    });
  }

  if (metrics.parentChildContainment.length > 0) {
    lines.push("\n🚨 CRITICAL: Parent-Child Containment Violations:");
    lines.push(
      `  ⚠️  ${metrics.parentChildContainment.length} child node(s) are outside their parent's bounding box!`
    );
    metrics.parentChildContainment.slice(0, 5).forEach((v) => {
      lines.push(`  - Child "${v.childId}" in parent "${v.parentId}": ${v.violation}`);
      lines.push(`    Details: ${v.details}`);
    });
    if (metrics.parentChildContainment.length > 5) {
      lines.push(`  ... and ${metrics.parentChildContainment.length - 5} more violations`);
    }
  }

  if (metrics.parentChildSizeViolations.length > 0) {
    lines.push("\nParent-Child Size Violations:");
    metrics.parentChildSizeViolations.slice(0, 5).forEach((v) => {
      lines.push(
        `  - Parent ${v.parentId}: ${v.parentWidth.toFixed(0)}x${v.parentHeight.toFixed(0)}px, ` +
          `required: ${v.requiredWidth.toFixed(0)}x${v.requiredHeight.toFixed(0)}px (${v.violation})`
      );
    });
  }

  if (
    metrics.aspectRatio < OPTIMAL_ASPECT_RATIO_MIN ||
    metrics.aspectRatio > OPTIMAL_ASPECT_RATIO_MAX
  ) {
    lines.push(
      `\n⚠️  Aspect Ratio Warning: ${metrics.aspectRatio.toFixed(2)} ` +
        `(optimal: ${OPTIMAL_ASPECT_RATIO_MIN}-${OPTIMAL_ASPECT_RATIO_MAX})`
    );
    if (metrics.aspectRatio < OPTIMAL_ASPECT_RATIO_MIN) {
      lines.push("   Diagram is too tall (vertically stretched)");
    } else {
      lines.push("   Diagram is too wide (horizontally stretched)");
    }
  }

  if (metrics.emptySpaceScore < 80) {
    const emptySpacePercent = (metrics.emptySpace * 100).toFixed(1);
    lines.push(
      `\n⚠️  Empty Space Warning: ${emptySpacePercent}% empty space (score: ${metrics.emptySpaceScore.toFixed(1)}/100)`
    );
    if (metrics.emptySpace > 0.8) {
      lines.push("   Diagram is too sparse (too much empty space)");
    } else if (metrics.emptySpace < 0.2) {
      lines.push("   Diagram is too crowded (too little empty space)");
    }
    lines.push(`   Optimal range: 20-80% empty space (40-60% density)`);
  }

  if (metrics.edgeLabelOverlaps > 0) {
    lines.push(
      `\n⚠️  Edge Label Overlaps: ${metrics.edgeLabelOverlaps} edge labels overlapping with node labels/content`
    );
  }

  if (metrics.clippedNodeLabels > 0) {
    lines.push(
      `\n⚠️  Clipped Node Labels: ${metrics.clippedNodeLabels} node labels cut off or not fully visible`
    );
  }

  lines.push("");
  return lines.join("\n");
}
