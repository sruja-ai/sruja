// Specific URL quality test for http://localhost:5173?tab=diagram&level=L0&expanded=ECommerce
import { test, expect, Page } from '@playwright/test';
import {
    calculateDiagramQuality,
    generateQualityReport
} from '../src/utils/diagramQuality';

/**
 * Get nodes from React Flow via window exposure
 */
async function getReactFlowNodes(page: Page): Promise<any[]> {
    return await page.evaluate(() => {
        const exposed = (window as any).__CYBER_GRAPH__;
        if (exposed?.nodes) {
            return exposed.nodes;
        }
        return [];
    });
}

/**
 * Get edges from React Flow via window exposure
 */
async function getReactFlowEdges(page: Page): Promise<any[]> {
    return await page.evaluate(() => {
        const exposed = (window as any).__CYBER_GRAPH__;
        if (exposed?.edges) {
            return exposed.edges;
        }
        return [];
    });
}

/**
 * Wait for layout to stabilize
 */
async function waitForLayoutStable(page: Page, timeout = 10000): Promise<void> {
    await page.waitForSelector('.loading-overlay', { state: 'hidden', timeout }).catch(() => { });
    await page.waitForTimeout(2000); // Wait for layout to complete
    await page.waitForSelector('.react-flow', { timeout });
}

test.describe('Specific URL Quality Tests', () => {
    test('L0 with ECommerce expanded should have good quality scores', async ({ page }) => {
        await page.goto('/?tab=diagram&level=L0&expanded=ECommerce');
        await page.waitForSelector('.architecture-canvas', { timeout: 15000 });
        await waitForLayoutStable(page);

        // Wait for graph to be exposed
        await page.waitForFunction(() => {
            return (window as any).__CYBER_GRAPH__?.nodes?.length > 0;
        }, { timeout: 10000 });

        const nodes = await getReactFlowNodes(page);
        const edges = await getReactFlowEdges(page);

        console.log(`Found ${nodes.length} nodes and ${edges.length} edges`);

        expect(nodes.length).toBeGreaterThan(0);

        const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
        const quality = calculateDiagramQuality(nodes, edges, viewportSize);

        const report = generateQualityReport(quality, 'L0 with ECommerce Expanded');
        console.log('\n' + report);

        // Assert good quality
        expect(quality.weightedScore).toBeGreaterThan(60);
        expect(quality.overlapScore).toBeGreaterThan(80); // Should have minimal overlaps
        expect(quality.grade).toMatch(/[ABC]/); // At least C grade
    });

    test('L0 baseline (no expansion) should have good quality', async ({ page }) => {
        await page.goto('/?tab=diagram&level=L0');
        await page.waitForSelector('.architecture-canvas', { timeout: 15000 });
        await waitForLayoutStable(page);

        await page.waitForFunction(() => {
            return (window as any).__CYBER_GRAPH__?.nodes?.length > 0;
        }, { timeout: 10000 });

        const nodes = await getReactFlowNodes(page);
        const edges = await getReactFlowEdges(page);

        expect(nodes.length).toBeGreaterThan(0);

        const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
        const quality = calculateDiagramQuality(nodes, edges, viewportSize);

        const report = generateQualityReport(quality, 'L0 Baseline');
        console.log('\n' + report);

        expect(quality.weightedScore).toBeGreaterThan(60);
        expect(quality.overlapScore).toBeGreaterThan(80);
    });

    test('Expansion should maintain quality (expand in place)', async ({ page }) => {
        // Start without expansion
        await page.goto('/?tab=diagram&level=L0');
        await page.waitForSelector('.architecture-canvas', { timeout: 15000 });
        await waitForLayoutStable(page);

        await page.waitForFunction(() => {
            return (window as any).__CYBER_GRAPH__?.nodes?.length > 0;
        }, { timeout: 10000 });

        const nodesBefore = await getReactFlowNodes(page);
        const edgesBefore = await getReactFlowEdges(page);
        const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
        const qualityBefore = calculateDiagramQuality(nodesBefore, edgesBefore, viewportSize);

        console.log('\n=== Before Expansion ===');
        console.log(generateQualityReport(qualityBefore, 'Before Expansion'));

        // Capture positions of root nodes
        const rootNodePositions = new Map<string, { x: number; y: number }>();
        nodesBefore.filter(n => !n.parentId).forEach(n => {
            rootNodePositions.set(n.id, { x: n.position.x, y: n.position.y });
        });

        // Navigate with expansion
        await page.goto('/?tab=diagram&level=L0&expanded=ECommerce');
        await waitForLayoutStable(page);

        await page.waitForFunction(() => {
            return (window as any).__CYBER_GRAPH__?.nodes?.length > 0;
        }, { timeout: 10000 });

        const nodesAfter = await getReactFlowNodes(page);
        const edgesAfter = await getReactFlowEdges(page);
        const qualityAfter = calculateDiagramQuality(nodesAfter, edgesAfter, viewportSize);

        console.log('\n=== After Expansion ===');
        console.log(generateQualityReport(qualityAfter, 'After Expansion'));

        // Check that quality didn't degrade significantly
        expect(qualityAfter.weightedScore).toBeGreaterThan(qualityBefore.weightedScore - 15);
        expect(qualityAfter.overlapScore).toBeGreaterThan(80);

        // Check that root nodes (other than expanded) maintained positions (within threshold)
        const EXPANSION_POSITION_THRESHOLD = 150; // Allow some movement for expansion
        const rootNodesAfter = nodesAfter.filter(n => !n.parentId && n.id !== 'ECommerce');
        let stableCount = 0;

        rootNodesAfter.forEach(n => {
            const beforePos = rootNodePositions.get(n.id);
            if (beforePos) {
                const distance = Math.sqrt(
                    Math.pow(n.position.x - beforePos.x, 2) +
                    Math.pow(n.position.y - beforePos.y, 2)
                );
                if (distance < EXPANSION_POSITION_THRESHOLD) {
                    stableCount++;
                }
            }
        });

        const stabilityPercentage = rootNodesAfter.length > 0
            ? (stableCount / rootNodesAfter.length) * 100
            : 100;

        console.log(`\nPosition Stability: ${stabilityPercentage.toFixed(1)}% (${stableCount}/${rootNodesAfter.length} nodes stable)`);

        // At least 60% of root nodes should be stable (allowing for some layout adjustment)
        expect(stabilityPercentage).toBeGreaterThan(60);
    });
});
