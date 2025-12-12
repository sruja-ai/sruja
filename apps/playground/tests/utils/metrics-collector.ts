// Metrics collection utility for layout optimization
import type { Page } from '@playwright/test';
import type { DiagramQualityMetrics } from '../../src/utils/diagramQuality';
import { calculateDiagramQuality, DEFAULT_QUALITY_WEIGHTS } from '../../src/utils/diagramQuality';
import { selectLayoutConfig } from '../../src/utils/layoutRules';

export interface LayoutMetrics {
    // Example info
    exampleName: string;
    category: string;
    timestamp: string;

    // Quality scores
    weightedScore: number;
    overallScore: number;
    grade: 'A' | 'B' | 'C' | 'D' | 'F';

    // Specific metrics
    overlappingNodes: number;
    edgeCrossings: number;
    edgesOverNodes: number;
    edgeBends: number;
    spacingViolations: number;
    parentChildViolations: number;

    // Layout characteristics
    nodeCount: number;
    edgeCount: number;
    hasHierarchy: boolean;
    currentLevel: string;
    selectedEngine: 'sruja' | 'c4level';
    selectedDirection: string;

    // Performance
    layoutTime: number; // milliseconds
    renderTime: number;

    // Visual metrics
    viewportUtilization: number;
    aspectRatio: number;
    diagramBounds: { x: number; y: number; width: number; height: number };

    // Full quality metrics
    qualityMetrics: DiagramQualityMetrics;
}

export interface MetricsCollectionOptions {
    includePerformance?: boolean;
    includeFullMetrics?: boolean;
}

/**
 * Get nodes from React Flow
 */
async function getReactFlowNodes(page: Page): Promise<any[]> {
    return await page.evaluate(() => {
        // Prefer exposed state for accuracy
        const exposed = (window as any).__CYBER_GRAPH__;
        if (exposed?.nodes) {
            return exposed.nodes;
        }

        // Fallback to DOM (legacy)
        const nodes: any[] = [];
        const nodeElements = document.querySelectorAll('.react-flow__node');
        const viewport = document.querySelector('.react-flow__viewport') as HTMLElement;

        // Calculate viewport scale to normalize dimensions
        let scale = 1;
        if (viewport) {
            const transform = viewport.style.transform || window.getComputedStyle(viewport).transform;
            if (transform && transform !== 'none') {
                // Try to parse "scale(s)" or matrix
                const match = transform.match(/scale\(([0-9.]+)\)/);
                if (match) {
                    scale = parseFloat(match[1]);
                } else if (transform.startsWith('matrix')) {
                    // matrix(a, b, c, d, tx, ty) - scale is usually 'a' (and 'd') if no rotation
                    const values = transform.split('(')[1].split(')')[0].split(',');
                    const a = parseFloat(values[0]);
                    const b = parseFloat(values[1]);
                    // Scale is magnitude of column vector [a, b]
                    scale = Math.sqrt(a * a + b * b);
                }
            }
        }

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
            // Normalize dimensions by viewport scale to match canvas coordinate system
            const width = (rect.width || 100) / scale;
            const height = (rect.height || 100) / scale;

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

async function getReactFlowEdges(page: Page): Promise<any[]> {
    return await page.evaluate(() => {
        // Prefer exposed state for accuracy
        const exposed = (window as any).__CYBER_GRAPH__;
        if (exposed?.edges) {
            return exposed.edges;
        }

        const edges: any[] = [];
        // React Flow wraps each edge in a group with class .react-flow__edge
        // If not found, fall back to looking for paths in the edges layer
        let edgeElements = document.querySelectorAll('.react-flow__edge');

        if (edgeElements.length === 0) {
            // Fallback: look for groups inside the edges layer
            edgeElements = document.querySelectorAll('.react-flow__edges > g');
        }

        edgeElements.forEach((el) => {
            const source = el.getAttribute('data-source');
            const target = el.getAttribute('data-target');
            const id = el.getAttribute('data-id') || `${source}-${target}`;

            if (source && target) {
                edges.push({
                    id,
                    source,
                    target,
                    data: {}
                });
            } else {
                // If data attributes are missing on the group (common in some versions),
                // we might not be able to reconstruct the graph purely from DOM without
                // exposing the internal state.
                // However, standard React Flow adds these attributes to the group.
            }
        });

        return edges;
    });
}

async function waitForLayoutStable(page: Page, timeout = 30000): Promise<void> {
    // Wait for loading overlay to disappear (if present)
    await page.waitForSelector('.loading-overlay', { state: 'hidden', timeout: timeout / 2 }).catch(() => { });

    // Wait for React Flow to be ready
    await page.waitForSelector('.react-flow', { timeout }).catch(() => {
        // If React Flow selector fails, try alternative selectors
        return page.waitForSelector('.react-flow__node', { timeout: timeout / 2 }).catch(() => { });
    });

    // Additional wait for layout to stabilize
    await page.waitForTimeout(1000);
}

/**
 * Collect comprehensive layout metrics for an example
 */
export async function collectLayoutMetrics(
    page: Page,
    exampleName: string,
    category: string,
    options: MetricsCollectionOptions = {}
): Promise<LayoutMetrics> {
    const { includePerformance = true, includeFullMetrics = true } = options;

    const startTime = Date.now();

    // Ensure Diagram tab is active
    const diagramTab = page.locator('button.view-tab:has-text("Diagram")').first();
    await diagramTab.waitFor({ timeout: 5000 }).catch(() => { });
    await diagramTab.click().catch(() => { });
    // Wait for layout to stabilize
    await waitForLayoutStable(page);

    const renderTime = Date.now() - startTime;

    // Get nodes and edges
    const nodes = await getReactFlowNodes(page);
    const edges = await getReactFlowEdges(page);

    if (nodes.length === 0) {
        throw new Error(`No nodes found for example: ${exampleName}`);
    }

    // Get current level
    const currentLevel = await page.evaluate(() => {
        const indicator = document.querySelector('.level-indicator');
        if (indicator) {
            const text = indicator.textContent || '';
            if (text.includes('L0')) return 'L0';
            if (text.includes('L1')) return 'L1';
            if (text.includes('L2')) return 'L2';
            if (text.includes('L3')) return 'L3';
        }
        return 'L1';
    });

    // Get viewport size
    const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };

    // Calculate quality metrics
    const qualityMetrics = calculateDiagramQuality(
        nodes,
        edges,
        viewportSize,
        DEFAULT_QUALITY_WEIGHTS
    );

    // Determine selected layout config
    const config = selectLayoutConfig(
        nodes,
        edges,
        currentLevel as any,
        undefined,
        undefined,
        new Set()
    );

    // Calculate diagram bounds
    const diagramBounds = nodes.reduce((bounds, node) => {
        const right = node.position.x + node.width;
        const bottom = node.position.y + node.height;
        return {
            x: Math.min(bounds.x, node.position.x),
            y: Math.min(bounds.y, node.position.y),
            width: Math.max(bounds.width, right - bounds.x),
            height: Math.max(bounds.height, bottom - bounds.y)
        };
    }, { x: Infinity, y: Infinity, width: 0, height: 0 });

    const aspectRatio = diagramBounds.width / diagramBounds.height || 1;
    const viewportUtilization = Math.min(
        (diagramBounds.width * diagramBounds.height) / (viewportSize.width * viewportSize.height),
        1
    );

    // Check for hierarchy
    const hasHierarchy = nodes.some(n => n.parentId);

    // Measure layout time (if performance tracking enabled)
    let layoutTime = 0;
    if (includePerformance) {
        layoutTime = await page.evaluate(() => {
            // Try to get layout time from performance marks
            const perfEntries = performance.getEntriesByType('measure');
            const layoutEntry = perfEntries.find(e => e.name.includes('layout'));
            return layoutEntry ? layoutEntry.duration : 0;
        });
    }

    return {
        exampleName,
        category,
        timestamp: new Date().toISOString(),
        weightedScore: qualityMetrics.weightedScore,
        overallScore: qualityMetrics.overallScore,
        grade: qualityMetrics.grade,
        overlappingNodes: qualityMetrics.overlappingNodes.length,
        edgeCrossings: qualityMetrics.edgeCrossings,
        edgesOverNodes: qualityMetrics.edgesOverNodes,
        edgeBends: qualityMetrics.edgeBends,
        spacingViolations: qualityMetrics.spacingViolations.length,
        parentChildViolations: qualityMetrics.parentChildContainment.length,
        nodeCount: nodes.length,
        edgeCount: edges.length,
        hasHierarchy,
        currentLevel,
        selectedEngine: config.engine,
        selectedDirection: config.direction,
        layoutTime,
        renderTime,
        viewportUtilization,
        aspectRatio,
        diagramBounds,
        qualityMetrics: includeFullMetrics ? qualityMetrics : {} as DiagramQualityMetrics
    };
}

/**
 * Collect baseline metrics for all examples
 */
export async function collectBaselineForAllExamples(
    page: Page,
    examples: Array<{ name: string; category: string }>,
    options: MetricsCollectionOptions = {}
): Promise<Map<string, LayoutMetrics>> {
    const results = new Map<string, LayoutMetrics>();

    for (const example of examples) {
        try {
            console.log(`Collecting metrics for: ${example.name}`);

            // Load example
            await page.click('button:has-text("Examples")');
            await page.waitForTimeout(500);

            const exampleButton = page.locator(`.example-item:has-text("${example.name}")`).first();
            await exampleButton.waitFor({ timeout: 5000 }).catch(() => { });
            await exampleButton.scrollIntoViewIfNeeded();
            await exampleButton.click();
            // Switch to Diagram tab so canvas is rendered
            const diagramTab = page.locator('button.view-tab:has-text("Diagram")').first();
            await diagramTab.waitFor({ timeout: 5000 }).catch(() => { });
            await diagramTab.click();

            await waitForLayoutStable(page);

            // Collect metrics
            const metrics = await collectLayoutMetrics(page, example.name, example.category, options);
            results.set(example.name, metrics);

            console.log(`  Score: ${metrics.weightedScore.toFixed(1)}/100 (${metrics.grade})`);

        } catch (error) {
            console.error(`Failed to collect metrics for ${example.name}:`, error);
        }

        await page.waitForTimeout(500);
    }

    return results;
}

/**
 * Save metrics to JSON file
 */
export function saveMetricsToFile(
    metrics: Map<string, LayoutMetrics>,
    filepath: string
): void {
    const data = {
        timestamp: new Date().toISOString(),
        metrics: Array.from(metrics.entries()).map(([_, m]) => ({
            ...m
        }))
    };

    // Note: In Node.js environment, use fs.writeFileSync
    // This is a placeholder - actual implementation depends on environment
    // Note: In Node.js environment, use fs.writeFileSync
    // This is a placeholder - actual implementation depends on environment
    console.log(`Would save ${metrics.size} metrics to ${filepath}`);
}
