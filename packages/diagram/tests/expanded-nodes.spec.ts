// Integration tests for expanded/hierarchical nodes
import { test, expect } from "@playwright/test";
import { getAvailableExamples } from "@sruja/shared";
import { collectLayoutMetrics, type LayoutMetrics } from "./utils/metrics-collector";

test.describe("Expanded Nodes Layout Quality", () => {
  test.setTimeout(300000); // 5 minutes

  test("E-Commerce Platform expanded should score at least B (80+)", async ({ page }) => {
    // Enable debug logging
    page.on("console", (msg) => console.log(`BROWSER LOG: ${msg.text()}`));
    page.on("pageerror", (err) => console.log(`BROWSER ERROR: ${err}`));

    const exampleName = "E-Commerce Platform";

    // First, load the example to find the system node ID
    await page.goto(`/?example=${encodeURIComponent(exampleName)}`);
    await page.waitForSelector(".react-flow", { timeout: 30000 });
    await page.waitForTimeout(1000);

    // Get the system node ID that should be expanded
    const systemNodeInfo = await page.evaluate(() => {
      const exposed = (window as any).__CYBER_GRAPH__;
      if (exposed?.nodes) {
        // Find the main system node (E-Commerce Platform)
        // Search by ID or label (more flexible)
        let systemNode = exposed.nodes.find((n: any) => {
          const id = (n.id || n.data?.id || "").toLowerCase();
          const label = (n.data?.label || "").toLowerCase();
          return (
            (id.includes("ecommerce") ||
              id.includes("platform") ||
              label.includes("e-commerce") ||
              label.includes("ecommerce")) &&
            (n.data?.childCount > 0 || n.data?.kind === "SoftwareSystem")
          );
        });

        // If not found, try finding any node with children (likely the main system)
        if (!systemNode) {
          systemNode = exposed.nodes.find(
            (n: any) =>
              (n.data?.childCount || 0) > 0 && (n.data?.kind === "SoftwareSystem" || !n.data?.kind) // Some nodes might not have kind set
          );
        }

        if (systemNode) {
          return {
            id: systemNode.id || systemNode.data?.id,
            label: systemNode.data?.label,
            hasChildren: (systemNode.data?.childCount || 0) > 0,
          };
        }
      }
      return null;
    });

    if (!systemNodeInfo) {
      // Log all nodes for debugging
      const allNodes = await page.evaluate(() => {
        const exposed = (window as any).__CYBER_GRAPH__;
        return (
          exposed?.nodes?.map((n: any) => ({
            id: n.id,
            dataId: n.data?.id,
            label: n.data?.label,
            kind: n.data?.kind,
            childCount: n.data?.childCount,
          })) || []
        );
      });
      console.log("Available nodes:", JSON.stringify(allNodes, null, 2));
      throw new Error("Could not find system node to expand");
    }

    const systemNodeId = systemNodeInfo.id;
    console.log(
      `Found system node: ${systemNodeInfo.label} (ID: ${systemNodeId}), reloading with expanded state...`
    );

    // Reload with expanded node via URL parameter
    await page.goto(
      `/?example=${encodeURIComponent(exampleName)}&expanded=${encodeURIComponent(systemNodeId)}`
    );
    await page.waitForSelector(".react-flow", { timeout: 30000 });
    await page.waitForTimeout(2000); // Wait for expansion and re-layout

    // Collect metrics for expanded state
    const metrics = await collectLayoutMetrics(page, exampleName, "advanced", {
      includePerformance: true,
      includeFullMetrics: true,
    });

    console.log(`\n=== Expanded State Metrics ===`);
    console.log(`Score: ${metrics.weightedScore.toFixed(1)}/100 (${metrics.grade})`);
    console.log(`Parent-Child Violations: ${metrics.parentChildViolations}`);
    console.log(`Edge Crossings: ${metrics.edgeCrossings}`);
    console.log(`Label Overlaps: ${metrics.qualityMetrics?.edgeLabelOverlaps || 0}`);
    console.log(`Has Hierarchy: ${metrics.hasHierarchy}`);
    console.log(`Node Count: ${metrics.nodeCount}`);

    // Assertions
    // Goal is 80+ (B) for all normal and expanded states
    expect(metrics.weightedScore).toBeGreaterThanOrEqual(80); // At least B grade (80+)
    expect(metrics.parentChildViolations).toBe(0); // No containment violations (critical - achieved!)

    // Log detailed breakdown for debugging
    if (metrics.qualityMetrics) {
      console.log(`\nDetailed Metrics:`);
      console.log(`  Overlap Score: ${metrics.qualityMetrics.overlapScore}`);
      console.log(`  Spacing Score: ${metrics.qualityMetrics.spacingScore}`);
      console.log(`  Edge Score: ${metrics.qualityMetrics.edgeScore}`);
      console.log(`  Hierarchy Score: ${metrics.qualityMetrics.hierarchyScore}`);
      console.log(`  Viewport Score: ${metrics.qualityMetrics.viewportScore}`);
    }

    // Track progress: if we're improving, log it
    if (metrics.weightedScore >= 80) {
      console.log(`✅ Goal achieved! Score is B or above.`);
    } else {
      console.log(
        `⚠️  Score ${metrics.weightedScore.toFixed(1)} is below B (80). Needs improvement.`
      );
    }
  });

  test("All examples with hierarchy should maintain B+ when expanded", async ({ page }) => {
    page.on("console", (msg) => console.log(`BROWSER LOG: ${msg.text()}`));

    const examples = await getAvailableExamples();
    const hierarchicalExamples = examples.filter(
      (e) =>
        e.name.includes("E-Commerce") ||
        e.name.includes("Real World") ||
        e.name.includes("Sruja Architecture")
    );

    const results: Array<{ name: string; score: number; violations: number }> = [];

    for (const example of hierarchicalExamples.slice(0, 3)) {
      // Test first 3 to keep it fast
      try {
        console.log(`\nTesting expanded state for: ${example.name}`);

        await page.goto(`/?example=${encodeURIComponent(example.name)}`);
        await page.waitForSelector(".react-flow", { timeout: 30000 });
        await page.waitForTimeout(1000);

        // Try to expand the main system
        await page.evaluate(() => {
          const exposed = (window as any).__CYBER_GRAPH__;
          if (exposed?.nodes) {
            const systemNode = exposed.nodes.find(
              (n: any) => n.data?.kind === "SoftwareSystem" && n.data?.childCount > 0
            );
            if (systemNode && systemNode.data?.onToggleExpand) {
              systemNode.data.onToggleExpand(systemNode.id);
            }
          }
        });
        await page.waitForTimeout(2000);

        const metrics = await collectLayoutMetrics(page, example.name, example.category, {
          includeFullMetrics: true,
        });

        results.push({
          name: example.name,
          score: metrics.weightedScore,
          violations: metrics.parentChildViolations,
        });

        console.log(
          `  Score: ${metrics.weightedScore.toFixed(1)} (${metrics.grade}), Violations: ${metrics.parentChildViolations}`
        );
      } catch (error) {
        console.error(`Failed for ${example.name}:`, error);
      }
    }

    // Summary
    console.log(`\n=== Expanded State Summary ===`);
    results.forEach((r) => {
      console.log(`  ${r.name}: ${r.score.toFixed(1)} (${r.violations} violations)`);
    });

    const avgScore = results.reduce((a, b) => a + b.score, 0) / results.length;
    const totalViolations = results.reduce((a, b) => a + b.violations, 0);

    console.log(`Average Score: ${avgScore.toFixed(1)}`);
    console.log(`Total Violations: ${totalViolations}`);

    // Assertions
    expect(avgScore).toBeGreaterThanOrEqual(80); // At least B average (80+)
    expect(totalViolations).toBe(0); // No containment violations
  });
});
