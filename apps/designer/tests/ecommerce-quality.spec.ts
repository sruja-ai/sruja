// apps/designer/tests/ecommerce-quality.spec.ts
// E2E test for ecommerce_platform.sruja quality score measurement
import { test, expect } from "@playwright/test";
import { writeFileSync, mkdirSync } from "fs";
import { join } from "path";

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

test.describe("ECommerce Platform Quality Measurement", () => {
  test("measure quality score for ecommerce_platform.sruja", async ({ page }) => {
    // Set base URL for tests
    const baseURL = process.env.PLAYWRIGHT_BASE_URL || "http://localhost:4321";
    
    // In production build, base path is /designer/, in dev it's /
    const isProduction = baseURL.includes("4322") || baseURL.includes("preview");
    const designerPath = isProduction ? "/designer" : "/designer";
    
    // Navigate to designer (following smoke test pattern)
    await page.goto(`${baseURL}${designerPath}`, { waitUntil: "domcontentloaded", timeout: 60000 });
    await page.waitForSelector(".app", { timeout: 30000 });
    
    // Try to load example from URL params first, but if that doesn't work, load manually
    const urlWithParams = `${baseURL}${designerPath}?level=L1&tab=diagram&example=ecommerce_platform.sruja&expanded=ECommerce%2CAPI`;
    await page.goto(urlWithParams, { waitUntil: "networkidle", timeout: 60000 });
    await page.waitForTimeout(2000);
    
    // Check if diagram is already visible
    const hasDiagram = await page.locator(".react-flow, .likec4-canvas").isVisible({ timeout: 5000 }).catch(() => false);
    
    // If not, load example manually (following smoke test pattern)
    if (!hasDiagram) {
      console.log("Loading example manually...");
      
      // Click examples button
      const examplesButton = page.locator('button[aria-label="Examples"]').first();
      await examplesButton.waitFor({ timeout: 10000 });
      await examplesButton.click();
      await page.waitForTimeout(1000);
      
      // Find and click ecommerce_platform example
      const exampleItems = page.locator(".example-item");
      await exampleItems.first().waitFor({ timeout: 10000 });
      
      // Find the ecommerce example by text
      const ecommerceExample = exampleItems.filter({ hasText: /ecommerce/i }).first();
      if (await ecommerceExample.isVisible({ timeout: 5000 }).catch(() => false)) {
        await ecommerceExample.click();
      } else {
        // Fallback: click first example if ecommerce not found
        await exampleItems.first().click();
      }
      
      await page.waitForTimeout(2000);
      
      // Switch to diagram tab if needed
      const diagramTab = page.locator('button.view-tab:has-text("Diagram")');
      if (await diagramTab.isVisible({ timeout: 5000 }).catch(() => false)) {
        await diagramTab.click();
        await page.waitForTimeout(1000);
      }
    }
    
    // Wait for diagram to load
    await page.waitForSelector(".react-flow, .likec4-canvas", { timeout: 90000 });
    
    // Wait for diagram content to appear (LikeC4 uses SVG)
    await page.waitForSelector(".likec4-diagram-container svg, .react-flow svg", { timeout: 60000 });
    
    // Wait for layout to stabilize
    await page.waitForTimeout(5000); // Give LikeC4 time to render diagram
    
    // Enable debug mode for overlap detection
    await page.evaluate(() => {
      (window as any).__LAYOUT_DEBUG__ = true;
    });

    // Wait for metrics to be available (they're set by ArchitectureCanvas)
    let layoutMetrics: any = null;
    for (let i = 0; i < 10; i++) {
      layoutMetrics = await page.evaluate(() => {
        return (window as any).__LAYOUT_METRICS__ as any;
      });
      if (layoutMetrics) break;
      await page.waitForTimeout(500);
    }
    
    if (!layoutMetrics) {
      throw new Error("Failed to extract layout metrics from page");
    }
    
    // Extract overlapping nodes details for debugging
    const overlappingNodesDetails = layoutMetrics?.overlappingNodes || [];
    if (overlappingNodesDetails.length > 0) {
      console.log("\n=== OVERLAPPING NODES DETAILS ===");
      overlappingNodesDetails.forEach((overlap: any, idx: number) => {
        console.log(`${idx + 1}. ${overlap.node1} overlaps ${overlap.node2}`);
        console.log(`   Overlap area: ${overlap.overlapArea?.toFixed(1) || 'N/A'}, Percentage: ${overlap.overlapPercentage?.toFixed(1) || 'N/A'}%`);
      });
      console.log("===================================\n");
    }
    
    // Extract all quality metrics
    const finalMetrics: QualityMetrics = {
      grade: layoutMetrics?.grade || "F",
      weightedScore: layoutMetrics?.weightedScore || 0,
      overallScore: layoutMetrics?.overallScore || 0,
      edgeCrossings: layoutMetrics?.edgeCrossings || 0,
      overlappingNodes: overlappingNodesDetails.length,
      parentChildContainment: layoutMetrics?.parentChildContainment?.length || 0,
      spacingViolations: layoutMetrics?.spacingViolations?.length || 0,
      edgeLabelOverlaps: layoutMetrics?.edgeLabelOverlaps || 0,
      clippedNodeLabels: layoutMetrics?.clippedNodeLabels || 0,
      violations: overlappingNodesDetails.map((o: any) => ({
        type: "node-overlap",
        severity: "high",
        description: `${o.node1} overlaps ${o.node2}`,
        affectedNodes: [o.node1, o.node2],
      })),
    };
    
    // Save metrics to file for iterative improvement script
    // Use process.cwd() to get the project root, then navigate to tests/results
    const resultsDir = join(process.cwd(), "tests", "results");
    mkdirSync(resultsDir, { recursive: true });
    const metricsFile = join(resultsDir, "ecommerce-quality-metrics.json");
    writeFileSync(metricsFile, JSON.stringify(finalMetrics, null, 2));
    
    console.log("\n=== ECOMMERCE PLATFORM QUALITY METRICS ===");
    console.log(`Grade: ${finalMetrics.grade}`);
    console.log(`Weighted Score: ${finalMetrics.weightedScore.toFixed(1)}`);
    console.log(`Overall Score: ${finalMetrics.overallScore.toFixed(1)}`);
    console.log(`Edge Crossings: ${finalMetrics.edgeCrossings}`);
    console.log(`Overlapping Nodes: ${finalMetrics.overlappingNodes}`);
    console.log(`Containment Violations: ${finalMetrics.parentChildContainment}`);
    console.log(`Spacing Violations: ${finalMetrics.spacingViolations}`);
    console.log(`Edge Label Overlaps: ${finalMetrics.edgeLabelOverlaps}`);
    console.log(`Clipped Labels: ${finalMetrics.clippedNodeLabels}`);
    console.log("==========================================\n");
    
    // Assert that we got metrics
    expect(finalMetrics.grade).toBeDefined();
    expect(finalMetrics.weightedScore).toBeGreaterThanOrEqual(0);
    
    // Log issues for improvement
    const issues: string[] = [];
    if (finalMetrics.grade === "F") issues.push("CRITICAL: Grade is F");
    if (finalMetrics.parentChildContainment > 0) {
      issues.push(`CRITICAL: ${finalMetrics.parentChildContainment} containment violations`);
    }
    if (finalMetrics.overlappingNodes > 0) {
      issues.push(`HIGH: ${finalMetrics.overlappingNodes} node overlaps`);
    }
    if (finalMetrics.edgeCrossings > 50) {
      issues.push(`MEDIUM: ${finalMetrics.edgeCrossings} edge crossings (target: <20)`);
    }
    if (finalMetrics.spacingViolations > 10) {
      issues.push(`MEDIUM: ${finalMetrics.spacingViolations} spacing violations`);
    }
    if (finalMetrics.edgeLabelOverlaps > 0) {
      issues.push(`LOW: ${finalMetrics.edgeLabelOverlaps} edge label overlaps`);
    }
    if (finalMetrics.clippedNodeLabels > 0) {
      issues.push(`LOW: ${finalMetrics.clippedNodeLabels} clipped node labels`);
    }
    
    if (issues.length > 0) {
      console.log("\n=== IDENTIFIED ISSUES ===");
      issues.forEach((issue) => console.log(`- ${issue}`));
      console.log("========================\n");
      
      // Save issues to file
      const issuesFile = join(resultsDir, "ecommerce-quality-issues.json");
      writeFileSync(issuesFile, JSON.stringify({ issues, metrics: finalMetrics }, null, 2));
    } else {
      // Save empty issues file even if no issues (for consistency)
      const issuesFile = join(resultsDir, "ecommerce-quality-issues.json");
      writeFileSync(issuesFile, JSON.stringify({ issues: [], metrics: finalMetrics }, null, 2));
    }
  });
});
