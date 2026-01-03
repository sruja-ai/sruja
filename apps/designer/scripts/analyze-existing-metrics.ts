#!/usr/bin/env tsx
/**
 * Analyze Existing Quality Metrics
 *
 * Analyzes existing test results and provides improvement recommendations
 * without requiring network access or running tests.
 */

import { readFileSync, writeFileSync, existsSync } from "fs";
import { join } from "path";

interface QualityMetrics {
  exampleName: string;
  grade: string;
  weightedScore: number;
  overallScore: number;
  edgeCrossings: number;
  overlappingNodes: number;
  parentChildContainment: number;
  spacingViolations: number;
  edgeLabelOverlaps: number;
  nodeCount: number;
  edgeCount: number;
}

const COMPLEX_EXAMPLES = [
  "project_ecommerce.sruja",
  "project_saas_platform.sruja",
  "project_iot_platform.sruja",
];

const TARGET_SCORE = 85;

function loadMetrics(): QualityMetrics[] {
  const metricsPath = join(process.cwd(), "tests/results/all-examples-metrics.json");

  if (!existsSync(metricsPath)) {
    console.error(`❌ Metrics file not found: ${metricsPath}`);
    console.log("\nTo generate metrics, run:");
    console.log("  npm run test:quality:all");
    return [];
  }

  try {
    const data = JSON.parse(readFileSync(metricsPath, "utf-8"));
    const allMetrics = data.metrics || [];

    const complexMetrics = allMetrics.filter((m: QualityMetrics) =>
      COMPLEX_EXAMPLES.some((example) => m.exampleName.includes(example))
    );

    console.log(`✅ Loaded ${complexMetrics.length} complex example metrics\n`);
    return complexMetrics;
  } catch (error) {
    console.error("Failed to load metrics:", error);
    return [];
  }
}

function analyzeMetrics(metrics: QualityMetrics[]) {
  const validMetrics = metrics.filter((m) => m.grade !== "ERROR" && m.overallScore > 0);

  if (validMetrics.length === 0) {
    console.log("❌ No valid metrics found");
    console.log("All examples may have failed to render.\n");
    return;
  }

  const averageScore =
    validMetrics.reduce((sum, m) => sum + m.overallScore, 0) / validMetrics.length;

  console.log("=".repeat(60));
  console.log("QUALITY ANALYSIS REPORT");
  console.log("=".repeat(60));
  console.log(`\nAverage Score: ${averageScore.toFixed(1)} / ${TARGET_SCORE}`);
  console.log(
    `Status: ${averageScore >= TARGET_SCORE ? "✅ Target Met" : "❌ Needs Improvement"}\n`
  );

  // Aggregate issues
  const totalOverlaps = validMetrics.reduce((sum, m) => sum + m.overlappingNodes, 0);
  const totalCrossings = validMetrics.reduce((sum, m) => sum + m.edgeCrossings, 0);
  const totalLabelOverlaps = validMetrics.reduce((sum, m) => sum + m.edgeLabelOverlaps, 0);
  const totalSpacingViolations = validMetrics.reduce((sum, m) => sum + m.spacingViolations, 0);
  const totalContainment = validMetrics.reduce((sum, m) => sum + m.parentChildContainment, 0);

  console.log("ISSUES SUMMARY:");
  console.log(`  Node Overlaps: ${totalOverlaps}`);
  console.log(`  Edge Crossings: ${totalCrossings}`);
  console.log(`  Label Overlaps: ${totalLabelOverlaps}`);
  console.log(`  Spacing Violations: ${totalSpacingViolations}`);
  console.log(`  Containment Violations: ${totalContainment}\n`);

  // Per-example breakdown
  console.log("PER-EXAMPLE BREAKDOWN:");
  console.log("-".repeat(60));
  for (const metric of validMetrics) {
    console.log(`\n${metric.exampleName}:`);
    console.log(`  Score: ${metric.overallScore.toFixed(1)} (${metric.grade})`);
    console.log(`  Nodes: ${metric.nodeCount}, Edges: ${metric.edgeCount}`);
    console.log(`  Overlaps: ${metric.overlappingNodes}, Crossings: ${metric.edgeCrossings}`);
    console.log(`  Label Overlaps: ${metric.edgeLabelOverlaps}`);
    console.log(`  Spacing Violations: ${metric.spacingViolations}`);
  }

  // Recommendations
  console.log("\n" + "=".repeat(60));
  console.log("RECOMMENDATIONS:");
  console.log("=".repeat(60) + "\n");

  const recommendations: string[] = [];

  if (totalOverlaps > 0) {
    recommendations.push(`🔴 CRITICAL: ${totalOverlaps} node overlaps detected`);
    recommendations.push("   → Increase nodesep/ranksep in pkg/export/dot/constraints.go");
    recommendations.push("   → Increase L1NodeSepScale and L1RankSepScale for L1 diagrams");
    recommendations.push("   → Increase DynamicScalingFactor for more aggressive spacing\n");
  }

  if (totalCrossings > 5) {
    recommendations.push(`🟡 HIGH: ${totalCrossings} edge crossings (target: < 5)`);
    recommendations.push("   → Increase edge minlen in buildEdgeConstraints()");
    recommendations.push("   → Consider using 'ortho' splines for very dense diagrams\n");
  } else if (totalCrossings > 0) {
    recommendations.push(`🟢 LOW: ${totalCrossings} edge crossings (acceptable)\n`);
  }

  if (totalLabelOverlaps > 5) {
    recommendations.push(`🟡 HIGH: ${totalLabelOverlaps} label overlaps (target: < 3)`);
    recommendations.push("   → Increase labelDistance in buildEdgeConstraints()");
    recommendations.push("   → Increase Sep value for diagrams with many edges\n");
  } else if (totalLabelOverlaps > 0) {
    recommendations.push(`🟢 LOW: ${totalLabelOverlaps} label overlaps (acceptable)\n`);
  }

  if (totalSpacingViolations > 5) {
    recommendations.push(`🟡 MEDIUM: ${totalSpacingViolations} spacing violations`);
    recommendations.push("   → Improve spacing scaling in BuildConstraints()\n");
  }

  if (totalContainment > 0) {
    recommendations.push(`🔴 CRITICAL: ${totalContainment} parent-child containment violations`);
    recommendations.push("   → Increase cluster margins in dot_generator.go");
    recommendations.push("   → Increase compound node padding in compoundNodes.ts\n");
  }

  if (averageScore < TARGET_SCORE) {
    const gap = TARGET_SCORE - averageScore;
    recommendations.push(`📊 Overall: Score gap of ${gap.toFixed(1)} points to reach target`);
    recommendations.push(
      "   → Focus on highest-impact issues first (overlaps > crossings > labels)"
    );
  }

  recommendations.forEach((rec) => console.log(rec));

  // Generate improvement plan
  console.log("\n" + "=".repeat(60));
  console.log("NEXT STEPS:");
  console.log("=".repeat(60) + "\n");

  console.log("1. Review recommendations above");
  console.log("2. Make code changes in:");
  console.log("   - pkg/export/dot/constraints.go");
  console.log("   - pkg/export/dot/dot_generator.go");
  console.log("   - apps/designer/src/components/SrujaCanvas/compoundNodes.ts");
  console.log("3. Rebuild WASM: make wasm");
  console.log("4. Test in browser: npm run dev");
  console.log("5. Run tests: npm run test:quality:all");
  console.log("6. Re-run this analysis: npm run analyze:metrics\n");

  // Save detailed report
  const report = {
    timestamp: new Date().toISOString(),
    averageScore,
    targetScore: TARGET_SCORE,
    gap: TARGET_SCORE - averageScore,
    issues: {
      nodeOverlaps: totalOverlaps,
      edgeCrossings: totalCrossings,
      labelOverlaps: totalLabelOverlaps,
      spacingViolations: totalSpacingViolations,
      containmentViolations: totalContainment,
    },
    examples: validMetrics.map((m) => ({
      name: m.exampleName,
      score: m.overallScore,
      grade: m.grade,
      issues: {
        overlaps: m.overlappingNodes,
        crossings: m.edgeCrossings,
        labels: m.edgeLabelOverlaps,
      },
    })),
    recommendations,
  };

  const reportPath = join(process.cwd(), "tests/results/quality-analysis-report.json");
  writeFileSync(reportPath, JSON.stringify(report, null, 2));
  console.log(`✅ Detailed report saved to: ${reportPath}`);
}

function main() {
  console.log("Analyzing Existing Quality Metrics\n");

  const metrics = loadMetrics();
  if (metrics.length === 0) {
    return;
  }

  analyzeMetrics(metrics);
}

main();
