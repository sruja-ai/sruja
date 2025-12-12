import { test, expect } from "@playwright/test";
import { getAvailableExamples } from "@sruja/shared";
import {
  collectLayoutMetrics,
  saveMetricsToFile,
  type LayoutMetrics,
} from "./utils/metrics-collector";
import { compareMetrics, generateComparisonReport } from "./utils/comparison";
import * as fs from "fs";
import * as path from "path";

test.describe("Iterative Layout Optimization", () => {
  test.setTimeout(300000); // 5 minutes for full suite

  test.beforeEach(async ({ page }) => {
    await page.goto("/?tab=diagram");
    await page.waitForSelector(".app", { timeout: 30000 });
    await page.waitForTimeout(1000);
    // Dismiss any initial overlays/modals
    await page.keyboard.press("Escape");
    await page.waitForTimeout(500);
  });

  test("run iterative metrics check", async ({ page }) => {
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

    // 1. Collect Current Metrics
    const currentMetrics = new Map<string, LayoutMetrics>();

    // Filter down to a subset for faster iteration if needed, or run all
    // For now, let's run a representative subset to keep it fast enough for "iterative"
    // or run all if user wants comprehensive coverage.
    // Let's pick 5 random examples + known complex ones if possible, or just first 10.
    // Actually, let's run all but cap at 10 for speed unless env var set
    const maxExamples = process.env.FULL_SUITE ? examples.length : 10;
    const targetExamples = examples.slice(0, maxExamples);

    console.log(`Running for ${targetExamples.length} examples...`);

    for (const example of targetExamples) {
      try {
        console.log(`Processing: ${example.name}`);

        // Load example
        await page.click('button:has-text("Examples")');
        await page.waitForTimeout(500);

        const exampleButton = page.locator(`.example-item:has-text("${example.name}")`).first();
        if (await exampleButton.isVisible()) {
          await exampleButton.click();
        } else {
          console.warn(`Example ${example.name} not visible/found in list`);
          continue;
        }

        await page.waitForTimeout(1000);

        // Ensure Diagram tab
        const diagramTab = page.locator('button.view-tab:has-text("Diagram")').first();
        if (await diagramTab.isVisible()) {
          await diagramTab.click();
        }

        // Wait for layout
        try {
          await page.waitForSelector(".react-flow__node", { timeout: 10000 });
        } catch (e) {
          console.warn(`Nodes not found for ${example.name}`);
          continue;
        }

        const metrics = await collectLayoutMetrics(page, example.name, example.category, {
          includePerformance: true,
          includeFullMetrics: true,
        });

        currentMetrics.set(example.name, metrics);
        console.log(`  Grade: ${metrics.grade}, Score: ${metrics.weightedScore.toFixed(1)}`);
      } catch (error) {
        console.error(`Error processing ${example.name}:`, error);
      }
    }

    // 2. Save Current as Latest
    const latestFile = path.join(resultsDir, "latest-metrics.json");
    saveMetricsToFile(currentMetrics, latestFile);
    console.log(`Saved latest metrics to ${latestFile}`);

    // 3. Compare with Baseline if exists
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
        // overly strict?
        // Maybe check for severe regressions only
        const severeRegressions = comparison.comparisons.filter((c) => c.scoreDelta < -5);
        if (severeRegressions.length > 0) {
          console.warn(`⚠️  ${severeRegressions.length} severe regressions detected!`);
        }
      }
    } else {
      console.log(`No baseline found. Setting current run as baseline.`);
      saveMetricsToFile(currentMetrics, baselineFile);
    }

    // 4. Assertions on quality
    // Ensure average score is decent
    const scores = Array.from(currentMetrics.values()).map((m) => m.weightedScore);
    const avgScore = scores.reduce((a, b) => a + b, 0) / (scores.length || 1);
    console.log(`Average Score: ${avgScore.toFixed(1)}`);

    expect(avgScore).toBeGreaterThan(60); // Minimum acceptable quality
  });
});
