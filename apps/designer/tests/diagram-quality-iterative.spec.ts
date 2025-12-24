// apps/designer/tests/diagram-quality-iterative.spec.ts
// E2E test for iterative diagram quality improvement based on scores
import { test, expect } from "@playwright/test";
import { writeFileSync, mkdirSync, readFileSync, existsSync } from "fs";
import { join } from "path";

interface QualityMetrics {
  score: number;
  edgeCrossings: number;
  nodeOverlaps: number;
  labelOverlaps: number;
  avgEdgeLength: number;
  edgeLengthVariance: number;
  rankAlignment: number;
  clusterBalance: number;
  spacingConsistency: number;
  timestamp: number;
  nodeCount: number;
  edgeCount: number;
  level: string;
}

interface QualityHistory {
  runs: Array<{
    timestamp: number;
    metrics: QualityMetrics;
    example: string;
    iteration: number;
  }>;
  baseline?: QualityMetrics;
  improvements: Array<{
    metric: string;
    before: number;
    after: number;
    improvement: number;
  }>;
}

test.describe("Diagram Quality Iterative Improvement", () => {
  const examples = [
    { file: "ecommerce_platform.sruja", name: "E-Commerce Platform" },
    { file: "project_ecommerce.sruja", name: "Project E-Commerce" },
  ];

  const resultsDir = join(process.cwd(), "tests", "results", "quality-iterative");
  mkdirSync(resultsDir, { recursive: true });

  examples.forEach(({ file, name }) => {
    test(`measure and track quality for ${name}`, async ({ page }) => {
      // Set base URL for tests
      const baseURL = process.env.PLAYWRIGHT_TEST_BASE_URL || "http://localhost:4173";
      const designerPath = "/designer";

      // Navigate to designer with example
      const urlWithParams = `${baseURL}${designerPath}?level=L1&tab=diagram&example=${file}`;
      await page.goto(urlWithParams, { waitUntil: "networkidle", timeout: 60000 });

      // Wait for diagram to load
      await page.waitForSelector(".react-flow", { timeout: 90000 });
      await page.waitForSelector(".react-flow svg", { timeout: 60000 });

      // Wait for layout to stabilize
      await page.waitForTimeout(5000);

      // Wait for quality metrics to be available
      let quality: QualityMetrics | null = null;
      for (let attempt = 0; attempt < 20; attempt++) {
        quality = await page.evaluate(() => {
          return (window as any).__DIAGRAM_QUALITY__ as QualityMetrics | null;
        });

        if (quality && quality.timestamp) {
          // Check if metrics are fresh (within last 10 seconds)
          const age = Date.now() - quality.timestamp;
          if (age < 10000) {
            break;
          }
        }

        await page.waitForTimeout(500);
      }

      if (!quality) {
        throw new Error("Failed to extract quality metrics from diagram");
      }

      // Load or create quality history
      const historyFile = join(resultsDir, `${file}-history.json`);
      let history: QualityHistory = {
        runs: [],
        improvements: [],
      };

      if (existsSync(historyFile)) {
        try {
          history = JSON.parse(readFileSync(historyFile, "utf-8"));
        } catch (e) {
          console.warn(`Failed to load history file: ${e}`);
        }
      }

      // Set baseline if this is the first run
      if (!history.baseline) {
        history.baseline = quality;
      }

      // Add current run
      const runNumber = history.runs.length + 1;
      history.runs.push({
        timestamp: Date.now(),
        metrics: quality,
        example: file,
        iteration: runNumber,
      });

      // Calculate improvements vs baseline
      if (history.baseline) {
        const improvements: typeof history.improvements = [];

        if (quality.score > history.baseline.score) {
          improvements.push({
            metric: "score",
            before: history.baseline.score,
            after: quality.score,
            improvement: quality.score - history.baseline.score,
          });
        }

        if (quality.edgeCrossings < history.baseline.edgeCrossings) {
          improvements.push({
            metric: "edgeCrossings",
            before: history.baseline.edgeCrossings,
            after: quality.edgeCrossings,
            improvement: history.baseline.edgeCrossings - quality.edgeCrossings,
          });
        }

        if (quality.nodeOverlaps < history.baseline.nodeOverlaps) {
          improvements.push({
            metric: "nodeOverlaps",
            before: history.baseline.nodeOverlaps,
            after: quality.nodeOverlaps,
            improvement: history.baseline.nodeOverlaps - quality.nodeOverlaps,
          });
        }

        if (quality.rankAlignment > history.baseline.rankAlignment) {
          improvements.push({
            metric: "rankAlignment",
            before: history.baseline.rankAlignment,
            after: quality.rankAlignment,
            improvement: quality.rankAlignment - history.baseline.rankAlignment,
          });
        }

        history.improvements = improvements;
      }

      // Save history
      writeFileSync(historyFile, JSON.stringify(history, null, 2));

      // Save current metrics snapshot
      const snapshotFile = join(resultsDir, `${file}-metrics-${runNumber}.json`);
      writeFileSync(snapshotFile, JSON.stringify(quality, null, 2));

      // Log metrics
      console.log(`\n=== ${name} Quality Metrics (Run ${runNumber}) ===`);
      console.log(`Score: ${quality.score.toFixed(3)} (target: ≥0.85)`);
      console.log(`Edge Crossings: ${quality.edgeCrossings} (target: ≤5)`);
      console.log(`Node Overlaps: ${quality.nodeOverlaps} (target: 0)`);
      console.log(`Label Overlaps: ${quality.labelOverlaps} (target: 0)`);
      console.log(`Rank Alignment: ${(quality.rankAlignment * 100).toFixed(1)}% (target: ≥95%)`);
      console.log(
        `Spacing Consistency: ${(quality.spacingConsistency * 100).toFixed(1)}% (target: ≥80%)`
      );
      console.log(`Nodes: ${quality.nodeCount}, Edges: ${quality.edgeCount}`);

      if (history.improvements.length > 0) {
        console.log(`\n=== Improvements vs Baseline ===`);
        history.improvements.forEach((imp) => {
          const percentage =
            imp.metric === "score" ||
            imp.metric.includes("Alignment") ||
            imp.metric.includes("Consistency")
              ? `${(imp.improvement * 100).toFixed(2)}%`
              : `${imp.improvement.toFixed(1)}`;
          console.log(
            `${imp.metric}: ${imp.before.toFixed(3)} → ${imp.after.toFixed(3)} (+${percentage})`
          );
        });
      }

      console.log("==========================================\n");

      // Assertions - these are soft thresholds that should improve over time
      expect(quality.score).toBeGreaterThan(0);
      expect(quality.nodeOverlaps).toBe(0); // No overlaps should be tolerated
      expect(quality.nodeCount).toBeGreaterThan(0);
      expect(quality.edgeCount).toBeGreaterThanOrEqual(0);

      // Quality thresholds (can be tightened as improvements are made)
      // Current baseline: score >= 0.50, crossings <= 10
      // Target: score >= 0.85, crossings <= 5
      const qualityGrade =
        quality.score >= 0.85 ? "A" : quality.score >= 0.7 ? "B" : quality.score >= 0.5 ? "C" : "F";

      console.log(`Quality Grade: ${qualityGrade}`);

      // Save quality report
      const reportFile = join(resultsDir, `${file}-report-${runNumber}.txt`);
      const report = [
        `Quality Report for ${name}`,
        `Run: ${runNumber}`,
        `Timestamp: ${new Date(quality.timestamp).toISOString()}`,
        ``,
        `Metrics:`,
        `  Score: ${quality.score.toFixed(3)} (Grade: ${qualityGrade})`,
        `  Edge Crossings: ${quality.edgeCrossings}`,
        `  Node Overlaps: ${quality.nodeOverlaps}`,
        `  Label Overlaps: ${quality.labelOverlaps}`,
        `  Rank Alignment: ${(quality.rankAlignment * 100).toFixed(1)}%`,
        `  Spacing Consistency: ${(quality.spacingConsistency * 100).toFixed(1)}%`,
        `  Avg Edge Length: ${quality.avgEdgeLength.toFixed(1)}`,
        `  Edge Length Variance: ${quality.edgeLengthVariance.toFixed(1)}`,
        ``,
        `Diagram:`,
        `  Nodes: ${quality.nodeCount}`,
        `  Edges: ${quality.edgeCount}`,
        `  Level: ${quality.level}`,
        ``,
        history.baseline ? `Baseline Score: ${history.baseline.score.toFixed(3)}` : "No baseline",
        history.baseline
          ? `Improvement: ${((quality.score - history.baseline.score) * 100).toFixed(2)}%`
          : "",
      ]
        .filter(Boolean)
        .join("\n");

      writeFileSync(reportFile, report);
    });
  });

  test("generate quality improvement summary", async () => {
    // This test generates a summary after all quality tests run
    const summary: Record<string, any> = {};

    examples.forEach(({ file }) => {
      const historyFile = join(resultsDir, `${file}-history.json`);
      if (existsSync(historyFile)) {
        try {
          const history = JSON.parse(readFileSync(historyFile, "utf-8"));
          summary[file] = {
            runs: history.runs.length,
            latestScore: history.runs[history.runs.length - 1]?.metrics.score,
            baselineScore: history.baseline?.score,
            improvements: history.improvements,
            trend:
              history.runs.length > 1
                ? history.runs[history.runs.length - 1].metrics.score >
                  history.runs[0].metrics.score
                  ? "improving"
                  : "regressing"
                : "baseline",
          };
        } catch (e) {
          console.warn(`Failed to load history for ${file}: ${e}`);
        }
      }
    });

    const summaryFile = join(resultsDir, "summary.json");
    writeFileSync(summaryFile, JSON.stringify(summary, null, 2));

    console.log("\n=== Quality Improvement Summary ===");
    Object.entries(summary).forEach(([file, data]) => {
      console.log(`\n${file}:`);
      console.log(`  Runs: ${data.runs}`);
      console.log(`  Latest Score: ${data.latestScore?.toFixed(3)}`);
      console.log(`  Baseline Score: ${data.baselineScore?.toFixed(3)}`);
      console.log(`  Trend: ${data.trend}`);
      if (data.improvements?.length > 0) {
        console.log(`  Improvements: ${data.improvements.length} metrics improved`);
      }
    });
    console.log("\n===================================\n");

    // This test always passes - it's just for reporting
    expect(Object.keys(summary).length).toBeGreaterThanOrEqual(0);
  });
});
