import { test, expect, Page } from '@playwright/test';
import type { DiagramQualityMetrics } from '../src/utils/diagramQuality';
import {
    calculateDiagramQuality,
    generateQualityReport
} from '../src/utils/diagramQuality';

/**
 * Get nodes from React Flow
 */
async function getReactFlowNodes(page: Page): Promise<any[]> {
    return await page.evaluate(() => {
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

async function waitForLayoutStable(page: Page, timeout = 5000): Promise<void> {
    await page.waitForSelector('.loading-overlay', { state: 'hidden', timeout }).catch(() => {});
    await page.waitForTimeout(1000);
    await page.waitForSelector('.react-flow', { timeout });
}

test.describe('Parent-Child Sizing and Aspect Ratio Tests', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('/');
        await page.waitForSelector('.architecture-canvas', { timeout: 10000 });
        await waitForLayoutStable(page);
    });

    test('parent nodes should properly contain their children', async ({ page }) => {
        // Navigate to L1 and expand a system
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
        }
        
        const nodes = await getReactFlowNodes(page);
        const edges = await getReactFlowNodes(page); // Empty edges for this test
        
        if (nodes.length === 0) {
            test.skip();
            return;
        }
        
        const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
        const quality = calculateDiagramQuality(nodes, edges, viewportSize);
        
        console.log(generateQualityReport(quality, 'Parent-Child Sizing'));
        
        // Check for parent-child size violations
        expect(quality.parentChildSizeViolations.length).toBe(0);
        expect(quality.hierarchyScore).toBeGreaterThan(80);
    });

    test('diagram should have reasonable aspect ratio', async ({ page }) => {
        const nodes = await getReactFlowNodes(page);
        const edges = await getReactFlowNodes(page);
        
        if (nodes.length === 0) {
            test.skip();
            return;
        }
        
        const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
        const quality = calculateDiagramQuality(nodes, edges, viewportSize);
        
        console.log(`Aspect Ratio: ${quality.aspectRatio.toFixed(2)} (score: ${quality.aspectRatioScore.toFixed(1)})`);
        
        // Aspect ratio should be between 0.3 and 3.0 (reasonable range)
        expect(quality.aspectRatio).toBeGreaterThan(0.3);
        expect(quality.aspectRatio).toBeLessThan(3.0);
        
        // Aspect ratio score should be reasonable
        expect(quality.aspectRatioScore).toBeGreaterThan(50);
    });

    test('expanded nodes should maintain proper parent-child relationships', async ({ page }) => {
        // Get baseline
        let nodes = await getReactFlowNodes(page);
        let edges = await getReactFlowNodes(page);
        const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
        const baselineQuality = calculateDiagramQuality(nodes, edges, viewportSize);
        
        // Expand multiple nodes
        const systemNodes = await page.evaluate(() => {
            const nodes: string[] = [];
            document.querySelectorAll('.react-flow__node').forEach(el => {
                const type = el.getAttribute('data-type');
                if (type === 'system') {
                    const id = el.getAttribute('data-id');
                    if (id) nodes.push(id);
                }
            });
            return nodes.slice(0, 2); // Expand up to 2 systems
        });
        
        for (const nodeId of systemNodes) {
            const node = page.locator(`[data-id="${nodeId}"]`).first();
            await node.dblclick();
            await waitForLayoutStable(page);
        }
        
        // Check quality after expansion
        nodes = await getReactFlowNodes(page);
        edges = await getReactFlowNodes(page);
        const expandedQuality = calculateDiagramQuality(nodes, edges, viewportSize);
        
        console.log(generateQualityReport(expandedQuality, 'After Expansion'));
        
        // Parent-child relationships should still be good
        expect(expandedQuality.parentChildSizeViolations.length).toBe(0);
        expect(expandedQuality.hierarchyScore).toBeGreaterThan(75);
        
        // Aspect ratio shouldn't degrade too much
        expect(expandedQuality.aspectRatioScore).toBeGreaterThan(baselineQuality.aspectRatioScore - 20);
    });

    test('should detect and report aspect ratio issues', async ({ page }) => {
        const nodes = await getReactFlowNodes(page);
        const edges = await getReactFlowNodes(page);
        
        if (nodes.length === 0) {
            test.skip();
            return;
        }
        
        const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
        const quality = calculateDiagramQuality(nodes, edges, viewportSize);
        
        const report = generateQualityReport(quality, 'Aspect Ratio Check');
        console.log(report);
        
        // If aspect ratio is problematic, it should be reported
        if (quality.aspectRatio < 0.5 || quality.aspectRatio > 2.0) {
            expect(quality.aspectRatioScore).toBeLessThan(70);
            console.log(`⚠️  Aspect ratio issue detected: ${quality.aspectRatio.toFixed(2)}`);
        }
    });
});
