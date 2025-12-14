// packages/diagram/tests/utils/improvement-suggestions.ts
// Generate actionable improvement suggestions based on metrics
import type { LayoutMetrics } from "./metrics-collector";
import type { ComparisonResult } from "./comparison";

export interface ImprovementSuggestion {
  id: string;
  priority: "critical" | "high" | "medium" | "low";
  category: string;
  title: string;
  description: string;
  affectedExamples: string[];
  metrics: {
    current: number;
    target?: number;
    improvement?: number;
  };
  actionable: boolean;
  codeLocation?: string;
  relatedFiles?: string[];
}

/**
 * Generate improvement suggestions from metrics
 */
export function generateImprovementSuggestions(
  metrics: Map<string, LayoutMetrics>,
  comparisons?: ComparisonResult[]
): ImprovementSuggestion[] {
  const suggestions: ImprovementSuggestion[] = [
    ...analyzeOverlapIssues(metrics),
    ...analyzeSpacingIssues(metrics),
    ...analyzeEdgeIssues(metrics),
    ...analyzeHierarchyIssues(metrics),
    ...analyzeLabelIssues(metrics),
    ...analyzeLayoutSelection(metrics),
  ];

  // Sort by priority
  const priorityOrder = { critical: 0, high: 1, medium: 2, low: 3 };
  suggestions.sort((a, b) => priorityOrder[a.priority] - priorityOrder[b.priority]);

  return suggestions;
}

function analyzeOverlapIssues(metrics: Map<string, LayoutMetrics>): ImprovementSuggestion[] {
  const suggestions: ImprovementSuggestion[] = [];
  const examplesWithOverlaps: string[] = [];
  let maxOverlaps = 0;

  metrics.forEach((m, name) => {
    if (m.overlappingNodes > 0) {
      examplesWithOverlaps.push(name);
      maxOverlaps = Math.max(maxOverlaps, m.overlappingNodes);
    }
  });

  if (examplesWithOverlaps.length > 0) {
    const avgOverlaps =
      examplesWithOverlaps.reduce(
        (sum, name) => sum + (metrics.get(name)?.overlappingNodes || 0),
        0
      ) / examplesWithOverlaps.length;

    suggestions.push({
      id: "overlap-detection",
      priority: maxOverlaps > 5 ? "critical" : "high",
      category: "Node Positioning",
      title: "Eliminate Node Overlaps",
      description: `${examplesWithOverlaps.length} example(s) have overlapping nodes (avg: ${avgOverlaps.toFixed(1)} overlaps). Overlaps break visual hierarchy and readability.`,
      affectedExamples: examplesWithOverlaps,
      metrics: {
        current: avgOverlaps,
        target: 0,
        improvement: avgOverlaps,
      },
      actionable: true,
      codeLocation: "packages/layout/src/algorithms/overlap.ts",
      relatedFiles: [
        "packages/layout/src/algorithms/coordinates.ts",
        "packages/diagram/src/utils/srujaLayoutEngine.ts",
      ],
    });
  }

  return suggestions;
}

function analyzeSpacingIssues(metrics: Map<string, LayoutMetrics>): ImprovementSuggestion[] {
  const suggestions: ImprovementSuggestion[] = [];
  const examplesWithSpacingViolations: string[] = [];
  let totalViolations = 0;

  metrics.forEach((m, name) => {
    if (m.spacingViolations > 0) {
      examplesWithSpacingViolations.push(name);
      totalViolations += m.spacingViolations;
    }
  });

  if (examplesWithSpacingViolations.length > 0) {
    const avgViolations = totalViolations / examplesWithSpacingViolations.length;

    suggestions.push({
      id: "spacing-violations",
      priority: avgViolations > 10 ? "high" : "medium",
      category: "Node Spacing",
      title: "Improve Node Spacing",
      description: `${examplesWithSpacingViolations.length} example(s) have spacing violations (avg: ${avgViolations.toFixed(1)}). Increase minimum spacing or adjust layout algorithm.`,
      affectedExamples: examplesWithSpacingViolations,
      metrics: {
        current: avgViolations,
        target: 0,
        improvement: avgViolations,
      },
      actionable: true,
      codeLocation: "packages/layout/src/algorithms/sizing.ts",
      relatedFiles: [
        "packages/layout/src/presets/default.ts",
        "packages/diagram/src/utils/layoutRules.ts",
      ],
    });
  }

  return suggestions;
}

function createEdgeCrossingSuggestion(
  examples: string[],
  avgCrossings: number
): ImprovementSuggestion {
  return {
    id: "edge-crossings",
    priority: avgCrossings > 5 ? "high" : "medium",
    category: "Edge Routing",
    title: "Reduce Edge Crossings",
    description: `${examples.length} example(s) have edge crossings (avg: ${avgCrossings.toFixed(1)}). Improve edge routing algorithm or adjust node positioning.`,
    affectedExamples: examples,
    metrics: { current: avgCrossings, target: 0, improvement: avgCrossings },
    actionable: true,
    codeLocation: "packages/layout/src/algorithms/edge-router.ts",
    relatedFiles: [
      "packages/layout/src/algorithms/unified-router.ts",
      "packages/layout/src/algorithms/edge-bundler.ts",
    ],
  };
}

function createEdgesOverNodesSuggestion(
  examples: string[],
  avgEdgesOverNodes: number
): ImprovementSuggestion {
  return {
    id: "edges-over-nodes",
    priority: avgEdgesOverNodes > 3 ? "high" : "medium",
    category: "Edge Routing",
    title: "Route Edges Around Nodes",
    description: `${examples.length} example(s) have edges passing through nodes (avg: ${avgEdgesOverNodes.toFixed(1)}). Improve edge routing to avoid node intersections.`,
    affectedExamples: examples,
    metrics: { current: avgEdgesOverNodes, target: 0, improvement: avgEdgesOverNodes },
    actionable: true,
    codeLocation: "packages/layout/src/algorithms/edge-router.ts",
    relatedFiles: ["packages/layout/src/algorithms/unified-router.ts"],
  };
}

function analyzeEdgeIssues(metrics: Map<string, LayoutMetrics>): ImprovementSuggestion[] {
  const examplesWithCrossings: string[] = [];
  const examplesWithEdgesOverNodes: string[] = [];

  metrics.forEach((m, name) => {
    if (m.edgeCrossings > 0) examplesWithCrossings.push(name);
    if (m.edgesOverNodes > 0) examplesWithEdgesOverNodes.push(name);
  });

  const suggestions: ImprovementSuggestion[] = [];

  if (examplesWithCrossings.length > 0) {
    const avgCrossings =
      examplesWithCrossings.reduce(
        (sum, name) => sum + (metrics.get(name)?.edgeCrossings || 0),
        0
      ) / examplesWithCrossings.length;
    suggestions.push(createEdgeCrossingSuggestion(examplesWithCrossings, avgCrossings));
  }

  if (examplesWithEdgesOverNodes.length > 0) {
    const avgEdgesOverNodes =
      examplesWithEdgesOverNodes.reduce(
        (sum, name) => sum + (metrics.get(name)?.edgesOverNodes || 0),
        0
      ) / examplesWithEdgesOverNodes.length;
    suggestions.push(createEdgesOverNodesSuggestion(examplesWithEdgesOverNodes, avgEdgesOverNodes));
  }

  return suggestions;
}

function createContainmentSuggestion(
  examplesWithViolations: string[]
): ImprovementSuggestion | null {
  if (examplesWithViolations.length === 0) return null;

  return {
    id: "hierarchy-containment",
    priority: "critical",
    category: "Hierarchy",
    title: "Fix Parent-Child Containment",
    description: `${examplesWithViolations.length} example(s) have children outside parent bounds. This is a critical layout failure.`,
    affectedExamples: examplesWithViolations,
    metrics: {
      current: examplesWithViolations.length,
      target: 0,
      improvement: examplesWithViolations.length,
    },
    actionable: true,
    codeLocation: "packages/layout/src/algorithms/hierarchy.ts",
    relatedFiles: [
      "packages/layout/src/algorithms/sizing.ts",
      "packages/layout/src/algorithms/coordinates.ts",
    ],
  };
}

function analyzeHierarchyIssues(metrics: Map<string, LayoutMetrics>): ImprovementSuggestion[] {
  const examplesWithContainmentViolations: string[] = [];

  metrics.forEach((m, name) => {
    if (m.parentChildViolations > 0) {
      examplesWithContainmentViolations.push(name);
    }
  });

  const suggestion = createContainmentSuggestion(examplesWithContainmentViolations);
  return suggestion ? [suggestion] : [];
}

function analyzeLabelIssues(metrics: Map<string, LayoutMetrics>): ImprovementSuggestion[] {
  const suggestions: ImprovementSuggestion[] = [];
  const examplesWithLabelOverlaps: string[] = [];
  const examplesWithClippedLabels: string[] = [];

  metrics.forEach((m, name) => {
    if (m.qualityMetrics?.edgeLabelOverlaps > 0) {
      examplesWithLabelOverlaps.push(name);
    }
    if (m.qualityMetrics?.clippedNodeLabels > 0) {
      examplesWithClippedLabels.push(name);
    }
  });

  if (examplesWithLabelOverlaps.length > 0) {
    suggestions.push({
      id: "label-overlaps",
      priority: "medium",
      category: "Label Positioning",
      title: "Fix Edge Label Overlaps",
      description: `${examplesWithLabelOverlaps.length} example(s) have edge labels overlapping with nodes. Improve label placement algorithm.`,
      affectedExamples: examplesWithLabelOverlaps,
      metrics: {
        current: examplesWithLabelOverlaps.length,
        target: 0,
      },
      actionable: true,
      codeLocation: "packages/layout/src/algorithms/label-placer.ts",
      relatedFiles: ["packages/diagram/src/components/edges/RelationEdge.tsx"],
    });
  }

  if (examplesWithClippedLabels.length > 0) {
    suggestions.push({
      id: "clipped-labels",
      priority: "medium",
      category: "Label Positioning",
      title: "Fix Clipped Node Labels",
      description: `${examplesWithClippedLabels.length} example(s) have clipped node labels. Increase node size or improve text wrapping.`,
      affectedExamples: examplesWithClippedLabels,
      metrics: {
        current: examplesWithClippedLabels.length,
        target: 0,
      },
      actionable: true,
      codeLocation: "packages/layout/src/algorithms/sizing.ts",
      relatedFiles: [
        "packages/layout/src/text/TextMeasurer.ts",
        "packages/diagram/src/components/nodes",
      ],
    });
  }

  return suggestions;
}

function analyzeLayoutSelection(metrics: Map<string, LayoutMetrics>): ImprovementSuggestion[] {
  const suggestions: ImprovementSuggestion[] = [];
  const engineUsage = new Map<string, number>();
  const directionUsage = new Map<string, number>();
  const lowScores: string[] = [];

  metrics.forEach((m, name) => {
    engineUsage.set(m.selectedEngine, (engineUsage.get(m.selectedEngine) || 0) + 1);
    directionUsage.set(m.selectedDirection, (directionUsage.get(m.selectedDirection) || 0) + 1);
    if (m.weightedScore < 70) {
      lowScores.push(name);
    }
  });

  if (lowScores.length > 0) {
    const avgScore =
      lowScores.reduce((sum, name) => sum + (metrics.get(name)?.weightedScore || 0), 0) /
      lowScores.length;

    suggestions.push({
      id: "layout-selection",
      priority: avgScore < 60 ? "high" : "medium",
      category: "Layout Selection",
      title: "Improve Layout Rule Selection",
      description: `${lowScores.length} example(s) have low quality scores (avg: ${avgScore.toFixed(1)}). Review layout rule priorities and conditions.`,
      affectedExamples: lowScores,
      metrics: {
        current: avgScore,
        target: 80,
        improvement: 80 - avgScore,
      },
      actionable: true,
      codeLocation: "packages/diagram/src/utils/layoutRules.ts",
      relatedFiles: ["packages/diagram/src/utils/srujaLayoutEngine.ts"],
    });
  }

  return suggestions;
}

/**
 * Generate markdown report of suggestions
 */
export function generateSuggestionsReport(suggestions: ImprovementSuggestion[]): string {
  let report = `# Layout Improvement Suggestions\n\n`;
  report += `Generated: ${new Date().toISOString()}\n\n`;
  report += `Total Suggestions: ${suggestions.length}\n\n`;

  const byPriority = {
    critical: suggestions.filter((s) => s.priority === "critical"),
    high: suggestions.filter((s) => s.priority === "high"),
    medium: suggestions.filter((s) => s.priority === "medium"),
    low: suggestions.filter((s) => s.priority === "low"),
  };

  for (const [priority, items] of Object.entries(byPriority)) {
    if (items.length === 0) continue;

    report += `## ${priority.toUpperCase()} Priority (${items.length})\n\n`;

    items.forEach((suggestion, i) => {
      report += `### ${i + 1}. ${suggestion.title}\n\n`;
      report += `**Category**: ${suggestion.category}\n\n`;
      report += `${suggestion.description}\n\n`;

      if (suggestion.metrics.improvement) {
        report += `**Expected Improvement**: ${suggestion.metrics.improvement.toFixed(1)} points\n\n`;
      }

      if (suggestion.affectedExamples.length > 0) {
        report += `**Affected Examples** (${suggestion.affectedExamples.length}):\n`;
        suggestion.affectedExamples.slice(0, 10).forEach((ex) => {
          report += `- ${ex}\n`;
        });
        if (suggestion.affectedExamples.length > 10) {
          report += `- ... and ${suggestion.affectedExamples.length - 10} more\n`;
        }
        report += `\n`;
      }

      if (suggestion.codeLocation) {
        report += `**Code Location**: \`${suggestion.codeLocation}\`\n\n`;
      }

      if (suggestion.relatedFiles && suggestion.relatedFiles.length > 0) {
        report += `**Related Files**:\n`;
        suggestion.relatedFiles.forEach((file) => {
          report += `- \`${file}\`\n`;
        });
        report += `\n`;
      }

      report += `---\n\n`;
    });
  }

  return report;
}
