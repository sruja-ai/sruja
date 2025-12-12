import { test, expect, Page } from '@playwright/test';
import type { DiagramQualityMetrics } from '../src/utils/diagramQuality';
import {
    calculateDiagramQuality,
    generateQualityReport,
    DEFAULT_QUALITY_WEIGHTS
} from '../src/utils/diagramQuality';
import { getAvailableExamples } from '@sruja/shared';

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
                    target,
                    data: {}
                });
            }
        });
        
        return edges;
    });
}

async function waitForLayoutStable(page: Page, timeout = 10000): Promise<void> {
    await page.waitForSelector('.loading-overlay', { state: 'hidden', timeout }).catch(() => {});
    await page.waitForTimeout(1500); // Wait for animations
    await page.waitForSelector('.react-flow', { timeout });
}

/**
 * Load an example and wait for it to render
 */
async function loadExample(page: Page, exampleName: string): Promise<void> {
    // Click examples dropdown
    await page.click('button:has-text("Examples")');
    await page.waitForTimeout(500);
    
    // Find and click the example (may be in a category)
    const exampleButton = page.locator(`.example-item:has-text("${exampleName}")`).first();
    await exampleButton.waitFor({ timeout: 5000 }).catch(() => {});
    
    // Scroll into view if needed
    await exampleButton.scrollIntoViewIfNeeded();
    await exampleButton.click();
    
    // Wait for layout
    await waitForLayoutStable(page);
}

/**
 * Ensure we're using Sruja or C4Level layout (not ELK)
 */
async function ensureCorrectLayoutEngine(page: Page): Promise<void> {
    // Check if layout selector exists and set to sruja or c4level
    const layoutSelector = page.locator('.layout-selector select');
    if (await layoutSelector.count() > 0) {
        const currentValue = await layoutSelector.inputValue();
        if (currentValue !== 'sruja' && currentValue !== 'c4level') {
            await layoutSelector.selectOption('sruja');
            await waitForLayoutStable(page);
        }
    }
}

test.describe('All Examples Quality Tests', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('/');
        await page.waitForSelector('.architecture-canvas', { timeout: 10000 });
        await waitForLayoutStable(page);
    });

    test('test all examples and collect quality metrics', async ({ page }) => {
        const examples = await getAvailableExamples();
        const results: Array<{
            example: string;
            category: string;
            quality: DiagramQualityMetrics;
            passed: boolean;
        }> = [];

        console.log(`\n=== Testing ${examples.length} Examples ===\n`);

        for (const example of examples) {
            try {
                console.log(`Testing: ${example.name} (${example.category})`);
                
                // Load the example
                await loadExample(page, example.name);
                
                // Ensure we're using our layout engines (sruja or c4level)
                await ensureCorrectLayoutEngine(page);
                
                // Get nodes and edges
                const nodes = await getReactFlowNodes(page);
                const edges = await getReactFlowEdges(page);
                
                if (nodes.length === 0) {
                    console.log(`  ⚠️  No nodes found, skipping`);
                    continue;
                }
                
                const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
                const quality = calculateDiagramQuality(nodes, edges, viewportSize, DEFAULT_QUALITY_WEIGHTS);
                
                const passed = quality.weightedScore >= 70 && quality.grade !== 'F';
                
                results.push({
                    example: example.name,
                    category: example.category,
                    quality,
                    passed
                });
                
                const status = passed ? '✅' : '❌';
                console.log(`  ${status} Score: ${quality.weightedScore.toFixed(1)}/100 (${quality.grade})`);
                console.log(`     Overlaps: ${quality.overlappingNodes.length}, Crossings: ${quality.edgeCrossings}, Edges over nodes: ${quality.edgesOverNodes}`);
                
            } catch (error) {
                console.error(`  ❌ Error testing ${example.name}:`, error);
                results.push({
                    example: example.name,
                    category: example.category,
                    quality: {} as DiagramQualityMetrics,
                    passed: false
                });
            }
            
            // Small delay between examples
            await page.waitForTimeout(500);
        }

        // Generate comprehensive report
        console.log('\n=== Quality Summary by Category ===\n');
        
        const byCategory = results.reduce((acc, r) => {
            if (!acc[r.category]) {
                acc[r.category] = [];
            }
            acc[r.category].push(r);
            return acc;
        }, {} as Record<string, typeof results>);

        Object.entries(byCategory).forEach(([category, categoryResults]) => {
            const avgScore = categoryResults.reduce((sum, r) => sum + (r.quality.weightedScore || 0), 0) / categoryResults.length;
            const passedCount = categoryResults.filter(r => r.passed).length;
            console.log(`${category}: ${passedCount}/${categoryResults.length} passed, avg score: ${avgScore.toFixed(1)}`);
        });

        console.log('\n=== Overall Statistics ===\n');
        const totalPassed = results.filter(r => r.passed).length;
        const avgScore = results.reduce((sum, r) => sum + (r.quality.weightedScore || 0), 0) / results.length;
        console.log(`Total: ${totalPassed}/${results.length} passed`);
        console.log(`Average Score: ${avgScore.toFixed(1)}/100`);

        // Find problematic examples
        const problematic = results.filter(r => !r.passed || (r.quality.weightedScore || 0) < 60);
        if (problematic.length > 0) {
            console.log('\n=== Problematic Examples ===\n');
            problematic.forEach(r => {
                console.log(`${r.example}: ${r.quality.weightedScore?.toFixed(1) || 'N/A'}/100`);
                if (r.quality.overlappingNodes && r.quality.overlappingNodes.length > 0) {
                    console.log(`  - ${r.quality.overlappingNodes.length} overlaps`);
                }
                if (r.quality.edgeCrossings && r.quality.edgeCrossings > 5) {
                    console.log(`  - ${r.quality.edgeCrossings} edge crossings`);
                }
                if (r.quality.edgesOverNodes && r.quality.edgesOverNodes > 3) {
                    console.log(`  - ${r.quality.edgesOverNodes} edges over nodes`);
                }
            });
        }

        // Assert that at least 70% of examples pass
        const passRate = totalPassed / results.length;
        expect(passRate).toBeGreaterThan(0.7);
    });

    test('identify patterns in high-quality vs low-quality diagrams', async ({ page }) => {
        const examples = await getAvailableExamples();
        const highQuality: any[] = [];
        const lowQuality: any[] = [];

        for (const example of examples.slice(0, 10)) { // Test first 10 for pattern analysis
            try {
                await loadExample(page, example.name);
                await ensureCorrectLayoutEngine(page);
                const nodes = await getReactFlowNodes(page);
                const edges = await getReactFlowEdges(page);
                
                if (nodes.length === 0) continue;
                
                const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
                const quality = calculateDiagramQuality(nodes, edges, viewportSize);
                
                const metrics = {
                    name: example.name,
                    nodeCount: nodes.length,
                    edgeCount: edges.length,
                    hasChildren: nodes.some(n => n.parentId),
                    quality: quality.weightedScore,
                    overlaps: quality.overlappingNodes.length,
                    crossings: quality.edgeCrossings,
                    edgesOverNodes: quality.edgesOverNodes,
                    aspectRatio: quality.aspectRatio
                };
                
                if (quality.weightedScore >= 80) {
                    highQuality.push(metrics);
                } else if (quality.weightedScore < 70) {
                    lowQuality.push(metrics);
                }
            } catch (error) {
                console.error(`Error testing ${example.name}:`, error);
            }
            
            await page.waitForTimeout(500);
        }

        console.log('\n=== Pattern Analysis ===\n');
        console.log('High Quality Examples:');
        highQuality.forEach(m => {
            console.log(`  ${m.name}: ${m.nodeCount} nodes, ${m.edgeCount} edges, score: ${m.quality.toFixed(1)}`);
        });
        
        console.log('\nLow Quality Examples:');
        lowQuality.forEach(m => {
            console.log(`  ${m.name}: ${m.nodeCount} nodes, ${m.edgeCount} edges, score: ${m.quality.toFixed(1)}`);
            console.log(`    Issues: ${m.overlaps} overlaps, ${m.crossings} crossings, ${m.edgesOverNodes} edges over nodes`);
        });
    });
});
