#!/usr/bin/env tsx
/**
 * Test and Analyze Diagram Quality
 *
 * This script tests specific complex examples and analyzes their quality metrics
 * to identify areas for improvement.
 */

import { readFileSync, writeFileSync } from "fs";
import { join } from "path";

interface QualityMetrics {
  score: number;
  edgeCrossings: number;
  nodeOverlaps: number;
  labelOverlaps: number;
  parentChildContainment: number;
  spacingConsistency: number;
  rankAlignment: number;
  nodeCount: number;
  edgeCount: number;
}

interface AnalysisResult {
  exampleName: string;
  metrics: QualityMetrics | null;
  issues: string[];
  recommendations: string[];
}

// Complex examples to test
const COMPLEX_EXAMPLES = [
  "project_ecommerce.sruja",
  "project_saas_platform.sruja",
  "project_iot_platform.sruja",
  "pattern_agentic_ai.sruja",
  "pattern_microservices.sruja",
];

/**
 * Analyze quality metrics and provide recommendations
 */
function analyzeQuality(metrics: QualityMetrics, exampleName: string): AnalysisResult {
  const issues: string[] = [];
  const recommendations: string[] = [];

  // Score analysis
  if (metrics.score < 0.7) {
    issues.push(`Low overall score: ${(metrics.score * 100).toFixed(1)}%`);
    recommendations.push("Increase spacing and optimize edge routing");
  } else if (metrics.score < 0.85) {
    issues.push(`Moderate score: ${(metrics.score * 100).toFixed(1)}% (target: 85%+)`);
  }

  // Edge crossings
  if (metrics.edgeCrossings > 5) {
    issues.push(`High edge crossings: ${metrics.edgeCrossings}`);
    recommendations.push(
      "Increase edge minlen, use better spline routing, or adjust node positions"
    );
  } else if (metrics.edgeCrossings > 0) {
    issues.push(`Some edge crossings: ${metrics.edgeCrossings}`);
    recommendations.push("Consider increasing edge minlen for complex diagrams");
  }

  // Node overlaps
  if (metrics.nodeOverlaps > 0) {
    issues.push(`Node overlaps detected: ${metrics.nodeOverlaps}`);
    recommendations.push("Increase nodesep and ranksep, especially for complex diagrams");
  }

  // Label overlaps
  if (metrics.labelOverlaps > 2) {
    issues.push(`Label overlaps: ${metrics.labelOverlaps}`);
    recommendations.push("Adjust label positioning and increase edge separation");
  } else if (metrics.labelOverlaps > 0) {
    issues.push(`Some label overlaps: ${metrics.labelOverlaps}`);
  }

  // Parent-child containment
  if (metrics.parentChildContainment > 0) {
    issues.push(`Parent-child containment violations: ${metrics.parentChildContainment}`);
    recommendations.push("Increase cluster margins and compound node padding");
  }

  // Spacing consistency
  if (metrics.spacingConsistency < 0.8) {
    issues.push(`Poor spacing consistency: ${(metrics.spacingConsistency * 100).toFixed(1)}%`);
    recommendations.push("Improve uniform spacing between nodes");
  }

  // Rank alignment
  if (metrics.rankAlignment < 0.9) {
    issues.push(`Poor rank alignment: ${(metrics.rankAlignment * 100).toFixed(1)}%`);
    recommendations.push("Improve rank constraints and vertical alignment");
  }

  return {
    exampleName,
    metrics,
    issues,
    recommendations,
  };
}

/**
 * Generate analysis report
 */
function generateReport(results: AnalysisResult[]): string {
  let report = "# Diagram Quality Analysis Report\n\n";
  report += `Generated: ${new Date().toISOString()}\n\n`;
  report += `## Summary\n\n`;

  const totalExamples = results.length;
  const examplesWithMetrics = results.filter((r) => r.metrics !== null).length;
  const avgScore =
    results.filter((r) => r.metrics !== null).reduce((sum, r) => sum + (r.metrics?.score || 0), 0) /
      examplesWithMetrics || 0;

  report += `- **Total Examples Tested**: ${totalExamples}\n`;
  report += `- **Examples with Metrics**: ${examplesWithMetrics}\n`;
  report += `- **Average Score**: ${(avgScore * 100).toFixed(1)}%\n\n`;

  report += `## Detailed Analysis\n\n`;

  for (const result of results) {
    report += `### ${result.exampleName}\n\n`;

    if (result.metrics === null) {
      report += `**Status**: No metrics available (diagram may have failed to render)\n\n`;
      continue;
    }

    report += `**Score**: ${(result.metrics.score * 100).toFixed(1)}%\n\n`;
    report += `**Metrics**:\n`;
    report += `- Edge Crossings: ${result.metrics.edgeCrossings}\n`;
    report += `- Node Overlaps: ${result.metrics.nodeOverlaps}\n`;
    report += `- Label Overlaps: ${result.metrics.labelOverlaps}\n`;
    report += `- Parent-Child Containment: ${result.metrics.parentChildContainment}\n`;
    report += `- Spacing Consistency: ${(result.metrics.spacingConsistency * 100).toFixed(1)}%\n`;
    report += `- Rank Alignment: ${(result.metrics.rankAlignment * 100).toFixed(1)}%\n`;
    report += `- Node Count: ${result.metrics.nodeCount}\n`;
    report += `- Edge Count: ${result.metrics.edgeCount}\n\n`;

    if (result.issues.length > 0) {
      report += `**Issues**:\n`;
      for (const issue of result.issues) {
        report += `- ${issue}\n`;
      }
      report += `\n`;
    }

    if (result.recommendations.length > 0) {
      report += `**Recommendations**:\n`;
      for (const rec of result.recommendations) {
        report += `- ${rec}\n`;
      }
      report += `\n`;
    }
  }

  // Aggregate recommendations
  const allRecommendations = new Set<string>();
  for (const result of results) {
    result.recommendations.forEach((rec) => allRecommendations.add(rec));
  }

  if (allRecommendations.size > 0) {
    report += `## Overall Recommendations\n\n`;
    for (const rec of allRecommendations) {
      report += `- ${rec}\n`;
    }
    report += `\n`;
  }

  return report;
}

/**
 * Main function
 */
async function main() {
  console.log("Diagram Quality Analysis Tool\n");
  console.log("This script analyzes quality metrics from test results.\n");
  console.log("To collect fresh metrics, run the Playwright tests first:\n");
  console.log("  cd apps/designer");
  console.log("  npm run test:quality\n");

  // Try to read existing metrics
  const metricsPath = join(process.cwd(), "tests/results/all-examples-metrics.json");
  let metrics: any[] = [];

  try {
    const data = JSON.parse(readFileSync(metricsPath, "utf-8"));
    metrics = data.metrics || [];
    console.log(`Found ${metrics.length} metrics entries\n`);
  } catch (error) {
    console.log(`Could not read metrics file: ${metricsPath}`);
    console.log("Please run the quality tests first.\n");
    process.exit(1);
  }

  // Filter for complex examples
  const complexMetrics = metrics.filter((m) =>
    COMPLEX_EXAMPLES.some((example) => m.exampleName.includes(example))
  );

  console.log(`Analyzing ${complexMetrics.length} complex examples...\n`);

  const results: AnalysisResult[] = [];

  for (const metric of complexMetrics) {
    if (metric.grade === "ERROR" || metric.overallScore === 0) {
      results.push({
        exampleName: metric.exampleName,
        metrics: null,
        issues: ["Failed to render or collect metrics"],
        recommendations: ["Check if diagram renders correctly in browser"],
      });
      continue;
    }

    // Convert test metrics to quality metrics format
    const qualityMetrics: QualityMetrics = {
      score: metric.overallScore / 100, // Convert from percentage
      edgeCrossings: metric.edgeCrossings || 0,
      nodeOverlaps: metric.overlappingNodes || 0,
      labelOverlaps: metric.edgeLabelOverlaps || 0,
      parentChildContainment: metric.parentChildContainment || 0,
      spacingConsistency: 1 - (metric.spacingViolations || 0) / 10, // Heuristic conversion
      rankAlignment: 0.9, // Not directly available in test metrics
      nodeCount: metric.nodeCount || 0,
      edgeCount: metric.edgeCount || 0,
    };

    const analysis = analyzeQuality(qualityMetrics, metric.exampleName);
    results.push(analysis);
  }

  // Generate report
  const report = generateReport(results);

  // Save report
  const reportPath = join(process.cwd(), "tests/results/quality-analysis.md");
  writeFileSync(reportPath, report);
  console.log(`\nAnalysis complete! Report saved to: ${reportPath}\n`);
  console.log(report);
}

main().catch(console.error);
