#!/usr/bin/env tsx
// packages/diagram/tests/improve-layout.ts
// Iterative layout improvement workflow
import * as fs from "fs";
import * as path from "path";
import { execSync } from "child_process";
import type { LayoutMetrics } from "./utils/metrics-collector";
import { compareMetrics, generateComparisonReport } from "./utils/comparison";
import {
  generateImprovementSuggestions,
  generateSuggestionsReport,
} from "./utils/improvement-suggestions";

const RESULTS_DIR = path.join(process.cwd(), "tests", "results");
const BASELINE_FILE = path.join(RESULTS_DIR, "baseline-metrics.json");
const LATEST_FILE = path.join(RESULTS_DIR, "latest-metrics.json");
const SUGGESTIONS_FILE = path.join(RESULTS_DIR, "improvement-suggestions.md");

async function main() {
  console.log("🚀 Starting Iterative Layout Improvement Workflow\n");

  // Ensure results directory exists
  if (!fs.existsSync(RESULTS_DIR)) {
    fs.mkdirSync(RESULTS_DIR, { recursive: true });
  }

  // Step 1: Run e2e tests to collect metrics
  console.log("📊 Step 1: Collecting layout metrics...");
  try {
    execSync("npx playwright test tests/iterative-optimization.spec.ts", {
      stdio: "inherit",
      cwd: process.cwd(),
    });
  } catch (error) {
    console.error("❌ Failed to run e2e tests:", error);
    process.exit(1);
  }

  // Step 2: Load latest metrics
  console.log("\n📈 Step 2: Analyzing metrics...");
  if (!fs.existsSync(LATEST_FILE)) {
    console.error("❌ Latest metrics file not found. Run e2e tests first.");
    process.exit(1);
  }

  const latestData = JSON.parse(fs.readFileSync(LATEST_FILE, "utf-8"));
  const latestMetrics = new Map<string, LayoutMetrics>();
  latestData.metrics.forEach((m: LayoutMetrics) => {
    latestMetrics.set(m.exampleName, m);
  });

  // Step 3: Compare with baseline if exists
  let comparison = null;
  if (fs.existsSync(BASELINE_FILE)) {
    console.log("📊 Comparing with baseline...");
    const baselineData = JSON.parse(fs.readFileSync(BASELINE_FILE, "utf-8"));
    const baselineMetrics = new Map<string, LayoutMetrics>();
    baselineData.metrics.forEach((m: LayoutMetrics) => {
      baselineMetrics.set(m.exampleName, m);
    });

    comparison = compareMetrics(baselineMetrics, latestMetrics);
    const report = generateComparisonReport(comparison);
    const reportFile = path.join(RESULTS_DIR, "iteration-report.md");
    fs.writeFileSync(reportFile, report);
    console.log(`✅ Comparison report saved to ${reportFile}`);
  }

  // Step 4: Generate improvement suggestions
  console.log("\n💡 Step 3: Generating improvement suggestions...");
  const suggestions = generateImprovementSuggestions(latestMetrics, comparison?.comparisons);
  const suggestionsReport = generateSuggestionsReport(suggestions);
  fs.writeFileSync(SUGGESTIONS_FILE, suggestionsReport);
  console.log(`✅ Suggestions saved to ${SUGGESTIONS_FILE}`);

  // Step 5: Summary
  console.log("\n📋 Summary:");
  console.log(`   Total Examples Analyzed: ${latestMetrics.size}`);
  const scores = Array.from(latestMetrics.values()).map((m) => m.weightedScore);
  const avgScore = scores.reduce((a, b) => a + b, 0) / scores.length;
  console.log(`   Average Score: ${avgScore.toFixed(1)}/100`);

  const critical = suggestions.filter((s) => s.priority === "critical").length;
  const high = suggestions.filter((s) => s.priority === "high").length;
  const medium = suggestions.filter((s) => s.priority === "medium").length;
  const low = suggestions.filter((s) => s.priority === "low").length;

  console.log(`\n   Improvement Suggestions:`);
  console.log(`   - Critical: ${critical}`);
  console.log(`   - High: ${high}`);
  console.log(`   - Medium: ${medium}`);
  console.log(`   - Low: ${low}`);

  if (suggestions.length > 0) {
    console.log(`\n   Top Priority: ${suggestions[0].title}`);
    console.log(`   See ${SUGGESTIONS_FILE} for details`);
  }

  console.log("\n✨ Workflow complete!");
}

main().catch((error) => {
  console.error("Fatal error:", error);
  process.exit(1);
});
