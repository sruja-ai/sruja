import { test, expect, Page } from '@playwright/test';
import type { DiagramQualityMetrics } from '../src/utils/diagramQuality';
import {
    calculateDiagramQuality,
    DEFAULT_QUALITY_WEIGHTS
} from '../src/utils/diagramQuality';
import { getAvailableExamples, loadExampleFile } from '@sruja/shared';
import LZString from 'lz-string';
import { selectLayoutConfig } from '../src/utils/layoutRules';

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
    await page.waitForTimeout(1500);
    await page.waitForSelector('.react-flow', { timeout });
}

async function loadExample(page: Page, exampleName: string): Promise<void> {
    const examples = await getAvailableExamples();
    const ex = examples.find(e => e.name === exampleName);
    if (!ex) return;
    const content = await loadExampleFile(ex.file);
    const compressed = LZString.compressToBase64(content);
    await page.goto(`/?tab=diagram&share=${encodeURIComponent(compressed)}`);
    await waitForLayoutStable(page);
}

async function ensureSrujaOrC4LevelLayout(page: Page): Promise<void> {
    const layoutSelector = page.locator('.layout-selector select');
    if (await layoutSelector.count() > 0) {
        const currentValue = await layoutSelector.inputValue();
        if (currentValue !== 'sruja' && currentValue !== 'c4level') {
            await layoutSelector.selectOption('sruja');
            await waitForLayoutStable(page);
        }
    }
}

test.describe('Rules-Based Layout Tests', () => {
    test.beforeEach(async ({ page }) => {
        const fallbackDsl = `architecture "Demo" {\n  persons { user "User" }\n  systems { web "WebApp" }\n  relations { user -> web "uses" }\n}`;
        const compressed = LZString.compressToBase64(fallbackDsl);
        await page.goto(`/?tab=diagram&share=${encodeURIComponent(compressed)}`);
        await waitForLayoutStable(page);
    });

    test('rules should select appropriate layout for different examples', async ({ page }) => {
        const examples = await getAvailableExamples();
        const ruleResults: Array<{
            example: string;
            selectedEngine: string;
            selectedDirection: string;
            quality: number;
        }> = [];

        for (const example of examples.slice(0, 15)) {
            try {
                await loadExample(page, example.name);
                await ensureSrujaOrC4LevelLayout(page);
                
                const nodes = await getReactFlowNodes(page);
                const edges = await getReactFlowEdges(page);
                
                if (nodes.length === 0) continue;
                
                // Get current level from page
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
                
                // Check what rule would select
                const config = selectLayoutConfig(
                    nodes,
                    edges,
                    currentLevel as any,
                    undefined,
                    undefined,
                    new Set()
                );
                
                const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
                const quality = calculateDiagramQuality(nodes, edges, viewportSize);
                
                ruleResults.push({
                    example: example.name,
                    selectedEngine: config.engine,
                    selectedDirection: config.direction,
                    quality: quality.weightedScore
                });
                
                console.log(`${example.name}: ${config.engine}-${config.direction} -> ${quality.weightedScore.toFixed(1)}`);
                
            } catch (error) {
                console.error(`Error testing ${example.name}:`, error);
            }
            
            await page.waitForTimeout(500);
        }

        // Analyze rule effectiveness
        console.log('\n=== Rule Selection Analysis ===\n');
        const byEngine = ruleResults.reduce((acc, r) => {
            if (!acc[r.selectedEngine]) {
                acc[r.selectedEngine] = [];
            }
            acc[r.selectedEngine].push(r.quality);
            return acc;
        }, {} as Record<string, number[]>);

        Object.entries(byEngine).forEach(([engine, scores]) => {
            const avg = scores.reduce((a, b) => a + b, 0) / scores.length;
            console.log(`${engine}: avg score ${avg.toFixed(1)} (${scores.length} examples)`);
        });

        // Rules should produce good results
        const avgQuality = ruleResults.reduce((sum, r) => sum + r.quality, 0) / ruleResults.length;
        expect(avgQuality).toBeGreaterThan(65);
    });

    test('test all examples with rules-based layout', async ({ page }) => {
        const examples = await getAvailableExamples();
        const results: Array<{
            example: string;
            category: string;
            quality: DiagramQualityMetrics;
            passed: boolean;
        }> = [];

        console.log(`\n=== Testing ${examples.length} Examples with Rules-Based Layout ===\n`);

        for (const example of examples) {
            try {
                console.log(`Testing: ${example.name} (${example.category})`);
                
                await loadExample(page, example.name);
                await ensureSrujaOrC4LevelLayout(page);
                
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
            }
            
            await page.waitForTimeout(500);
        }

        // Generate report
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

        const totalPassed = results.filter(r => r.passed).length;
        const avgScore = results.reduce((sum, r) => sum + (r.quality.weightedScore || 0), 0) / results.length;
        
        console.log(`\nOverall: ${totalPassed}/${results.length} passed, avg: ${avgScore.toFixed(1)}/100\n`);

        // At least 70% should pass
        const passRate = totalPassed / results.length;
        expect(passRate).toBeGreaterThan(0.7);
    });
});
