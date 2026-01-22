#!/usr/bin/env tsx
/**
 * Iterative Diagram Quality Improvement Loop
 *
 * This script:
 * 1. Runs quality tests on complex examples
 * 2. Analyzes results
 * 3. Identifies issues
 * 4. Suggests improvements
 * 5. Repeats until target quality is achieved
 */

import { execSync } from "child_process";
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

interface IterationResult {
  iteration: number;
  timestamp: string;
  metrics: QualityMetrics[];
  averageScore: number;
  issues: string[];
  improvements: string[];
}

const COMPLEX_EXAMPLES = [
  "project_ecommerce.sruja",
  "project_saas_platform.sruja",
  "project_iot_platform.sruja",
];

const TARGET_SCORE = 85; // Target B+ grade
const MAX_ITERATIONS = 5;
const MIN_SCORE_IMPROVEMENT = 2; // Minimum improvement per iteration to continue

/**
 * Run quality tests
 */
function runQualityTests(): boolean {
  console.log("Running quality tests...");
  try {
    const result = execSync("npm run test:quality", {
      cwd: join(process.cwd(), "apps/designer"),
      stdio: "pipe",
      encoding: "utf-8",
    });
    console.log("Tests completed");
    return true;
  } catch (error: any) {
    console.error("Test execution failed:", error.message);
    return false;
  }
}

/**
 * Load metrics from test results
 */
function loadMetrics(): QualityMetrics[] {
  const metricsPath = join(process.cwd(), "apps/designer/tests/results/all-examples-metrics.json");

  if (!existsSync(metricsPath)) {
    console.error(`Metrics file not found: ${metricsPath}`);
    return [];
  }

  try {
    const data = JSON.parse(readFileSync(metricsPath, "utf-8"));
    return (data.metrics || []).filter((m: QualityMetrics) =>
      COMPLEX_EXAMPLES.some((example) => m.exampleName.includes(example))
    );
  } catch (error) {
    console.error("Failed to load metrics:", error);
    return [];
  }
}

/**
 * Analyze metrics and identify issues
 */
function analyzeMetrics(metrics: QualityMetrics[]): {
  averageScore: number;
  issues: string[];
  recommendations: string[];
} {
  const validMetrics = metrics.filter((m) => m.grade !== "ERROR" && m.overallScore > 0);

  if (validMetrics.length === 0) {
    return {
      averageScore: 0,
      issues: ["No valid metrics found"],
      recommendations: ["Check if diagrams render correctly"],
    };
  }

  const averageScore =
    validMetrics.reduce((sum, m) => sum + m.overallScore, 0) / validMetrics.length;

  const issues: string[] = [];
  const recommendations: string[] = [];

  // Aggregate issues
  const totalOverlaps = validMetrics.reduce((sum, m) => sum + m.overlappingNodes, 0);
  const totalCrossings = validMetrics.reduce((sum, m) => sum + m.edgeCrossings, 0);
  const totalLabelOverlaps = validMetrics.reduce((sum, m) => sum + m.edgeLabelOverlaps, 0);
  const totalSpacingViolations = validMetrics.reduce((sum, m) => sum + m.spacingViolations, 0);
  const totalContainment = validMetrics.reduce((sum, m) => sum + m.parentChildContainment, 0);

  if (totalOverlaps > 0) {
    issues.push(`${totalOverlaps} node overlaps across examples`);
    recommendations.push("Increase nodesep and ranksep further");
  }

  if (totalCrossings > 5) {
    issues.push(`${totalCrossings} edge crossings (high)`);
    recommendations.push("Increase edge minlen, improve spline routing");
  } else if (totalCrossings > 0) {
    issues.push(`${totalCrossings} edge crossings`);
    recommendations.push("Consider increasing edge minlen slightly");
  }

  if (totalLabelOverlaps > 5) {
    issues.push(`${totalLabelOverlaps} label overlaps (high)`);
    recommendations.push("Increase label distance and edge separation");
  } else if (totalLabelOverlaps > 0) {
    issues.push(`${totalLabelOverlaps} label overlaps`);
  }

  if (totalSpacingViolations > 5) {
    issues.push(`${totalSpacingViolations} spacing violations (high)`);
    recommendations.push("Increase spacing scaling factors");
  }

  if (totalContainment > 0) {
    issues.push(`${totalContainment} parent-child containment violations`);
    recommendations.push("Increase cluster margins and compound node padding");
  }

  return {
    averageScore,
    issues,
    recommendations,
  };
}

/**
 * Generate improvement suggestions based on analysis
 */
function generateImprovements(analysis: ReturnType<typeof analyzeMetrics>): string[] {
  const improvements: string[] = [];

  if (analysis.averageScore < TARGET_SCORE) {
    improvements.push(
      `Current average score: ${analysis.averageScore.toFixed(1)} (target: ${TARGET_SCORE})`
    );
  }

  // Specific improvements based on issues
  if (analysis.issues.some((i) => i.includes("overlaps"))) {
    improvements.push("Consider increasing L1NodeSepScale and L1RankSepScale in constants.rs");
    improvements.push(
      "Increase DynamicScalingFactor or reduce DynamicScalingDivisor for more aggressive scaling"
    );
  }

  if (analysis.issues.some((i) => i.includes("crossings"))) {
    improvements.push("Increase edge minlen for dense/complex diagrams in buildEdgeConstraints");
    improvements.push("Consider using 'ortho' splines for very dense diagrams");
  }

  if (analysis.issues.some((i) => i.includes("label"))) {
    improvements.push("Increase label distance further in buildEdgeConstraints");
    improvements.push("Increase Sep value for diagrams with many edges");
  }

  if (analysis.issues.some((i) => i.includes("spacing"))) {
    improvements.push("Increase spacing scaling factors in BuildConstraints");
  }

  if (analysis.issues.some((i) => i.includes("containment"))) {
    improvements.push("Increase cluster margins in dot_generator.rs");
    improvements.push("Increase compound node padding in compoundNodes.ts");
  }

  return improvements;
}

/**
 * Save iteration results
 */
function saveIterationResult(result: IterationResult): void {
  const resultsDir = join(process.cwd(), "apps/designer/tests/results");
  const resultPath = join(resultsDir, `iteration-${result.iteration}.json`);

  writeFileSync(resultPath, JSON.stringify(result, null, 2));
  console.log(`\nIteration ${result.iteration} results saved to: ${resultPath}`);
}

/**
 * Generate summary report
 */
function generateSummaryReport(results: IterationResult[]): string {
  let report = "# Iterative Quality Improvement Summary\n\n";
  report += `Generated: ${new Date().toISOString()}\n\n`;

  report += `## Iterations Summary\n\n`;
  report += `| Iteration | Average Score | Status |\n`;
  report += `|---|---|---|\n`;

  for (const result of results) {
    const status =
      result.averageScore >= TARGET_SCORE
        ? "✅ Target Met"
        : result.averageScore >= TARGET_SCORE - 5
          ? "🟡 Close"
          : "❌ Needs Work";
    report += `| ${result.iteration} | ${result.averageScore.toFixed(1)} | ${status} |\n`;
  }

  report += `\n## Final Results\n\n`;
  const finalResult = results[results.length - 1];
  report += `- **Final Average Score**: ${finalResult.averageScore.toFixed(1)}\n`;
  report += `- **Target Score**: ${TARGET_SCORE}\n`;
  report += `- **Gap**: ${(TARGET_SCORE - finalResult.averageScore).toFixed(1)} points\n\n`;

  if (finalResult.issues.length > 0) {
    report += `### Remaining Issues\n\n`;
    for (const issue of finalResult.issues) {
      report += `- ${issue}\n`;
    }
    report += `\n`;
  }

  if (finalResult.improvements.length > 0) {
    report += `### Recommended Improvements\n\n`;
    for (const improvement of finalResult.improvements) {
      report += `- ${improvement}\n`;
    }
    report += `\n`;
  }

  // Show progress
  if (results.length > 1) {
    const firstScore = results[0].averageScore;
    const lastScore = results[results.length - 1].averageScore;
    const improvement = lastScore - firstScore;
    report += `### Progress\n\n`;
    report += `- **Starting Score**: ${firstScore.toFixed(1)}\n`;
    report += `- **Final Score**: ${lastScore.toFixed(1)}\n`;
    report += `- **Total Improvement**: ${improvement > 0 ? "+" : ""}${improvement.toFixed(1)} points\n`;
  }

  return report;
}

/**
 * Main iterative improvement loop
 */
async function main() {
  console.log("=".repeat(60));
  console.log("Iterative Diagram Quality Improvement Loop");
  console.log("=".repeat(60));
  console.log(`Target Score: ${TARGET_SCORE}`);
  console.log(`Max Iterations: ${MAX_ITERATIONS}`);
  console.log(`Complex Examples: ${COMPLEX_EXAMPLES.join(", ")}\n`);

  const results: IterationResult[] = [];
  let previousScore = 0;

  for (let iteration = 1; iteration <= MAX_ITERATIONS; iteration++) {
    console.log(`\n${"=".repeat(60)}`);
    console.log(`Iteration ${iteration}/${MAX_ITERATIONS}`);
    console.log(`${"=".repeat(60)}\n`);

    // Step 1: Run tests
    console.log("Step 1: Running quality tests...");
    const testSuccess = runQualityTests();
    if (!testSuccess) {
      console.error("Tests failed, stopping iteration");
      break;
    }

    // Step 2: Load and analyze metrics
    console.log("\nStep 2: Analyzing metrics...");
    const metrics = loadMetrics();
    if (metrics.length === 0) {
      console.error("No metrics found, stopping iteration");
      break;
    }

    const analysis = analyzeMetrics(metrics);
    console.log(`Average Score: ${analysis.averageScore.toFixed(1)}`);
    console.log(`Issues Found: ${analysis.issues.length}`);

    // Step 3: Generate improvements
    console.log("\nStep 4: Generating improvement suggestions...");
    const improvements = generateImprovements(analysis);

    // Save iteration result
    const iterationResult: IterationResult = {
      iteration,
      timestamp: new Date().toISOString(),
      metrics,
      averageScore: analysis.averageScore,
      issues: analysis.issues,
      improvements,
    };

    results.push(iterationResult);
    saveIterationResult(iterationResult);

    // Check if target achieved
    if (analysis.averageScore >= TARGET_SCORE) {
      console.log(
        `\n✅ Target score achieved! (${analysis.averageScore.toFixed(1)} >= ${TARGET_SCORE})`
      );
      break;
    }

    // Check if improvement is too small
    if (iteration > 1) {
      const scoreImprovement = analysis.averageScore - previousScore;
      if (scoreImprovement < MIN_SCORE_IMPROVEMENT) {
        console.log(
          `\n⚠️  Improvement too small (${scoreImprovement.toFixed(1)} < ${MIN_SCORE_IMPROVEMENT}), stopping`
        );
        break;
      }
    }

    previousScore = analysis.averageScore;

    // Display recommendations for manual implementation
    if (improvements.length > 0 && iteration < MAX_ITERATIONS) {
      console.log("\n📋 Recommended Improvements for Next Iteration:");
      improvements.forEach((imp, i) => {
        console.log(`   ${i + 1}. ${imp}`);
      });
      console.log(
        "\n⚠️  Please implement these improvements manually, then run this script again."
      );
      console.log("   Or press Enter to continue with current settings...");
    }
  }

  // Generate final summary
  console.log(`\n${"=".repeat(60)}`);
  console.log("Final Summary");
  console.log(`${"=".repeat(60)}\n`);

  const summary = generateSummaryReport(results);
  const summaryPath = join(
    process.cwd(),
    "apps/designer/tests/results/iterative-improvement-summary.md"
  );
  writeFileSync(summaryPath, summary);
  console.log(summary);
  console.log(`\nFull summary saved to: ${summaryPath}`);
}

main().catch(console.error);
