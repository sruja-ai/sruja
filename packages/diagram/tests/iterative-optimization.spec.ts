import { test, expect } from "@playwright/test";
import { getAvailableExamples } from "@sruja/shared";
import {
  collectBaselineForAllExamples,
  saveMetricsToFile,
  type LayoutMetrics,
} from "./utils/metrics-collector";
import { compareMetrics, generateComparisonReport } from "./utils/comparison";
import * as fs from "fs";
import * as path from "path";

test.describe("Iterative Layout Optimization", () => {
  test.setTimeout(300000); // 5 minutes for full suite

  test("run iterative metrics check", async ({ page }) => {
    // Enable debug logging
    page.on("console", (msg) => console.log(`BROWSER LOG: ${msg.text()}`));
    page.on("pageerror", (err) => console.log(`BROWSER ERROR: ${err}`));

    const examples = await getAvailableExamples();
    const resultsDir = path.join(process.cwd(), "tests", "results");
    if (!fs.existsSync(resultsDir)) {
      fs.mkdirSync(resultsDir, { recursive: true });
    }

    const baselineFile = path.join(resultsDir, "baseline-metrics.json");
    const hasBaseline = fs.existsSync(baselineFile);

    console.log(`\n=== Starting Iterative Metrics Check ===`);
    console.log(`Found ${examples.length} examples`);
    console.log(`Baseline available: ${hasBaseline}`);

    // Filter down to a subset for faster iteration if needed, or run all
    const maxExamples = process.env.FULL_SUITE ? examples.length : 10;
    const targetExamples = examples.slice(0, maxExamples);

    console.log(`Running for ${targetExamples.length} examples...`);

    // Collect Current Metrics
    const currentMetrics = await collectBaselineForAllExamples(page, targetExamples, {
      includePerformance: true,
      includeFullMetrics: true,
    });

    // Save Current as Latest
    const latestFile = path.join(resultsDir, "latest-metrics.json");
    saveMetricsToFile(currentMetrics, latestFile);

    // Compare with Baseline if exists
    if (hasBaseline) {
      const baselineContent = fs.readFileSync(baselineFile, "utf-8");
      const baselineData = JSON.parse(baselineContent);
      const baselineMetrics = new Map<string, LayoutMetrics>();

      baselineData.metrics.forEach((m: LayoutMetrics) => {
        baselineMetrics.set(m.exampleName, m);
      });

      const comparison = compareMetrics(baselineMetrics, currentMetrics);
      const report = generateComparisonReport(comparison);
      const reportFile = path.join(resultsDir, "iteration-report.md");
      fs.writeFileSync(reportFile, report);

      console.log(`\n=== Iteration Report ===`);
      console.log(`Improved: ${comparison.summary.improved}`);
      console.log(`Regressed: ${comparison.summary.regressed}`);
      console.log(`Report saved to ${reportFile}`);

      // Fail if major regressions
      if (comparison.summary.regressed > 0) {
        const severeRegressions = comparison.comparisons.filter((c) => c.scoreDelta < -5);
        if (severeRegressions.length > 0) {
          console.warn(`⚠️  ${severeRegressions.length} severe regressions detected!`);
        }
      }
    } else {
      console.log(`No baseline found. Setting current run as baseline.`);
      saveMetricsToFile(currentMetrics, baselineFile);
    }

    // Assertions on quality
    const scores = Array.from(currentMetrics.values()).map((m) => m.weightedScore);
    const avgScore = scores.reduce((a, b) => a + b, 0) / (scores.length || 1);
    console.log(`Average Score: ${avgScore.toFixed(1)}`);

    expect(avgScore).toBeGreaterThan(60); // Minimum acceptable quality
  });
});
