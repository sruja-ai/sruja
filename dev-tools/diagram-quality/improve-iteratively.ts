// apps/designer/scripts/improve-iteratively.ts
// Iterative improvement script for ecommerce_platform.sruja quality score
//
// ⚠️ DEVELOPER TOOL ONLY - For offline development use
// This script is NOT used in production or real-time user workflows.
// It's meant for developers to improve layout algorithms during development.
import { readFileSync, writeFileSync, existsSync, mkdirSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";
import { execSync } from "child_process";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

interface QualityMetrics {
  grade: string;
  weightedScore: number;
  overallScore: number;
  edgeCrossings: number;
  overlappingNodes: number;
  parentChildContainment: number;
  spacingViolations: number;
  edgeLabelOverlaps: number;
  clippedNodeLabels: number;
  violations: Array<{
    type: string;
    severity: string;
    description: string;
    affectedNodes: string[];
  }>;
}

interface ImprovementIteration {
  iteration: number;
  timestamp: string;
  metrics: QualityMetrics;
  issues: string[];
  improvements: string[];
  scoreDelta: number;
}

// Results directory - matches the test file's results directory
const RESULTS_DIR = join(__dirname, "../tests/results");
const METRICS_FILE = join(RESULTS_DIR, "ecommerce-quality-metrics.json");
const ISSUES_FILE = join(RESULTS_DIR, "ecommerce-quality-issues.json");
const ITERATION_LOG = join(RESULTS_DIR, "improvement-iterations.json");

function loadMetrics(): QualityMetrics | null {
  if (!existsSync(METRICS_FILE)) return null;
  try {
    return JSON.parse(readFileSync(METRICS_FILE, "utf-8"));
  } catch {
    return null;
  }
}

function loadIterations(): ImprovementIteration[] {
  if (!existsSync(ITERATION_LOG)) return [];
  try {
    return JSON.parse(readFileSync(ITERATION_LOG, "utf-8"));
  } catch {
    return [];
  }
}

function saveIteration(iteration: ImprovementIteration) {
  const iterations = loadIterations();
  iterations.push(iteration);
  writeFileSync(ITERATION_LOG, JSON.stringify(iterations, null, 2));
}

function identifyIssues(metrics: QualityMetrics): string[] {
  const issues: string[] = [];

  if (metrics.grade === "F") {
    issues.push("CRITICAL: Grade is F - must fix critical violations first");
  }

  if (metrics.parentChildContainment > 0) {
    issues.push(
      `CRITICAL: ${metrics.parentChildContainment} containment violations ` +
        `(causes F grade, penalty: 30 per violation)`
    );
  }

  if (metrics.overlappingNodes > 0) {
    issues.push(
      `HIGH: ${metrics.overlappingNodes} node overlaps ` + `(penalty: 25 per overlap, weight: 0.25)`
    );
  }

  if (metrics.edgeCrossings > 20) {
    const target = Math.max(0, Math.floor(metrics.edgeCrossings * 0.1));
    issues.push(
      `MEDIUM: ${metrics.edgeCrossings} edge crossings (target: ${target}, weight: 0.18)`
    );
  }

  if (metrics.spacingViolations > 5) {
    issues.push(
      `MEDIUM: ${metrics.spacingViolations} spacing violations ` + `(target: 0, weight: 0.1)`
    );
  }

  if (metrics.edgeLabelOverlaps > 0) {
    issues.push(`LOW: ${metrics.edgeLabelOverlaps} edge label overlaps`);
  }

  if (metrics.clippedNodeLabels > 0) {
    issues.push(`LOW: ${metrics.clippedNodeLabels} clipped node labels (weight: 0.1)`);
  }

  return issues;
}

function generateImprovementSuggestions(issues: string[]): string[] {
  const suggestions: string[] = [];

  if (issues.some((i) => i.includes("containment violations"))) {
    suggestions.push(
      "1. Fix containment violations in packages/layout/src/phases/optimization.ts:\n" +
        "   - Ensure applyContainmentEnforcement runs after every optimization phase\n" +
        "   - Increase padding in containment checks (currently 50px)\n" +
        "   - Add validation in layout phase to prevent containment-breaking operations"
    );
  }

  if (issues.some((i) => i.includes("node overlaps"))) {
    suggestions.push(
      "2. Fix node overlaps in packages/layout/src/phases/optimization.ts:\n" +
        "   - Improve applyOverlapRemoval to handle all cases\n" +
        "   - Ensure bottom-up processing covers all hierarchy levels\n" +
        "   - Add spatial indexing for faster overlap detection"
    );
  }

  if (issues.some((i) => i.includes("edge crossings"))) {
    suggestions.push(
      "3. Reduce edge crossings:\n" +
        "   - Improve initial layout in packages/layout/src/phases/layout.ts\n" +
        "   - Use Sugiyama algorithm for L1 level if needed\n" +
        "   - Optimize edge routing in packages/layout/src/phases/edge-routing.ts\n" +
        "   - Apply crossing minimization in optimization phase"
    );
  }

  if (issues.some((i) => i.includes("spacing violations"))) {
    suggestions.push(
      "4. Fix spacing violations:\n" +
        "   - Improve spacing optimization in packages/layout/src/phases/optimization.ts\n" +
        "   - Ensure minimum spacing of 30px between nodes\n" +
        "   - Adjust spacing based on node sizes"
    );
  }

  if (issues.some((i) => i.includes("clipped") || i.includes("label"))) {
    suggestions.push(
      "5. Fix label issues:\n" +
        "   - Improve node sizing in packages/layout/src/phases/sizing.ts\n" +
        "   - Ensure labels fit within node bounds\n" +
        "   - Add label placement optimization"
    );
  }

  return suggestions;
}

function runE2ETest(useProduction: boolean = true): boolean {
  console.log("\n🔍 Running e2e test to measure quality...");
  if (useProduction) {
    console.log("   Using production build (build + preview server)");
  } else {
    console.log("   Using dev server (http://localhost:4321)");
  }

  try {
    if (useProduction) {
      // Use production build test script
      execSync("npm run test:quality:prod", {
        cwd: join(__dirname, ".."),
        stdio: "inherit",
        env: { ...process.env },
      });
    } else {
      // Use dev server
      execSync("npx playwright test tests/ecommerce-quality.spec.ts", {
        cwd: join(__dirname, ".."),
        stdio: "inherit",
        env: { ...process.env, PLAYWRIGHT_BASE_URL: "http://localhost:4321" },
      });
    }
    return true;
  } catch (error) {
    console.error("❌ E2E test failed:", error);
    if (!useProduction) {
      console.error("\n💡 Make sure:");
      console.error("   1. Dev server is running: npm run dev");
      console.error("   2. Server is accessible at http://localhost:4321");
      console.error("   3. Playwright is installed: npx playwright install");
    }
    return false;
  }
}

function printReport(iteration: ImprovementIteration, previous?: ImprovementIteration) {
  console.log("\n" + "=".repeat(60));
  console.log(`ITERATION ${iteration.iteration} - QUALITY REPORT`);
  console.log("=".repeat(60));
  console.log(`Grade: ${iteration.metrics.grade}`);
  console.log(`Weighted Score: ${iteration.metrics.weightedScore.toFixed(1)}`);
  console.log(`Overall Score: ${iteration.metrics.overallScore.toFixed(1)}`);

  if (previous) {
    const delta = iteration.metrics.weightedScore - previous.metrics.weightedScore;
    const gradeChange = iteration.metrics.grade !== previous.metrics.grade;
    console.log(`\nScore Change: ${delta > 0 ? "+" : ""}${delta.toFixed(1)}`);
    if (gradeChange) {
      console.log(`Grade Change: ${previous.metrics.grade} → ${iteration.metrics.grade}`);
    }
  }

  console.log("\n--- METRICS ---");
  console.log(`Edge Crossings: ${iteration.metrics.edgeCrossings}`);
  console.log(`Overlapping Nodes: ${iteration.metrics.overlappingNodes}`);
  console.log(`Containment Violations: ${iteration.metrics.parentChildContainment}`);
  console.log(`Spacing Violations: ${iteration.metrics.spacingViolations}`);
  console.log(`Edge Label Overlaps: ${iteration.metrics.edgeLabelOverlaps}`);
  console.log(`Clipped Labels: ${iteration.metrics.clippedNodeLabels}`);

  if (iteration.issues.length > 0) {
    console.log("\n--- IDENTIFIED ISSUES ---");
    iteration.issues.forEach((issue) => console.log(`  • ${issue}`));
  }

  if (iteration.improvements.length > 0) {
    console.log("\n--- SUGGESTED IMPROVEMENTS ---");
    iteration.improvements.forEach((improvement) => console.log(improvement));
  }

  console.log("=".repeat(60) + "\n");
}

async function main() {
  mkdirSync(RESULTS_DIR, { recursive: true });

  // Check if we should use production build (default: true)
  const useProduction = process.env.USE_DEV_SERVER !== "true";

  const iterations = loadIterations();
  const currentIteration = iterations.length + 1;

  console.log(`\n🚀 Starting Iteration ${currentIteration}`);
  if (useProduction) {
    console.log("📦 Using production build (build + preview server)");
  } else {
    console.log("🔧 Using dev server");
  }
  console.log("=".repeat(60));

  // Run e2e test
  if (!runE2ETest(useProduction)) {
    console.error(
      "❌ Failed to run e2e test. Make sure the dev server is running on localhost:4321"
    );
    process.exit(1);
  }

  // Load metrics
  const metrics = loadMetrics();
  if (!metrics) {
    console.error("❌ Failed to load metrics. Check if e2e test completed successfully.");
    process.exit(1);
  }

  // Identify issues
  const issues = identifyIssues(metrics);
  const improvements = generateImprovementSuggestions(issues);

  // Calculate score delta
  const previous = iterations[iterations.length - 1];
  const scoreDelta = previous ? metrics.weightedScore - previous.metrics.weightedScore : 0;

  // Create iteration record
  const iteration: ImprovementIteration = {
    iteration: currentIteration,
    timestamp: new Date().toISOString(),
    metrics,
    issues,
    improvements,
    scoreDelta,
  };

  // Save and print
  saveIteration(iteration);
  printReport(iteration, previous);

  // Check if we've reached target
  if (metrics.grade !== "F" && metrics.weightedScore >= 80) {
    console.log("✅ Target achieved! Grade is B or better with score >= 80");
    console.log(`Final Score: ${metrics.weightedScore.toFixed(1)} (${metrics.grade})`);
  } else if (metrics.grade === "F") {
    console.log("⚠️  Still at F grade. Focus on critical violations:");
    issues.filter((i) => i.includes("CRITICAL")).forEach((issue) => console.log(`  - ${issue}`));
  } else {
    console.log(`📈 Progress: ${metrics.grade} grade, score: ${metrics.weightedScore.toFixed(1)}`);
    console.log("Continue iterating to improve further.");
  }

  // Save summary report
  const summaryFile = join(RESULTS_DIR, "improvement-summary.md");
  const summary = generateSummaryReport(iterations);
  writeFileSync(summaryFile, summary);
  console.log(`\n📄 Summary report saved to: ${summaryFile}`);
}

function generateSummaryReport(iterations: ImprovementIteration[]): string {
  let report = "# ECommerce Platform Quality Improvement Summary\n\n";
  report += `Total Iterations: ${iterations.length}\n\n`;

  if (iterations.length === 0) return report;

  const first = iterations[0];
  const last = iterations[iterations.length - 1];

  report += "## Progress Overview\n\n";
  report += `- **Initial Score**: ${first.metrics.weightedScore.toFixed(1)} (${first.metrics.grade})\n`;
  report += `- **Current Score**: ${last.metrics.weightedScore.toFixed(1)} (${last.metrics.grade})\n`;
  report += `- **Total Improvement**: ${(last.metrics.weightedScore - first.metrics.weightedScore).toFixed(1)} points\n\n`;

  report += "## Iteration History\n\n";
  iterations.forEach((iter) => {
    report += `### Iteration ${iter.iteration}\n\n`;
    report += `- **Score**: ${iter.metrics.weightedScore.toFixed(1)} (${iter.metrics.grade})\n`;
    if (iter.scoreDelta !== 0) {
      report += `- **Change**: ${iter.scoreDelta > 0 ? "+" : ""}${iter.scoreDelta.toFixed(1)}\n`;
    }
    report += `- **Issues**: ${iter.issues.length}\n`;
    if (iter.issues.length > 0) {
      iter.issues.forEach((issue) => {
        report += `  - ${issue}\n`;
      });
    }
    report += "\n";
  });

  return report;
}

main().catch(console.error);
