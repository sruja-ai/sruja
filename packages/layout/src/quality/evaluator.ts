/**
 * Quality Evaluator - Calculates layout quality scores and violations
 * Implements comprehensive quality metrics for C4 diagrams
 */

import type {
  LayoutContext,
  QualityScore,
  QualityMetric,
  QualityViolation,
  QualityRecommendation,
  LayoutNode,
  LayoutEdge,
  Point,
  QualityOptions,
} from "../core/types";
import {
  classifyC4Node,
  type C4SemanticTier,
} from "../algorithms/c4-classifier";
import { MIN_PARENT_PADDING, SEMANTIC_TIER_SPACING } from "../constants";

export function createQualityEvaluator(options: QualityOptions) {
  return (context: LayoutContext): QualityScore => {
    const metrics = calculateQualityMetrics(context);
    const violations = detectViolations(context);
    const recommendations = generateRecommendations(violations, context);
    const score = calculateOverallScore(metrics, violations, options);

    return {
      grade: getGrade(score),
      score,
      metrics,
      violations,
      recommendations,
    };
  };
}

function calculateQualityMetrics(context: LayoutContext): Map<string, QualityMetric> {
  const metrics = new Map<string, QualityMetric>();
  const nodes = Array.from(context.nodes.values());
  const edges = Array.from(context.edges.values());

  // Calculate individual metrics
  metrics.set("edgeCrossings", calculateEdgeCrossings(edges));
  metrics.set("nodeOverlaps", calculateNodeOverlaps(nodes));
  metrics.set("containmentViolations", calculateContainmentViolations(nodes));
  metrics.set("edgeLength", calculateEdgeLength(edges));
  metrics.set("edgeBends", calculateEdgeBends(edges));
  metrics.set("spacing", calculateSpacing(nodes));
  metrics.set("aspectRatio", calculateAspectRatio(nodes));
  metrics.set("viewportUtilization", calculateViewportUtilization(nodes, context.options));
  metrics.set("labelClipping", calculateLabelClipping(nodes));
  metrics.set("portCongestion", calculatePortCongestion(nodes, edges));

  // C4-specific quality metrics
  metrics.set("semanticPositioning", calculateSemanticPositioning(nodes));
  metrics.set("tierSeparation", calculateTierSeparation(nodes));
  metrics.set("boundaryClarity", calculateBoundaryClarity(nodes));

  return metrics;
}

function calculateEdgeCrossings(edges: readonly LayoutEdge[]): QualityMetric {
  let crossings = 0;
  for (let i = 0; i < edges.length; i++) {
    for (let j = i + 1; j < edges.length; j++) {
      if (edgesCross(edges[i], edges[j])) {
        crossings++;
      }
    }
  }

  const target = Math.max(0, Math.floor(edges.length * 0.1)); // Allow 10% of max possible crossings
  const value = crossings;
  const weight = 0.18; // High importance

  return {
    name: "Edge Crossings",
    value,
    weight,
    target,
    achieved: value <= target,
    impact: "important",
  };
}

function calculateNodeOverlaps(nodes: readonly LayoutNode[]): QualityMetric {
  let overlaps = 0;
  for (let i = 0; i < nodes.length; i++) {
    for (let j = i + 1; j < nodes.length; j++) {
      if (nodesOverlap(nodes[i], nodes[j])) {
        overlaps++;
      }
    }
  }

  return {
    name: "Node Overlaps",
    value: overlaps,
    weight: 0.25,
    target: 0,
    achieved: overlaps === 0,
    impact: "critical",
  };
}

function calculateContainmentViolations(nodes: readonly LayoutNode[]): QualityMetric {
  let violations = 0;
  for (const node of nodes) {
    if (!node.parent) continue;

    const parentBounds = node.parent.bbox;
    const childBounds = node.bbox;
    const padding = 50; // Minimum padding

    if (
      childBounds.x < parentBounds.x + padding ||
      childBounds.y < parentBounds.y + padding ||
      childBounds.x + childBounds.width > parentBounds.x + parentBounds.width - padding ||
      childBounds.y + childBounds.height > parentBounds.y + parentBounds.height - padding
    ) {
      violations++;
    }
  }

  return {
    name: "Containment Violations",
    value: violations,
    weight: 0.2,
    target: 0,
    achieved: violations === 0,
    impact: "critical",
  };
}

function calculateEdgeLength(edges: readonly LayoutEdge[]): QualityMetric {
  if (edges.length === 0) {
    return {
      name: "Edge Length",
      value: 0,
      weight: 0.08,
      target: 200,
      achieved: true,
      impact: "aesthetic",
    };
  }

  const totalLength = edges.reduce((sum, edge) => sum + edge.length, 0);
  const avgLength = totalLength / edges.length;

  return {
    name: "Edge Length",
    value: avgLength,
    weight: 0.08,
    target: 200,
    achieved: avgLength >= 100 && avgLength <= 300,
    impact: "aesthetic",
  };
}

function calculateEdgeBends(edges: readonly LayoutEdge[]): QualityMetric {
  if (edges.length === 0) {
    return {
      name: "Edge Bends",
      value: 0,
      weight: 0.06,
      target: 2,
      achieved: true,
      impact: "aesthetic",
    };
  }

  const totalBends = edges.reduce((sum, edge) => sum + edge.bendCount, 0);
  const avgBends = totalBends / edges.length;

  return {
    name: "Edge Bends",
    value: avgBends,
    weight: 0.06,
    target: 2,
    achieved: avgBends <= 3,
    impact: "aesthetic",
  };
}

function calculateSpacing(nodes: readonly LayoutNode[]): QualityMetric {
  if (nodes.length < 2) {
    return {
      name: "Node Spacing",
      value: 50,
      weight: 0.1,
      target: 30,
      achieved: true,
      impact: "important",
    };
  }

  let minSpacing = Infinity;
  let totalSpacing = 0;
  let pairs = 0;

  for (let i = 0; i < nodes.length; i++) {
    for (let j = i + 1; j < nodes.length; j++) {
      // Skip parent-child relationships
      if (nodes[i].parent?.id === nodes[j].id || nodes[j].parent?.id === nodes[i].id) continue;

      const spacing = calculateNodeSpacing(nodes[i], nodes[j]);
      minSpacing = Math.min(minSpacing, spacing);
      totalSpacing += spacing;
      pairs++;
    }
  }

  return {
    name: "Node Spacing",
    value: minSpacing,
    weight: 0.1,
    target: 30,
    achieved: minSpacing >= 30,
    impact: "important",
  };
}

function calculateAspectRatio(nodes: readonly LayoutNode[]): QualityMetric {
  if (nodes.length === 0) {
    return {
      name: "Aspect Ratio",
      value: 1,
      weight: 0.05,
      target: 1.5,
      achieved: true,
      impact: "aesthetic",
    };
  }

  let minX = Infinity,
    minY = Infinity,
    maxX = -Infinity,
    maxY = -Infinity;
  for (const node of nodes) {
    minX = Math.min(minX, node.bbox.x);
    minY = Math.min(minY, node.bbox.y);
    maxX = Math.max(maxX, node.bbox.x + node.bbox.width);
    maxY = Math.max(maxY, node.bbox.y + node.bbox.height);
  }

  const width = maxX - minX;
  const height = maxY - minY;
  const aspectRatio = width / height || 1;

  return {
    name: "Aspect Ratio",
    value: aspectRatio,
    weight: 0.05,
    target: 1.5,
    achieved: aspectRatio >= 0.5 && aspectRatio <= 2.0,
    impact: "aesthetic",
  };
}

function calculateViewportUtilization(nodes: readonly LayoutNode[], _options: any): QualityMetric {
  if (nodes.length === 0) {
    return {
      name: "Viewport Utilization",
      value: 0,
      weight: 0.04,
      target: 0.7,
      achieved: false,
      impact: "aesthetic",
    };
  }

  let minX = Infinity,
    minY = Infinity,
    maxX = -Infinity,
    maxY = -Infinity;
  for (const node of nodes) {
    minX = Math.min(minX, node.bbox.x);
    minY = Math.min(minY, node.bbox.y);
    maxX = Math.max(maxX, node.bbox.x + node.bbox.width);
    maxY = Math.max(maxY, node.bbox.y + node.bbox.height);
  }

  const diagramArea = (maxX - minX) * (maxY - minY);
  const viewportArea = 1920 * 1080; // Default viewport
  const utilization = diagramArea / viewportArea;

  return {
    name: "Viewport Utilization",
    value: utilization,
    weight: 0.04,
    target: 0.7,
    achieved: utilization >= 0.5 && utilization <= 0.9,
    impact: "aesthetic",
  };
}

function calculateLabelClipping(nodes: readonly LayoutNode[]): QualityMetric {
  let clipped = 0;
  for (const node of nodes) {
    // Simple check - in real implementation would check actual text rendering
    if (node.bbox.width < 100 || node.bbox.height < 40) {
      clipped++;
    }
  }

  return {
    name: "Label Clipping",
    value: clipped,
    weight: 0.1,
    target: 0,
    achieved: clipped === 0,
    impact: "important",
  };
}

function calculatePortCongestion(
  _nodes: readonly LayoutNode[],
  edges: readonly LayoutEdge[]
): QualityMetric {
  const portUsage = new Map<string, number>();

  for (const edge of edges) {
    if (edge.sourcePort) {
      const key = `${edge.source.id}:${edge.sourcePort.side}`;
      portUsage.set(key, (portUsage.get(key) || 0) + 1);
    }
    if (edge.targetPort) {
      const key = `${edge.target.id}:${edge.targetPort.side}`;
      portUsage.set(key, (portUsage.get(key) || 0) + 1);
    }
  }

  let congestedPorts = 0;
  for (const usage of portUsage.values()) {
    if (usage > 3) congestedPorts++;
  }

  return {
    name: "Port Congestion",
    value: congestedPorts,
    weight: 0.04,
    target: 0,
    achieved: congestedPorts === 0,
    impact: "aesthetic",
  };
}

/**
 * Semantic Positioning - Measures % of nodes in correct C4 semantic zones
 * - Actors/Persons should be at the top (lowest Y)
 * - Data nodes (DataStore, Queue, Cache) should be at the bottom (highest Y)
 * - Presentation nodes should be above logic nodes
 * - Logic nodes should be above data nodes
 */
function calculateSemanticPositioning(nodes: readonly LayoutNode[]): QualityMetric {
  if (nodes.length < 2) {
    return {
      name: "Semantic Positioning",
      value: 100,
      weight: 0.12,
      target: 80,
      achieved: true,
      impact: "important",
    };
  }

  const rootNodes = nodes.filter((n) => !n.parent && n.visible);
  if (rootNodes.length < 2) {
    return {
      name: "Semantic Positioning",
      value: 100,
      weight: 0.12,
      target: 80,
      achieved: true,
      impact: "important",
    };
  }

  const classified = rootNodes.map((node) => ({
    node,
    classification: classifyC4Node({ id: node.id, original: node.original as any }),
    centerY: node.bbox.y + node.bbox.height / 2,
  }));

  const actors = classified.filter((c) => c.classification.tier === "actor");
  const dataNodes = classified.filter((c) => c.classification.tier === "data");
  const presentationNodes = classified.filter((c) => c.classification.tier === "presentation");
  const logicNodes = classified.filter((c) => c.classification.tier === "logic");

  let correctPositions = 0;
  let totalChecks = 0;

  const avgActorY = actors.length > 0 ? actors.reduce((s, a) => s + a.centerY, 0) / actors.length : Infinity;
  const avgDataY = dataNodes.length > 0 ? dataNodes.reduce((s, d) => s + d.centerY, 0) / dataNodes.length : -Infinity;
  const avgPresentationY = presentationNodes.length > 0 ? presentationNodes.reduce((s, p) => s + p.centerY, 0) / presentationNodes.length : Infinity;
  const avgLogicY = logicNodes.length > 0 ? logicNodes.reduce((s, l) => s + l.centerY, 0) / logicNodes.length : 0;

  if (actors.length > 0 && (dataNodes.length > 0 || logicNodes.length > 0)) {
    totalChecks++;
    const otherAvgY = dataNodes.length > 0 ? avgDataY : avgLogicY;
    if (avgActorY < otherAvgY) correctPositions++;
  }

  if (dataNodes.length > 0 && (actors.length > 0 || logicNodes.length > 0)) {
    totalChecks++;
    const otherAvgY = actors.length > 0 ? avgActorY : avgLogicY;
    if (avgDataY > otherAvgY) correctPositions++;
  }

  if (presentationNodes.length > 0 && logicNodes.length > 0) {
    totalChecks++;
    if (avgPresentationY < avgLogicY) correctPositions++;
  }

  if (presentationNodes.length > 0 && dataNodes.length > 0) {
    totalChecks++;
    if (avgPresentationY < avgDataY) correctPositions++;
  }

  if (logicNodes.length > 0 && dataNodes.length > 0) {
    totalChecks++;
    if (avgLogicY < avgDataY) correctPositions++;
  }

  const percentCorrect = totalChecks > 0 ? (correctPositions / totalChecks) * 100 : 100;

  return {
    name: "Semantic Positioning",
    value: percentCorrect,
    weight: 0.12,
    target: 80,
    achieved: percentCorrect >= 80,
    impact: "important",
  };
}

/**
 * Tier Separation - Measures visual gap between presentation/logic/data tiers
 * Target: >150px gap between tiers for clear visual hierarchy
 */
function calculateTierSeparation(nodes: readonly LayoutNode[]): QualityMetric {
  const TARGET_SEPARATION = SEMANTIC_TIER_SPACING > 0 ? Math.min(150, SEMANTIC_TIER_SPACING) : 150;

  if (nodes.length < 2) {
    return {
      name: "Tier Separation",
      value: TARGET_SEPARATION,
      weight: 0.08,
      target: TARGET_SEPARATION,
      achieved: true,
      impact: "important",
    };
  }

  const rootNodes = nodes.filter((n) => !n.parent && n.visible);
  if (rootNodes.length < 2) {
    return {
      name: "Tier Separation",
      value: TARGET_SEPARATION,
      weight: 0.08,
      target: TARGET_SEPARATION,
      achieved: true,
      impact: "important",
    };
  }

  const tierBounds = new Map<C4SemanticTier, { minY: number; maxY: number }>();

  for (const node of rootNodes) {
    const classification = classifyC4Node({ id: node.id, original: node.original as any });
    const tier = classification.tier;
    const nodeTop = node.bbox.y;
    const nodeBottom = node.bbox.y + node.bbox.height;

    const current = tierBounds.get(tier);
    if (current) {
      tierBounds.set(tier, {
        minY: Math.min(current.minY, nodeTop),
        maxY: Math.max(current.maxY, nodeBottom),
      });
    } else {
      tierBounds.set(tier, { minY: nodeTop, maxY: nodeBottom });
    }
  }

  const orderedTiers: C4SemanticTier[] = ["actor", "presentation", "logic", "data", "external"];
  const presentTiers = orderedTiers.filter((t) => tierBounds.has(t));

  if (presentTiers.length < 2) {
    return {
      name: "Tier Separation",
      value: TARGET_SEPARATION,
      weight: 0.08,
      target: TARGET_SEPARATION,
      achieved: true,
      impact: "important",
    };
  }

  let minSeparation = Infinity;

  for (let i = 0; i < presentTiers.length - 1; i++) {
    const currentTier = tierBounds.get(presentTiers[i])!;
    const nextTier = tierBounds.get(presentTiers[i + 1])!;

    const separation = Math.abs(nextTier.minY - currentTier.maxY);
    minSeparation = Math.min(minSeparation, separation);
  }

  return {
    name: "Tier Separation",
    value: minSeparation === Infinity ? TARGET_SEPARATION : minSeparation,
    weight: 0.08,
    target: TARGET_SEPARATION,
    achieved: minSeparation >= TARGET_SEPARATION,
    impact: "important",
  };
}

/**
 * Boundary Clarity - Measures distance between parent edge and child nodes
 * Target: >80px padding for clear visual containment
 */
function calculateBoundaryClarity(nodes: readonly LayoutNode[]): QualityMetric {
  const TARGET_PADDING = MIN_PARENT_PADDING > 0 ? MIN_PARENT_PADDING : 80;

  const childNodes = nodes.filter((n) => n.parent && n.visible);

  if (childNodes.length === 0) {
    return {
      name: "Boundary Clarity",
      value: TARGET_PADDING,
      weight: 0.1,
      target: TARGET_PADDING,
      achieved: true,
      impact: "important",
    };
  }

  let minPadding = Infinity;

  for (const child of childNodes) {
    const parent = child.parent;
    if (!parent) continue;

    const leftPadding = child.bbox.x - parent.bbox.x;
    const topPadding = child.bbox.y - parent.bbox.y;
    const rightPadding = (parent.bbox.x + parent.bbox.width) - (child.bbox.x + child.bbox.width);
    const bottomPadding = (parent.bbox.y + parent.bbox.height) - (child.bbox.y + child.bbox.height);

    const minChildPadding = Math.min(leftPadding, topPadding, rightPadding, bottomPadding);
    minPadding = Math.min(minPadding, minChildPadding);
  }

  return {
    name: "Boundary Clarity",
    value: minPadding === Infinity ? TARGET_PADDING : minPadding,
    weight: 0.1,
    target: TARGET_PADDING,
    achieved: minPadding >= TARGET_PADDING,
    impact: "important",
  };
}

function detectViolations(context: LayoutContext): QualityViolation[] {
  const violations: QualityViolation[] = [];
  const { nodes } = context;

  // Convert Maps to arrays for iteration
  const nodeArray = Array.from(nodes.values());

  // Detect node overlaps
  for (let i = 0; i < nodeArray.length; i++) {
    for (let j = i + 1; j < nodeArray.length; j++) {
      if (nodesOverlap(nodeArray[i], nodeArray[j])) {
        violations.push({
          type: "node-overlap",
          severity: "critical",
          description: `Nodes '${nodeArray[i].id}' and '${nodeArray[j].id}' overlap`,
          affectedNodes: [nodeArray[i].id, nodeArray[j].id],
          affectedEdges: [],
          autoFixable: true,
          penalty: 25,
        });
      }
    }
  }

  // Detect containment violations
  for (const node of nodeArray) {
    if (!node.parent) continue;

    const parentBounds = node.parent.bbox;
    const childBounds = node.bbox;
    const padding = 50;

    if (
      childBounds.x < parentBounds.x + padding ||
      childBounds.y < parentBounds.y + padding ||
      childBounds.x + childBounds.width > parentBounds.x + parentBounds.width - padding ||
      childBounds.y + childBounds.height > parentBounds.y + parentBounds.height - padding
    ) {
      violations.push({
        type: "containment-violation",
        severity: "critical",
        description: `Node '${node.id}' violates containment in parent '${node.parent.id}'`,
        affectedNodes: [node.id, node.parent.id],
        affectedEdges: [],
        autoFixable: true,
        penalty: 30,
      });
    }
  }

  return violations;
}

function generateRecommendations(
  violations: readonly QualityViolation[],
  _context: LayoutContext
): readonly QualityRecommendation[] {
  const recommendations: QualityRecommendation[] = [];

  if (violations.some((v) => v.type === "node-overlap")) {
    recommendations.push({
      action: "run-overlap-removal",
      description: "Run overlap removal phase to eliminate node overlaps",
      expectedImprovement: 25,
      effort: "low",
      applicablePhases: ["overlap-removal"],
    });
  }

  if (violations.some((v) => v.type === "containment-violation")) {
    recommendations.push({
      action: "enforce-containment",
      description: "Run containment enforcement to ensure children stay within parents",
      expectedImprovement: 30,
      effort: "low",
      applicablePhases: ["containment-enforcement"],
    });
  }

  const crossingViolations = violations.filter((v) => v.type === "edge-crossing");
  if (crossingViolations.length > 0) {
    recommendations.push({
      action: "optimize-edge-routing",
      description: "Run edge routing optimization to reduce crossings",
      expectedImprovement: Math.min(20, crossingViolations.length * 2),
      effort: "medium",
      applicablePhases: ["crossing-minimization", "edge-routing"],
    });
  }

  if (violations.some((v) => v.type === "insufficient-spacing")) {
    recommendations.push({
      action: "increase-tier-spacing",
      description: "Increase spacing between tiers for better visual clarity",
      expectedImprovement: 10,
      effort: "low",
      applicablePhases: ["spacing-optimization"],
    });
  }

  return recommendations;
}

function calculateOverallScore(
  metrics: Map<string, QualityMetric>,
  violations: readonly QualityViolation[],
  _options: QualityOptions
): number {
  let score = 100;

  // C4-specific metrics that get higher weight in scoring
  const c4MetricNames = new Set(["semanticPositioning", "tierSeparation", "boundaryClarity"]);

  // Apply metric penalties
  for (const [name, metric] of metrics) {
    if (!metric.achieved) {
      // C4 metrics get 1.5x penalty multiplier for being more important
      const c4Multiplier = c4MetricNames.has(name) ? 1.5 : 1.0;
      const penalty = ((metric.target - metric.value) / Math.abs(metric.target || 1)) * metric.weight * 100 * c4Multiplier;
      score -= Math.max(0, Math.min(penalty, 20)); // Cap individual penalties at 20
    }
  }

  // Bonus for achieving C4-specific metrics (up to +10 points)
  const c4Bonus = calculateC4Bonus(metrics, c4MetricNames);
  score += c4Bonus;

  // Apply violation penalties
  for (const violation of violations) {
    score -= violation.penalty;
  }

  // Apply caps based on critical violations
  const criticalViolations = violations.filter((v) => v.severity === "critical");
  if (criticalViolations.length > 0) {
    score = Math.min(score, 60); // Cap at D grade with any critical violation
  }

  return Math.max(0, Math.min(100, score));
}

/**
 * Calculate bonus points for achieving C4-specific quality metrics
 */
function calculateC4Bonus(
  metrics: Map<string, QualityMetric>,
  c4MetricNames: Set<string>
): number {
  let achievedCount = 0;
  let totalC4Metrics = 0;

  for (const [name, metric] of metrics) {
    if (c4MetricNames.has(name)) {
      totalC4Metrics++;
      if (metric.achieved) {
        achievedCount++;
      }
    }
  }

  if (totalC4Metrics === 0) return 0;

  // Award up to 10 bonus points for achieving all C4 metrics
  const achievementRatio = achievedCount / totalC4Metrics;
  return Math.round(achievementRatio * 10);
}

function getGrade(score: number): "A" | "B" | "C" | "D" | "F" {
  if (score >= 90) return "A";
  if (score >= 80) return "B";
  if (score >= 70) return "C";
  if (score >= 60) return "D";
  return "F";
}

// Helper functions
function edgesCross(edge1: LayoutEdge, edge2: LayoutEdge): boolean {
  if (
    edge1.source.id === edge2.source.id ||
    edge1.source.id === edge2.target.id ||
    edge1.target.id === edge2.source.id ||
    edge1.target.id === edge2.target.id
  ) {
    return false; // Connected edges don't count as crossings
  }

  for (let i = 0; i < edge1.points.length - 1; i++) {
    for (let j = 0; j < edge2.points.length - 1; j++) {
      if (
        lineSegmentsIntersect(
          edge1.points[i],
          edge1.points[i + 1],
          edge2.points[j],
          edge2.points[j + 1]
        )
      ) {
        return true;
      }
    }
  }
  return false;
}

function nodesOverlap(node1: LayoutNode, node2: LayoutNode): boolean {
  return !(
    node1.bbox.x + node1.bbox.width <= node2.bbox.x ||
    node2.bbox.x + node2.bbox.width <= node1.bbox.x ||
    node1.bbox.y + node1.bbox.height <= node2.bbox.y ||
    node2.bbox.y + node2.bbox.height <= node1.bbox.y
  );
}

function calculateNodeSpacing(node1: LayoutNode, node2: LayoutNode): number {
  const center1 = {
    x: node1.bbox.x + node1.bbox.width / 2,
    y: node1.bbox.y + node1.bbox.height / 2,
  };
  const center2 = {
    x: node2.bbox.x + node2.bbox.width / 2,
    y: node2.bbox.y + node2.bbox.height / 2,
  };

  const dist = Math.hypot(center1.x - center2.x, center1.y - center2.y);
  const avgRadius =
    (Math.max(node1.bbox.width, node1.bbox.height) +
      Math.max(node2.bbox.width, node2.bbox.height)) /
    4;

  return Math.max(0, dist - avgRadius);
}

function lineSegmentsIntersect(p1: Point, p2: Point, p3: Point, p4: Point): boolean {
  const det = (p2.x - p1.x) * (p4.y - p3.y) - (p4.x - p3.x) * (p2.y - p1.y);
  if (Math.abs(det) < 1e-10) return false;

  const lambda = ((p4.y - p3.y) * (p4.x - p1.x) + (p3.x - p4.x) * (p4.y - p1.y)) / det;
  const gamma = ((p1.y - p2.y) * (p4.x - p1.x) + (p2.x - p1.x) * (p4.y - p1.y)) / det;

  return 0 < lambda && lambda < 1 && 0 < gamma && gamma < 1;
}
