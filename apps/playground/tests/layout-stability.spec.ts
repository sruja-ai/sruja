import { test, expect, Page } from '@playwright/test';
import type { LayoutValidationResult, NodePosition } from '../src/utils/layoutValidation';
import {
    extractNodePositions,
    validatePositionPreservation,
    generateValidationReport
} from '../src/utils/layoutValidation';

/**
 * Helper to get node positions from the page
 */
async function getNodePositions(page: Page): Promise<Map<string, NodePosition>> {
    return await page.evaluate(() => {
        // Access React Flow's internal state through window
        // This assumes React Flow exposes nodes via a global or we can access via DOM
        const nodes: NodePosition[] = [];

        // Try to get nodes from React Flow's internal state
        // We'll use a more reliable method: extract from DOM
        const nodeElements = document.querySelectorAll('[data-id]');

        nodeElements.forEach((el) => {
            const id = el.getAttribute('data-id');
            if (!id) return;

            const rect = el.getBoundingClientRect();
            const transform = window.getComputedStyle(el).transform;

            // Extract position from transform matrix
            let x = 0, y = 0;
            if (transform && transform !== 'none') {
                const matrix = new DOMMatrix(transform);
                x = matrix.e;
                y = matrix.f;
            } else {
                // Fallback to bounding rect
                x = rect.left;
                y = rect.top;
            }

            const parentId = el.closest('[data-id]')?.getAttribute('data-id') || undefined;
            const type = el.getAttribute('data-type') || undefined;

            nodes.push({
                id,
                x,
                y,
                type,
                parentId: parentId !== id ? parentId : undefined
            });
        });

        const positions = new Map<string, NodePosition>();
        nodes.forEach(node => {
            positions.set(node.id, node);
        });

        return positions;
    });
}

/**
 * Better method: Get positions from React Flow's internal state via injected script
 */
async function getNodePositionsFromReactFlow(page: Page): Promise<Map<string, NodePosition>> {
    const result = await page.evaluate(() => {
        // Inject a function to access React Flow's internal state
        const reactFlowInstance = (window as any).__REACT_FLOW_INSTANCE__;

        if (reactFlowInstance && reactFlowInstance.getNodes) {
            const nodes = reactFlowInstance.getNodes();
            // Return array of objects, Map is not serializable
            return nodes.map((node: any) => ({
                id: node.id,
                x: node.position.x,
                y: node.position.y,
                type: node.data?.type,
                parentId: node.parentId
            }));
        }

        // Fallback: return null to trigger DOM extraction in Node context
        return null;
    });

    if (result) {
        const positions = new Map<string, NodePosition>();
        result.forEach((p: any) => positions.set(p.id, p));
        return positions;
    }

    // Fallback: extract from DOM
    return getNodePositionsFromDOM(page);
}

/**
 * Extract positions from DOM elements
 */
async function getNodePositionsFromDOM(page: Page): Promise<Map<string, NodePosition>> {
    return await page.evaluate(() => {
        const positions = new Map<string, NodePosition>();

        // React Flow nodes have specific classes and data attributes
        const nodeElements = document.querySelectorAll('.react-flow__node');

        nodeElements.forEach((el) => {
            const id = el.getAttribute('data-id') || el.id;
            if (!id) return;

            // Get position from transform
            const style = window.getComputedStyle(el);
            const transform = style.transform;

            let x = 0, y = 0;
            if (transform && transform !== 'none') {
                const matrix = new DOMMatrix(transform);
                x = matrix.e;
                y = matrix.f;
            }

            // Get type from data attribute or class
            const type = el.getAttribute('data-type') ||
                el.className.match(/node-(\w+)/)?.[1] ||
                undefined;

            positions.set(id, {
                id,
                x,
                y,
                type
            });
        });

        return positions;
    });
}

/**
 * Wait for layout to stabilize
 */
async function waitForLayoutStable(page: Page, timeout = 5000): Promise<void> {
    // Wait for loading overlay to disappear
    await page.waitForSelector('.loading-overlay', { state: 'hidden', timeout }).catch(() => { });

    // Wait a bit more for animations/transitions
    await page.waitForTimeout(500);

    // Wait for React Flow to be ready
    await page.waitForSelector('.react-flow', { timeout });
}

/**
 * Double-click a node to expand/collapse it
 */
async function toggleNodeExpansion(page: Page, nodeId: string): Promise<void> {
    const nodeSelector = `[data-id="${nodeId}"], .react-flow__node[data-id="${nodeId}"]`;

    // Wait for node to be visible
    await page.waitForSelector(nodeSelector, { timeout: 5000 });

    // Double-click the node
    await page.locator(nodeSelector).dblclick();

    // Wait for layout to update
    await waitForLayoutStable(page);
}

/**
 * Get node by label (helper for finding nodes)
 */
async function findNodeByLabel(page: Page, label: string): Promise<string | null> {
    return await page.evaluate((searchLabel) => {
        const nodes = document.querySelectorAll('.react-flow__node');
        for (const node of Array.from(nodes)) {
            const text = node.textContent || '';
            if (text.includes(searchLabel)) {
                return node.getAttribute('data-id') || node.id || null;
            }
        }
        return null;
    }, label);
}

test.describe('Layout Stability Tests', () => {
    test.beforeEach(async ({ page }) => {
        // Capture browser logs
        page.on('console', msg => console.log(`BROWSER: ${msg.text()}`));

        await page.goto('/?tab=diagram');

        // Wait for app to load
        await page.waitForSelector('.architecture-canvas', { timeout: 10000 });

        // Wait for initial layout
        await waitForLayoutStable(page);
    });

    test('should preserve positions when expanding a single node', async ({ page }) => {
        // Get initial positions
        const initialPositions = await getNodePositionsFromDOM(page);
        expect(initialPositions.size).toBeGreaterThan(0);

        console.log(`Initial nodes: ${initialPositions.size}`);

        // Find a system node to expand (look for nodes with "System" in the text or type="system")
        const systemNodeId = await page.evaluate(() => {
            const nodes = document.querySelectorAll('.react-flow__node');
            for (const node of Array.from(nodes)) {
                const type = node.getAttribute('data-type');
                const id = node.getAttribute('data-id') || node.id;
                if (type === 'system' || node.classList.contains('node-system')) {
                    return id;
                }
            }
            // Fallback: get first node or try known IDs
            const firstNode = nodes[0];
            const fallbackId = document.querySelector('[data-id="API"]')?.getAttribute('data-id') || document.querySelector('[data-id="sruja"]')?.getAttribute('data-id');
            return fallbackId || firstNode?.getAttribute('data-id') || firstNode?.id || null;
        });

        if (!systemNodeId) {
            test.skip();
            return;
        }

        console.log(`Expanding node: ${systemNodeId}`);

        // Expand the node
        await toggleNodeExpansion(page, systemNodeId);

        // Get positions after expansion
        const afterPositions = await getNodePositionsFromDOM(page);

        // Validate position preservation
        const validation = validatePositionPreservation(
            initialPositions,
            Array.from(afterPositions.values()).map(pos => ({
                id: pos.id,
                position: { x: pos.x, y: pos.y },
                data: { type: pos.type },
                parentId: pos.parentId
            } as any)),
            {
                movementThreshold: 100, // Allow 100px movement for root nodes
                ignoreChildren: true // Only check root nodes
            }
        );

        const report = generateValidationReport(validation, 'Single Node Expansion');
        console.log(report);

        // Assert that most nodes are stable (at least 70% should remain stable)
        expect(validation.metrics.stableNodePercentage).toBeGreaterThan(70);

        // Log violations for debugging
        if (validation.violations.length > 0) {
            console.log(`\nViolations found: ${validation.violations.length}`);
            validation.violations.slice(0, 5).forEach(v => {
                console.log(`  - ${v.nodeId}: moved ${v.movement.toFixed(1)}px`);
            });
        }
    });

    test('should preserve positions when collapsing a node', async ({ page }) => {
        // First, expand a node
        const systemNodeId = await page.evaluate(() => {
            const nodes = document.querySelectorAll('.react-flow__node');
            for (const node of Array.from(nodes)) {
                const type = node.getAttribute('data-type');
                const id = node.getAttribute('data-id') || node.id;
                if (type === 'system' || node.classList.contains('node-system')) {
                    return id;
                }
            }
            return null;
        });

        if (!systemNodeId) {
            test.skip();
            return;
        }

        // Expand first
        await toggleNodeExpansion(page, systemNodeId);
        await waitForLayoutStable(page);

        // Get positions after expansion
        const expandedPositions = await getNodePositionsFromDOM(page);

        // Now collapse
        await toggleNodeExpansion(page, systemNodeId);
        await waitForLayoutStable(page);

        // Get positions after collapse
        const collapsedPositions = await getNodePositionsFromDOM(page);

        // Validate - root nodes should be in same positions
        const validation = validatePositionPreservation(
            expandedPositions,
            Array.from(collapsedPositions.values()).map(pos => ({
                id: pos.id,
                position: { x: pos.x, y: pos.y },
                data: { type: pos.type },
                parentId: pos.parentId
            } as any)),
            {
                movementThreshold: 100,
                ignoreChildren: true
            }
        );

        const report = generateValidationReport(validation, 'Node Collapse');
        console.log(report);

        expect(validation.metrics.stableNodePercentage).toBeGreaterThan(70);
    });

    test('should handle multiple expansions without major layout shifts', async ({ page }) => {
        const initialPositions = await getNodePositionsFromDOM(page);

        // Find multiple system nodes
        const systemNodeIds = await page.evaluate(() => {
            const ids: string[] = [];
            const nodes = document.querySelectorAll('.react-flow__node');
            for (const node of Array.from(nodes)) {
                const type = node.getAttribute('data-type');
                const id = node.getAttribute('data-id') || node.id;
                if ((type === 'system' || node.classList.contains('node-system')) && id) {
                    ids.push(id);
                    if (ids.length >= 3) break; // Expand up to 3 nodes
                }
            }
            return ids;
        });

        if (systemNodeIds.length === 0) {
            test.skip();
            return;
        }

        console.log(`Expanding ${systemNodeIds.length} nodes`);

        // Expand each node sequentially
        for (const nodeId of systemNodeIds) {
            await toggleNodeExpansion(page, nodeId);
        }

        const finalPositions = await getNodePositionsFromDOM(page);

        const validation = validatePositionPreservation(
            initialPositions,
            Array.from(finalPositions.values()).map(pos => ({
                id: pos.id,
                position: { x: pos.x, y: pos.y },
                data: { type: pos.type },
                parentId: pos.parentId
            } as any)),
            {
                movementThreshold: 150, // Slightly higher threshold for multiple expansions
                ignoreChildren: true
            }
        );

        const report = generateValidationReport(validation, 'Multiple Expansions');
        console.log(report);

        // With multiple expansions, we expect at least 60% stability
        expect(validation.metrics.stableNodePercentage).toBeGreaterThan(60);
    });

    test('should collect and analyze layout metrics', async ({ page }) => {
        const results: LayoutValidationResult[] = [];

        // Baseline
        const baselinePositions = await getNodePositionsFromDOM(page);
        const baselineValidation = validatePositionPreservation(
            new Map(), // No previous positions
            Array.from(baselinePositions.values()).map(pos => ({
                id: pos.id,
                position: { x: pos.x, y: pos.y },
                data: { type: pos.type },
                parentId: pos.parentId
            } as any)),
            { movementThreshold: 0 }
        );
        results.push(baselineValidation);

        // After first expansion
        const systemNodeId = await page.evaluate(() => {
            const nodes = document.querySelectorAll('.react-flow__node');
            for (const node of Array.from(nodes)) {
                const type = node.getAttribute('data-type');
                const id = node.getAttribute('data-id') || node.id;
                if (type === 'system' || node.classList.contains('node-system')) {
                    return id;
                }
            }
            return null;
        });

        if (systemNodeId) {
            await toggleNodeExpansion(page, systemNodeId);
            const afterFirstPositions = await getNodePositionsFromDOM(page);
            const firstValidation = validatePositionPreservation(
                baselinePositions,
                Array.from(afterFirstPositions.values()).map(pos => ({
                    id: pos.id,
                    position: { x: pos.x, y: pos.y },
                    data: { type: pos.type },
                    parentId: pos.parentId
                } as any)),
                { movementThreshold: 100, ignoreChildren: true }
            );
            results.push(firstValidation);
        }

        // Generate comprehensive report
        console.log('\n=== Comprehensive Layout Analysis ===');
        results.forEach((result, index) => {
            console.log(generateValidationReport(result, `Step ${index + 1}`));
        });

        // Assert overall stability
        const finalResult = results[results.length - 1];
        expect(finalResult.metrics.stableNodePercentage).toBeGreaterThan(60);
    });
});
