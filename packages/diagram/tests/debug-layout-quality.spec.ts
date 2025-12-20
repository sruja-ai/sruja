
import { test, expect } from "@playwright/test";
import { collectLayoutMetrics } from "./utils/metrics-collector";
import * as fs from 'fs';
import * as path from 'path';

test.describe("Debug Layout Quality", () => {
    test("Analyze Ecommerce Platform L1 with Expanded Nodes", async ({ page }) => {
        // 1. Setup
        const exampleName = "ecommerce_platform.sruja";
        const level = "L1";
        const expandedNodes = new Set(["ECommerce", "API"]);

        // Read file content explicitly
        // Read file content explicitly
        const examplePath = path.join(process.cwd(), 'test-app/public/examples', exampleName);
        const dsl = fs.readFileSync(examplePath, 'utf8');

        // Go to base page
        await page.goto('/?tab=diagram');

        // 2. Load Graph via API
        await page.waitForFunction(() => (window as any).loadGraph);

        // Execute loadGraph
        await page.evaluate(async ({ dsl, level, expanded }) => {
            const result = await (window as any).loadGraph(dsl, level);
            if (!result.success) throw new Error(result.error);

            // Manually trigger expansion if needed, or rely on loadGraph to handle it if we modify it
            // The current loadGraph implementation doesn't accept expanded nodes, so we might need to patch it or just set the state directly
            // Waiting for graph to be set
        }, { dsl, level, expanded: Array.from(expandedNodes) });

        // We need to handle expansion. The current loadGraph doesn't take expanded nodes.
        // Let's modify the test to update the internal state or just reload with the expanded parameter but *after* we know the file content is safe?
        // Actually, let's just use the URL approach but with the *content* passed in a safer way? No, URL param length limit.

        // Better approach:
        // The previous error was that `loadExampleFile` failed.
        // If I use `loadGraph` I bypass `loadExampleFile`.
        // But `loadGraph` in `main.tsx` sets `expandedNodes` to empty set.
        // I need to update `main.tsx` to allow passing `expandedNodes` to `loadGraph`.

        // First action: Update main.tsx to accept expandedNodes in loadGraph
        // Second action: Update test to use loadGraph with DSL content.

        // 2. Wait for diagram to settle
        try {
            await page.waitForSelector(".react-flow", { timeout: 30000 });
            await page.waitForTimeout(2000); // Extra time for layout stability
        } catch (e) {
            console.error("Failed to load diagram:", e);
            throw e;
        }

        // 3. Collect Metrics
        const metrics = await collectLayoutMetrics(page, exampleName, "Debug", {
            includeFullMetrics: true,
        });

        // 4. Report Findings
        console.log("\n--- LAYOUT QUALITY REPORT ---");
        console.log(`Score: ${metrics.weightedScore.toFixed(1)} (Grade: ${metrics.grade})`);

        console.log("\nCRITICAL VIOLATIONS:");
        console.log(`Containment Violations: ${metrics.parentChildViolations}`);
        console.log(`Overlapping Nodes: ${metrics.overlappingNodes}`);

        console.log("\nOTHER ISSUES:");
        console.log(`Edge Crossings: ${metrics.edgeCrossings}`);
        console.log(`Edge Label Overlaps: ${metrics.qualityMetrics?.edgeLabelOverlaps}`);
        console.log(`Clipped Node Labels: ${metrics.qualityMetrics?.clippedNodeLabels}`);

        // Detailed violations if available
        if (metrics.qualityMetrics?.parentChildContainment?.length > 0) {
            console.log("\nCONTAINMENT DETAILS:");
            metrics.qualityMetrics.parentChildContainment.forEach((v: any) => {
                console.log(`- Child ${v.childId} is ${v.violation} parent ${v.parentId}: ${v.details}`);
            });
        }

        if (metrics.qualityMetrics?.overlappingNodes?.length > 0) {
            console.log("\nOVERLAP DETAILS:");
            metrics.qualityMetrics.overlappingNodes.forEach((v: any) => {
                console.log(`- ${v.node1} overlaps ${v.node2} by ${v.overlapPercentage.toFixed(1)}%`);
            });
        }

        // Capture screenshot for visual verification
        await page.screenshot({ path: "tests/results/debug-layout-failure.png", fullPage: true });

        // Fail if score is F
        expect(metrics.grade).not.toBe("F");
    });
});
