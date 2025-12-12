// Comprehensive analysis of all Sruja examples with layout quality scoring
// This test identifies examples with poor scores and helps optimize the layout engine
// Updated to use rule-based optimization system

import { test, expect, Page } from '@playwright/test';
import type { DiagramQualityMetrics } from '../src/utils/diagramQuality';
import {
    calculateDiagramQuality,
    generateQualityReport,
    DEFAULT_QUALITY_WEIGHTS
} from '../src/utils/diagramQuality';
import { getAvailableExamples } from '@sruja/shared';
import * as fs from 'fs';
import * as path from 'path';

interface ExampleAnalysis {
    name: string;
    category: string;
    quality: DiagramQualityMetrics;
    nodeCount: number;
    edgeCount: number;
    hasHierarchy: boolean;
    issues: string[];
    passed: boolean;
}

/**
 * Get nodes and edges from window.__CYBER_GRAPH__
 */
async function getGraphData(page: Page): Promise<{ nodes: any[]; edges: any[] }> {
    return await page.evaluate(() => {
        const graph = (window as any).__CYBER_GRAPH__;
        if (!graph) {
            return { nodes: [], edges: [] };
        }
        return {
            nodes: graph.nodes || [],
            edges: graph.edges || []
        };
    });
}

async function waitForLayoutStable(page: Page, timeout = 15000): Promise<void> {
    await page.waitForSelector('.loading-overlay', { state: 'hidden', timeout }).catch(() => {});
    await page.waitForFunction(() => {
        const graph = (window as any).__CYBER_GRAPH__;
        return graph && graph.nodes && graph.nodes.length > 0;
    }, { timeout });
    await page.waitForTimeout(2000); // Wait for animations
}

async function loadExample(page: Page, exampleName: string): Promise<void> {
    try {
        // Click examples dropdown
        const examplesButton = page.locator('button:has-text("Examples")').first();
        if (await examplesButton.count() > 0) {
            await examplesButton.click();
            await page.waitForTimeout(500);
        }
        
        // Find and click the example
        const exampleButton = page.locator(`.example-item:has-text("${exampleName}")`).first();
        if (await exampleButton.count() > 0) {
            await exampleButton.scrollIntoViewIfNeeded();
            await exampleButton.click();
            await waitForLayoutStable(page);
            return;
        }
    } catch (error) {
        // Fallback: try loading via URL if examples menu doesn't work
        console.warn(`Could not load example via menu, trying URL: ${exampleName}`);
    }
}

async function ensureSrujaLayout(page: Page): Promise<void> {
    const layoutSelector = page.locator('.layout-selector select, select[name="layout"]');
    if (await layoutSelector.count() > 0) {
        const currentValue = await layoutSelector.inputValue();
        if (currentValue !== 'sruja') {
            await layoutSelector.selectOption('sruja');
            await waitForLayoutStable(page);
        }
    }
}

test.describe('Comprehensive Examples Analysis', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('/');
        await page.waitForSelector('.architecture-canvas', { timeout: 15000 });
        await waitForLayoutStable(page);
    });

    test('analyze all examples and identify optimization opportunities', async ({ page }) => {
        const examples = await getAvailableExamples();
        const results: ExampleAnalysis[] = [];
        const outputDir = path.join(__dirname, 'results');
        if (!fs.existsSync(outputDir)) {
            fs.mkdirSync(outputDir, { recursive: true });
        }

        console.log(`\n=== Analyzing ${examples.length} Examples ===\n`);

        for (const example of examples) {
            try {
                console.log(`Analyzing: ${example.name} (${example.category})`);
                
                await loadExample(page, example.name);
                await ensureSrujaLayout(page);
                
                // Get graph data
                const { nodes, edges } = await getGraphData(page);
                
                if (nodes.length === 0) {
                    console.log(`  ⚠️  No nodes found, skipping`);
                    continue;
                }

                const viewportSize = await page.viewportSize() || { width: 1920, height: 1080 };
                const quality = calculateDiagramQuality(nodes, edges, viewportSize, DEFAULT_QUALITY_WEIGHTS);
                
                // Identify issues
                const issues: string[] = [];
                if (quality.overlappingNodes.length > 0) {
                    issues.push(`${quality.overlappingNodes.length} node overlaps`);
                }
                if (quality.parentChildContainment.length > 0) {
                    issues.push(`🚨 ${quality.parentChildContainment.length} containment violations`);
                }
                if (quality.edgeCrossings > 5) {
                    issues.push(`${quality.edgeCrossings} edge crossings`);
                }
                if (quality.edgesOverNodes > 3) {
                    issues.push(`${quality.edgesOverNodes} edges over nodes`);
                }
                if (quality.edgeLabelOverlaps > 0) {
                    issues.push(`${quality.edgeLabelOverlaps} edge label overlaps`);
                }
                if (quality.clippedNodeLabels > 0) {
                    issues.push(`${quality.clippedNodeLabels} clipped labels`);
                }
                if (quality.emptySpaceScore < 70) {
                    issues.push(`Poor space utilization (${quality.emptySpaceScore.toFixed(0)})`);
                }
                if (quality.hierarchyScore < 70) {
                    issues.push(`Hierarchy issues (${quality.hierarchyScore.toFixed(0)})`);
                }
                
                const hasHierarchy = nodes.some(n => n.parentId);
                const passed = quality.weightedScore >= 75 && quality.grade !== 'F';
                
                results.push({
                    name: example.name,
                    category: example.category,
                    quality,
                    nodeCount: nodes.length,
                    edgeCount: edges.length,
                    hasHierarchy,
                    issues,
                    passed
                });
                
                const status = passed ? '✅' : '❌';
                console.log(`  ${status} Score: ${quality.weightedScore.toFixed(1)}/100 (${quality.grade}) - ${nodes.length} nodes, ${edges.length} edges`);
                if (issues.length > 0) {
                    console.log(`     Issues: ${issues.join(', ')}`);
                }
                
            } catch (error) {
                console.error(`  ❌ Error analyzing ${example.name}:`, error);
            }
            
            await page.waitForTimeout(500);
        }

        // Generate comprehensive report
        const reportPath = path.join(outputDir, `examples-analysis-${Date.now()}.json`);
        fs.writeFileSync(reportPath, JSON.stringify(results, null, 2));
        console.log(`\n✅ Analysis saved to: ${reportPath}`);

        // Summary statistics
        console.log('\n=== Summary Statistics ===\n');
        const totalPassed = results.filter(r => r.passed).length;
        const avgScore = results.reduce((sum, r) => sum + r.quality.weightedScore, 0) / results.length;
        const byCategory = results.reduce((acc, r) => {
            if (!acc[r.category]) {
                acc[r.category] = { total: 0, passed: 0, scores: [] };
            }
            acc[r.category].total++;
            if (r.passed) acc[r.category].passed++;
            acc[r.category].scores.push(r.quality.weightedScore);
            return acc;
        }, {} as Record<string, { total: number; passed: number; scores: number[] }>);

        console.log(`Total: ${totalPassed}/${results.length} passed (${((totalPassed / results.length) * 100).toFixed(1)}%)`);
        console.log(`Average Score: ${avgScore.toFixed(1)}/100`);
        
        console.log('\n=== By Category ===\n');
        Object.entries(byCategory).forEach(([category, stats]) => {
            const avg = stats.scores.reduce((a, b) => a + b, 0) / stats.scores.length;
            console.log(`${category}: ${stats.passed}/${stats.total} passed, avg: ${avg.toFixed(1)}`);
        });

        // Find problematic examples
        const problematic = results.filter(r => !r.passed || r.quality.weightedScore < 70);
        if (problematic.length > 0) {
            console.log('\n=== Problematic Examples (Need Optimization) ===\n');
            problematic.forEach(r => {
                console.log(`${r.name}: ${r.quality.weightedScore.toFixed(1)}/100 (${r.quality.grade})`);
                console.log(`  Nodes: ${r.nodeCount}, Edges: ${r.edgeCount}, Hierarchy: ${r.hasHierarchy ? 'Yes' : 'No'}`);
                if (r.issues.length > 0) {
                    console.log(`  Issues: ${r.issues.join(', ')}`);
                }
                console.log('');
            });
        }

        // Find examples with specific issues
        const withContainment = results.filter(r => r.quality.parentChildContainment.length > 0);
        if (withContainment.length > 0) {
            console.log(`\n=== Examples with Containment Violations (${withContainment.length}) ===\n`);
            withContainment.forEach(r => {
                console.log(`${r.name}: ${r.quality.parentChildContainment.length} violations`);
            });
        }

        const withOverlaps = results.filter(r => r.quality.overlappingNodes.length > 0);
        if (withOverlaps.length > 0) {
            console.log(`\n=== Examples with Node Overlaps (${withOverlaps.length}) ===\n`);
            withOverlaps.forEach(r => {
                console.log(`${r.name}: ${r.quality.overlappingNodes.length} overlaps`);
            });
        }

        // Generate markdown report
        const markdownReport = generateMarkdownReport(results);
        const reportMdPath = path.join(outputDir, `examples-analysis-${Date.now()}.md`);
        fs.writeFileSync(reportMdPath, markdownReport);
        console.log(`\n✅ Markdown report saved to: ${reportMdPath}`);

        // Assert that at least 60% pass (we'll optimize to improve this)
        const passRate = totalPassed / results.length;
        expect(passRate).toBeGreaterThan(0.6);
    });
});

function generateMarkdownReport(results: ExampleAnalysis[]): string {
    const lines: string[] = [];
    lines.push('# Comprehensive Examples Analysis Report');
    lines.push(`\nGenerated: ${new Date().toISOString()}\n`);
    
    const total = results.length;
    const passed = results.filter(r => r.passed).length;
    const avgScore = results.reduce((sum, r) => sum + r.quality.weightedScore, 0) / total;
    
    lines.push('## Summary');
    lines.push(`- Total Examples: ${total}`);
    lines.push(`- Passed (≥75%): ${passed} (${((passed / total) * 100).toFixed(1)}%)`);
    lines.push(`- Average Score: ${avgScore.toFixed(1)}/100\n`);
    
    // Group by category
    const byCategory = results.reduce((acc, r) => {
        if (!acc[r.category]) acc[r.category] = [];
        acc[r.category].push(r);
        return acc;
    }, {} as Record<string, ExampleAnalysis[]>);
    
    lines.push('## By Category\n');
    Object.entries(byCategory).forEach(([category, examples]) => {
        const avg = examples.reduce((sum, e) => sum + e.quality.weightedScore, 0) / examples.length;
        const passedCount = examples.filter(e => e.passed).length;
        lines.push(`### ${category}`);
        lines.push(`- Examples: ${examples.length}`);
        lines.push(`- Passed: ${passedCount} (${((passedCount / examples.length) * 100).toFixed(1)}%)`);
        lines.push(`- Average Score: ${avg.toFixed(1)}/100\n`);
    });
    
    // Problematic examples
    const problematic = results.filter(r => !r.passed || r.quality.weightedScore < 70);
    if (problematic.length > 0) {
        lines.push('## Examples Requiring Optimization\n');
        problematic.sort((a, b) => a.quality.weightedScore - b.quality.weightedScore);
        problematic.forEach(r => {
            lines.push(`### ${r.name} (${r.quality.weightedScore.toFixed(1)}/100, ${r.quality.grade})`);
            lines.push(`- Category: ${r.category}`);
            lines.push(`- Nodes: ${r.nodeCount}, Edges: ${r.edgeCount}`);
            lines.push(`- Has Hierarchy: ${r.hasHierarchy ? 'Yes' : 'No'}`);
            if (r.issues.length > 0) {
                lines.push(`- Issues:`);
                r.issues.forEach(issue => lines.push(`  - ${issue}`));
            }
            lines.push('');
        });
    }
    
    return lines.join('\n');
}
