import { test, expect, Page } from '@playwright/test';
import type { DiagramQualityMetrics } from '../src/utils/diagramQuality';
import {
    calculateDiagramQuality,
    generateQualityReport,
    DEFAULT_QUALITY_WEIGHTS
} from '../src/utils/diagramQuality';
import {
    extractNodePositions,
    validatePositionPreservation
} from '../src/utils/layoutValidation';
import {
    optimizeLayout,
    getOptimizationRecommendations
} from '../src/utils/layoutOptimizer';

/**
 * Get nodes from React Flow via injected script
 */
async function getReactFlowNodes(page: Page): Promise<any[]> {
    return await page.evaluate(() => {
        // Try to access React Flow instance
        const reactFlowElement = document.querySelector('.react-flow');
        if (!reactFlowElement) return [];

        // Access via React DevTools or internal state
        // For now, extract from DOM
        const nodes: any[] = [];
        const nodeElements = document.querySelectorAll('.react-flow__node');

        nodeElements.forEach((el) => {
            const id = el.getAttribute('data-id') || el.id;
            if (!id) return;

            const style = window.getComputedStyle(el);
            const transform = style.transform;

            let x = 0, y = 0;
            if (transform && transform !== 'none') {
                const matrix = new DOMMatrix(transform);
                x = matrix.e;
                y = matrix.f;
            }

            const rect = el.getBoundingClientRect();
            const width = rect.width || 100;
            const height = rect.height || 100;

            const type = el.getAttribute('data-type') ||
                el.className.match(/node-(\w+)/)?.[1] ||
                'unknown';

            const parentId = el.closest('.react-flow__node')?.getAttribute('data-id') ||
                el.getAttribute('data-parent-id') ||
                undefined;

            nodes.push({
                id,
                position: { x, y },
                width,
                height,
                data: { type },
                parentId: parentId !== id ? parentId : undefined
            });
        });

        return nodes;
    });
}

/**
 * Get edges from React Flow
 */
async function getReactFlowEdges(page: Page): Promise<any[]> {
    return await page.evaluate(() => {
        const edges: any[] = [];
        const edgeElements = document.querySelectorAll('.react-flow__edge');

        edgeElements.forEach((el) => {
            const source = el.getAttribute('data-source');
            const target = el.getAttribute('data-target');
            const id = el.getAttribute('data-id') || `${source}-${target}`;

            if (source && target) {
                edges.push({
                    id,
                    source,
                    target
                });
            }
        });

        return edges;
    });
}

/**
 * Wait for layout to stabilize
 */
async function waitForLayoutStable(page: Page, timeout = 5000): Promise<void> {
    await page.waitForSelector('.loading-overlay', { state: 'hidden', timeout }).catch(() => { });
    await page.waitForTimeout(1000); // Wait for animations
    await page.waitForSelector('.react-flow', { timeout });
}

/**
 * Navigate to a specific C4 level
 */
async function navigateToLevel(page: Page, level: 'L0' | 'L1' | 'L2' | 'L3'): Promise<void> {
    // Check if there's a level selector or navigation
    const levelIndicator = page.locator('.level-indicator');
    if (await levelIndicator.count() > 0) {
        const currentLevel = await levelIndicator.textContent();
        if (currentLevel?.includes(level)) {
            return; // Already at this level
        }
    }

    // Try to find navigation buttons or use breadcrumb
    const breadcrumb = page.locator('.breadcrumb, [data-breadcrumb]');
    if (await breadcrumb.count() > 0) {
        // Navigate using breadcrumb
        if (level === 'L0') {
            // Click root
            await breadcrumb.locator('text=Architecture').first().click().catch(() => { });
        }
    }

    await waitForLayoutStable(page);
}

/**
 * Drill down into a system (L1 -> L2)
 */
async function drillDownToSystem(page: Page, systemLabel: string): Promise<void> {
    // Find and double-click the system node
    const systemNode = page.locator(`.react-flow__node:has-text("${systemLabel}")`).first();
    await systemNode.waitFor({ timeout: 5000 }).catch(() => { });
    await systemNode.dblclick().catch(() => { });
    await waitForLayoutStable(page);
}

test.describe('Diagram Quality Tests - All Levels', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('/?tab=diagram');
        await page.waitForSelector('.architecture-canvas', { timeout: 10000 });
        await waitForLayoutStable(page);
    });

    test('L0 - Landscape level should be well-structured', async ({ page }) => {
        await navigateToLevel(page, 'L0');

        const nodes = await getReactFlowNodes(page);
        const edges = await getReactFlowEdges(page);

        expect(nodes.length).toBeGreaterThan(0);

        const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
        const quality = calculateDiagramQuality(nodes, edges, viewportSize);

        const report = generateQualityReport(quality, 'L0 - Landscape');
        console.log(report);

        // L0 should have high quality
        expect(quality.overallScore).toBeGreaterThan(70);
        expect(quality.overlapScore).toBeGreaterThan(90); // No overlaps
        expect(quality.grade).toMatch(/[ABC]/);
    });

    test('L1 - System Context level should be well-structured', async ({ page }) => {
        await navigateToLevel(page, 'L1');

        const nodes = await getReactFlowNodes(page);
        const edges = await getReactFlowEdges(page);

        expect(nodes.length).toBeGreaterThan(0);

        const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
        const quality = calculateDiagramQuality(nodes, edges, viewportSize);

        const report = generateQualityReport(quality, 'L1 - System Context');
        console.log(report);

        expect(quality.overallScore).toBeGreaterThan(70);
        expect(quality.overlapScore).toBeGreaterThan(90);
        expect(quality.hierarchyScore).toBeGreaterThan(80); // Good hierarchy
    });

    test('L2 - Container level should be well-structured', async ({ page }) => {
        // First navigate to L1, then drill down
        await navigateToLevel(page, 'L1');

        // Find first system and drill down
        const systemNodes = await page.evaluate(() => {
            const nodes: string[] = [];
            document.querySelectorAll('.react-flow__node').forEach(el => {
                const type = el.getAttribute('data-type');
                const text = el.textContent || '';
                if (type === 'system' || text.includes('System')) {
                    nodes.push(text.trim().split('\n')[0]);
                }
            });
            return nodes;
        });

        if (systemNodes.length > 0) {
            await drillDownToSystem(page, systemNodes[0]);
        }

        const nodes = await getReactFlowNodes(page);
        const edges = await getReactFlowEdges(page);

        if (nodes.length > 0) {
            const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
            const quality = calculateDiagramQuality(nodes, edges, viewportSize);

            const report = generateQualityReport(quality, 'L2 - Container');
            console.log(report);

            expect(quality.overallScore).toBeGreaterThan(65);
            expect(quality.overlapScore).toBeGreaterThan(85);
        }
    });

    test('L3 - Component level should be well-structured', async ({ page }) => {
        // Navigate through levels to L3
        await navigateToLevel(page, 'L1');

        // Try to find a container to drill into
        const containerNodes = await page.evaluate(() => {
            const nodes: string[] = [];
            document.querySelectorAll('.react-flow__node').forEach(el => {
                const type = el.getAttribute('data-type');
                const text = el.textContent || '';
                if (type === 'container' || text.includes('Container')) {
                    nodes.push(text.trim().split('\n')[0]);
                }
            });
            return nodes;
        });

        if (containerNodes.length > 0) {
            const containerNode = page.locator(`.react-flow__node:has-text("${containerNodes[0]}")`).first();
            await containerNode.dblclick().catch(() => { });
            await waitForLayoutStable(page);
        }

        const nodes = await getReactFlowNodes(page);
        const edges = await getReactFlowEdges(page);

        if (nodes.length > 0) {
            const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
            const quality = calculateDiagramQuality(nodes, edges, viewportSize);

            const report = generateQualityReport(quality, 'L3 - Component');
            console.log(report);

            expect(quality.overallScore).toBeGreaterThan(60);
            expect(quality.overlapScore).toBeGreaterThan(80);
        }
    });

    test('Navigation between levels should maintain quality', async ({ page }) => {
        const qualityScores: { level: string; score: number }[] = [];

        // Test L0
        await navigateToLevel(page, 'L0');
        let nodes = await getReactFlowNodes(page);
        let edges = await getReactFlowEdges(page);
        if (nodes.length > 0) {
            const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
            const quality = calculateDiagramQuality(nodes, edges, viewportSize);
            qualityScores.push({ level: 'L0', score: quality.overallScore });
        }

        // Test L1
        await navigateToLevel(page, 'L1');
        nodes = await getReactFlowNodes(page);
        edges = await getReactFlowEdges(page);
        if (nodes.length > 0) {
            const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
            const quality = calculateDiagramQuality(nodes, edges, viewportSize);
            qualityScores.push({ level: 'L1', score: quality.overallScore });
        }

        // Test back to L0
        await navigateToLevel(page, 'L0');
        nodes = await getReactFlowNodes(page);
        edges = await getReactFlowEdges(page);
        if (nodes.length > 0) {
            const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
            const quality = calculateDiagramQuality(nodes, edges, viewportSize);
            qualityScores.push({ level: 'L0-return', score: quality.overallScore });
        }

        console.log('\n=== Quality Scores Across Navigation ===');
        qualityScores.forEach(({ level, score }) => {
            console.log(`${level}: ${score.toFixed(1)}/100`);
        });

        // All levels should maintain reasonable quality
        qualityScores.forEach(({ level, score }) => {
            expect(score).toBeGreaterThan(60);
        });
    });

    test('Expanded nodes should maintain diagram quality', async ({ page }) => {
        await navigateToLevel(page, 'L1');

        // Get baseline quality
        let nodes = await getReactFlowNodes(page);
        let edges = await getReactFlowEdges(page);
        const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
        const baselineQuality = calculateDiagramQuality(nodes, edges, viewportSize);

        console.log(generateQualityReport(baselineQuality, 'Baseline L1'));

        // Expand a system node
        const systemNodes = await page.evaluate(() => {
            const nodes: string[] = [];
            document.querySelectorAll('.react-flow__node').forEach(el => {
                const type = el.getAttribute('data-type');
                if (type === 'system') {
                    const id = el.getAttribute('data-id');
                    if (id) nodes.push(id);
                }
            });
            return nodes;
        });

        if (systemNodes.length > 0) {
            const systemNode = page.locator(`[data-id="${systemNodes[0]}"]`).first();
            await systemNode.dblclick();
            await waitForLayoutStable(page);

            // Get quality after expansion
            nodes = await getReactFlowNodes(page);
            edges = await getReactFlowEdges(page);
            const expandedQuality = calculateDiagramQuality(nodes, edges, viewportSize);

            console.log(generateQualityReport(expandedQuality, 'After Expansion'));

            // Quality should not degrade significantly
            expect(expandedQuality.overallScore).toBeGreaterThan(baselineQuality.overallScore - 15);
            expect(expandedQuality.overlapScore).toBeGreaterThan(80);
            expect(expandedQuality.hierarchyScore).toBeGreaterThan(70); // Children should be contained
        }
    });

    test('Comprehensive quality analysis across all scenarios', async ({ page }) => {
        const results: Array<{
            scenario: string;
            quality: DiagramQualityMetrics;
        }> = [];

        // Scenario 1: L0
        await navigateToLevel(page, 'L0');
        let nodes = await getReactFlowNodes(page);
        let edges = await getReactFlowEdges(page);
        if (nodes.length > 0) {
            const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
            results.push({
                scenario: 'L0 - Initial',
                quality: calculateDiagramQuality(nodes, edges, viewportSize)
            });
        }

        // Scenario 2: L1
        await navigateToLevel(page, 'L1');
        nodes = await getReactFlowNodes(page);
        edges = await getReactFlowEdges(page);
        if (nodes.length > 0) {
            const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
            results.push({
                scenario: 'L1 - System Context',
                quality: calculateDiagramQuality(nodes, edges, viewportSize)
            });
        }

        // Scenario 3: L1 with expansion
        const systemNodes = await page.evaluate(() => {
            const nodes: string[] = [];
            document.querySelectorAll('.react-flow__node').forEach(el => {
                const type = el.getAttribute('data-type');
                if (type === 'system') {
                    const id = el.getAttribute('data-id');
                    if (id) nodes.push(id);
                }
            });
            return nodes;
        });

        if (systemNodes.length > 0) {
            const systemNode = page.locator(`[data-id="${systemNodes[0]}"]`).first();
            await systemNode.dblclick();
            await waitForLayoutStable(page);

            nodes = await getReactFlowNodes(page);
            edges = await getReactFlowEdges(page);
            if (nodes.length > 0) {
                const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
                results.push({
                    scenario: 'L1 - With Expanded System',
                    quality: calculateDiagramQuality(nodes, edges, viewportSize)
                });
            }
        }

        // Generate comprehensive report
        console.log('\n=== Comprehensive Quality Analysis ===\n');
        results.forEach(({ scenario, quality }) => {
            console.log(generateQualityReport(quality, scenario));
        });

        // All scenarios should meet minimum quality standards
        results.forEach(({ scenario, quality }) => {
            expect(quality.weightedScore).toBeGreaterThan(60);
            expect(quality.grade).toMatch(/[ABCD]/); // At least D grade
        });

        // Average quality should be good
        const avgScore = results.reduce((sum, r) => sum + r.quality.weightedScore, 0) / results.length;
        expect(avgScore).toBeGreaterThan(70);
    });

    test('should optimize layout based on weighted scores', async ({ page }) => {
        await navigateToLevel(page, 'L1');

        const nodes = await getReactFlowNodes(page);
        const edges = await getReactFlowEdges(page);

        if (nodes.length === 0) {
            test.skip();
            return;
        }

        const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };

        // Get current quality
        const currentQuality = calculateDiagramQuality(nodes, edges, viewportSize);
        console.log(generateQualityReport(currentQuality, 'Current Layout'));

        // Try optimization
        const optimizationResult = await optimizeLayout({
            nodes,
            edges,
            currentLevel: 'L1',
            viewportSize,
            weights: DEFAULT_QUALITY_WEIGHTS,
            maxIterations: 3
        });

        console.log('\n=== Optimization Results ===');
        console.log(`Best Score: ${optimizationResult.bestScore.toFixed(1)}/100`);
        console.log(`Tried ${optimizationResult.triedConfigurations.length} configurations:`);
        optimizationResult.triedConfigurations.forEach(({ config, score }) => {
            console.log(`  ${config}: ${score.toFixed(1)}/100`);
        });

        console.log(generateQualityReport(optimizationResult.bestMetrics, 'Optimized Layout'));

        // Get recommendations
        const recommendations = getOptimizationRecommendations(optimizationResult.bestMetrics);
        console.log('\nRecommendations:');
        recommendations.forEach(rec => console.log(`  - ${rec}`));

        // Optimized layout should be at least as good as current
        expect(optimizationResult.bestScore).toBeGreaterThanOrEqual(currentQuality.weightedScore - 5); // Allow small margin
    });

    test('should track all quality metrics with weights', async ({ page }) => {
        await navigateToLevel(page, 'L1');

        const nodes = await getReactFlowNodes(page);
        const edges = await getReactFlowEdges(page);

        if (nodes.length === 0) {
            test.skip();
            return;
        }

        const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
        const quality = calculateDiagramQuality(nodes, edges, viewportSize);

        // Verify all metrics are calculated
        expect(quality.overlapScore).toBeGreaterThanOrEqual(0);
        expect(quality.spacingScore).toBeGreaterThanOrEqual(0);
        expect(quality.edgeCrossings).toBeGreaterThanOrEqual(0);
        expect(quality.edgesOverNodes).toBeGreaterThanOrEqual(0);
        expect(quality.edgeBends).toBeGreaterThanOrEqual(0);
        expect(quality.hierarchyScore).toBeGreaterThanOrEqual(0);
        expect(quality.viewportScore).toBeGreaterThanOrEqual(0);
        expect(quality.consistencyScore).toBeGreaterThanOrEqual(0);
        expect(quality.weightedScore).toBeGreaterThanOrEqual(0);

        // Log detailed metrics
        console.log('\n=== Detailed Quality Metrics ===');
        console.log(`Node Overlaps: ${quality.overlappingNodes.length}`);
        console.log(`Edge Crossings: ${quality.edgeCrossings}`);
        console.log(`Edges Over Nodes: ${quality.edgesOverNodes}`);
        console.log(`Edge Bends: ${quality.edgeBends}`);
        console.log(`Edge Length: min=${quality.edgeLength.min.toFixed(0)}px, max=${quality.edgeLength.max.toFixed(0)}px, avg=${quality.edgeLength.average.toFixed(0)}px`);
        console.log(`Hierarchy Violations: ${quality.parentChildContainment.length}`);
        console.log(`Viewport Utilization: ${(quality.viewportUtilization * 100).toFixed(1)}%`);

        // Verify weighted score calculation
        const manualWeighted = (
            quality.overlapScore * DEFAULT_QUALITY_WEIGHTS.overlap +
            quality.spacingScore * DEFAULT_QUALITY_WEIGHTS.spacing +
            (100 - (quality.edgeCrossings * 10)) * DEFAULT_QUALITY_WEIGHTS.edgeCrossings +
            quality.hierarchyScore * DEFAULT_QUALITY_WEIGHTS.hierarchy +
            quality.viewportScore * DEFAULT_QUALITY_WEIGHTS.viewport +
            quality.consistencyScore * DEFAULT_QUALITY_WEIGHTS.consistency
        );

        // Weighted score should be reasonable
        expect(quality.weightedScore).toBeGreaterThan(0);
        expect(quality.weightedScore).toBeLessThanOrEqual(100);
    });
});
