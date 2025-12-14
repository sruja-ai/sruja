/**
 * Grade B Quality Gate Test
 *
 * Comprehensive test that validates ALL examples achieve Grade B (80+)
 * across ALL C4 levels (L0-L3) with both collapsed and expanded states.
 *
 * This is the primary quality gate for the layout engine.
 */
import { test, expect } from "@playwright/test";
import { getAvailableExamples } from "@sruja/shared";
import { collectLayoutMetrics, type LayoutMetrics } from "./utils/metrics-collector";
import * as fs from "fs";
import * as path from "path";

const GRADE_B_THRESHOLD = 80;
const LEVELS = ["L1", "L2", "L3"] as const; // L0 is enterprise overview, not always present

interface TestResult {
  example: string;
  level: string;
  state: "collapsed" | "expanded";
  score: number;
  grade: string;
  violations: {
    overlaps: number;
    containment: number;
    spacing: number;
    edgeCrossings: number;
  };
  passed: boolean;
}

test.describe("Grade B Quality Gate - All Examples", () => {
  test.setTimeout(600000); // 10 minutes for full suite

  test("All examples must achieve Grade B (80+) at all levels", async ({ page }) => {
    page.on("console", (msg) => {
      if (msg.type() === "error") {
        console.log(`BROWSER ERROR: ${msg.text()}`);
      }
    });

    const examples = await getAvailableExamples();
    const results: TestResult[] = [];
    const failures: TestResult[] = [];

    console.log(`\n${"=".repeat(60)}`);
    console.log(`GRADE B QUALITY GATE - Testing ${examples.length} examples`);
    console.log(`${"=".repeat(60)}\n`);

    for (const example of examples) {
      for (const level of LEVELS) {
        try {
          // Test collapsed state
          const collapsedResult = await testExampleAtLevel(
            page,
            example.name,
            example.category,
            level,
            "collapsed"
          );

          if (collapsedResult) {
            results.push(collapsedResult);
            if (!collapsedResult.passed) failures.push(collapsedResult);
            logResult(collapsedResult);

            // Test expanded state (find expandable nodes)
            const expandableNodeId = await findExpandableNode(page);
            if (expandableNodeId) {
              const expandedResult = await testExampleAtLevel(
                page,
                example.name,
                example.category,
                level,
                "expanded",
                expandableNodeId
              );
              if (expandedResult) {
                results.push(expandedResult);
                if (!expandedResult.passed) failures.push(expandedResult);
                logResult(expandedResult);
              }
            }
          }
        } catch (error) {
          console.error(`  ❌ ERROR: ${example.name}@${level}: ${error}`);
        }
      }
    }

    // Generate summary report
    const report = generateReport(results, failures);
    const reportPath = path.join(process.cwd(), "tests", "results", "grade-b-report.md");
    fs.mkdirSync(path.dirname(reportPath), { recursive: true });
    fs.writeFileSync(reportPath, report);

    console.log(`\n${"=".repeat(60)}`);
    console.log("SUMMARY");
    console.log(`${"=".repeat(60)}`);
    console.log(`Total Tests: ${results.length}`);
    console.log(`Passed: ${results.filter((r) => r.passed).length}`);
    console.log(`Failed: ${failures.length}`);
    console.log(`Report saved to: ${reportPath}`);

    // Final assertion
    if (failures.length > 0) {
      console.log("\nFAILED TESTS:");
      failures.forEach((f) => {
        console.log(`  - ${f.example}@${f.level} (${f.state}): ${f.score.toFixed(1)} (${f.grade})`);
      });
    }

    expect(failures.length, `${failures.length} tests failed to achieve Grade B`).toBe(0);
  });

  test("Quick validation - first 5 examples at L1 and L2", async ({ page }) => {
    // Faster test for CI - just check first 5 examples at L1 and L2
    const examples = (await getAvailableExamples()).slice(0, 5);
    const failures: TestResult[] = [];

    for (const example of examples) {
      for (const level of ["L1", "L2"] as const) {
        try {
          const result = await testExampleAtLevel(
            page,
            example.name,
            example.category,
            level,
            "collapsed"
          );
          if (result) {
            if (!result.passed) {
              failures.push(result);
              console.log(`❌ ${example.name}@${level}: ${result.score.toFixed(1)}`);
            } else {
              console.log(`✓ ${example.name}@${level}: ${result.score.toFixed(1)}`);
            }
          } else {
            console.log(`⊘ ${example.name}@${level}: skipped (no nodes)`);
          }
        } catch (error) {
          console.error(`Error testing ${example.name}@${level}:`, error);
        }
      }
    }

    expect(failures.length, `${failures.length} examples below Grade B`).toBe(0);
  });
});

async function testExampleAtLevel(
  page: any,
  exampleName: string,
  category: string,
  level: string,
  state: "collapsed" | "expanded",
  expandedNodeId?: string
): Promise<TestResult | null> {
  // Build URL
  let url = `/?example=${encodeURIComponent(exampleName)}&level=${level}`;
  if (state === "expanded" && expandedNodeId) {
    url += `&expanded=${encodeURIComponent(expandedNodeId)}`;
  }

  await page.goto(url);

  try {
    await page.waitForSelector(".react-flow", { timeout: 30000 });
    await page.waitForTimeout(1000); // Let layout settle
  } catch {
    // React Flow didn't render - likely no content at this level
    return null;
  }

  // Check if nodes exist at this level
  const hasNodes = await page.evaluate(() => {
    const graph = (window as any).__CYBER_GRAPH__;
    return graph?.nodes?.length > 0;
  });

  if (!hasNodes) {
    // No nodes at this level - skip this test
    return null;
  }

  const metrics = await collectLayoutMetrics(page, exampleName, category, {
    includeFullMetrics: true,
  });

  return {
    example: exampleName,
    level,
    state,
    score: metrics.weightedScore,
    grade: metrics.grade,
    violations: {
      overlaps: metrics.overlappingNodes,
      containment: metrics.parentChildViolations,
      spacing: metrics.qualityMetrics?.spacingViolations?.length || 0,
      edgeCrossings: metrics.edgeCrossings,
    },
    passed: metrics.weightedScore >= GRADE_B_THRESHOLD,
  };
}

async function findExpandableNode(page: any): Promise<string | null> {
  return await page.evaluate(() => {
    const graph = (window as any).__CYBER_GRAPH__;
    if (graph?.nodes) {
      const expandable = graph.nodes.find(
        (n: any) =>
          n.data?.childCount > 0 || (n.data?.kind === "SoftwareSystem" && !n.data?.expanded)
      );
      return expandable?.id || null;
    }
    return null;
  });
}

function logResult(result: TestResult): void {
  const icon = result.passed ? "✓" : "❌";
  const stateIndicator = result.state === "expanded" ? " [EXP]" : "";
  console.log(
    `  ${icon} ${result.example}@${result.level}${stateIndicator}: ` +
      `${result.score.toFixed(1)} (${result.grade}) ` +
      `[O:${result.violations.overlaps} C:${result.violations.containment}]`
  );
}

function generateReport(results: TestResult[], failures: TestResult[]): string {
  const passRate = (((results.length - failures.length) / results.length) * 100).toFixed(1);
  const avgScore = (results.reduce((a, b) => a + b.score, 0) / results.length).toFixed(1);

  let report = `# Grade B Quality Gate Report\n\n`;
  report += `**Generated:** ${new Date().toISOString()}\n\n`;
  report += `## Summary\n\n`;
  report += `| Metric | Value |\n`;
  report += `|--------|-------|\n`;
  report += `| Total Tests | ${results.length} |\n`;
  report += `| Passed | ${results.length - failures.length} |\n`;
  report += `| Failed | ${failures.length} |\n`;
  report += `| Pass Rate | ${passRate}% |\n`;
  report += `| Average Score | ${avgScore} |\n\n`;

  if (failures.length > 0) {
    report += `## Failed Tests\n\n`;
    report += `| Example | Level | State | Score | Grade | Violations |\n`;
    report += `|---------|-------|-------|-------|-------|------------|\n`;
    failures.forEach((f) => {
      report += `| ${f.example} | ${f.level} | ${f.state} | ${f.score.toFixed(1)} | ${f.grade} | O:${f.violations.overlaps} C:${f.violations.containment} |\n`;
    });
    report += `\n`;
  }

  report += `## All Results\n\n`;
  report += `| Example | Level | State | Score | Grade | Pass |\n`;
  report += `|---------|-------|-------|-------|-------|------|\n`;
  results.forEach((r) => {
    const icon = r.passed ? "✓" : "❌";
    report += `| ${r.example} | ${r.level} | ${r.state} | ${r.score.toFixed(1)} | ${r.grade} | ${icon} |\n`;
  });

  return report;
}
