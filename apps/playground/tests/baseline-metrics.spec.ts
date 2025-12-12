import { test, expect } from '@playwright/test';
import { getAvailableExamples } from '@sruja/shared';
import {
    collectLayoutMetrics,
    collectBaselineForAllExamples,
    type LayoutMetrics
} from './utils/metrics-collector';
import * as fs from 'fs';
import * as path from 'path';

test.describe('Baseline Metrics Collection', () => {
    test.setTimeout(180000);
    test.beforeEach(async ({ page }) => {
        await page.goto('/?tab=diagram');
        await page.waitForSelector('.app', { timeout: 30000 });
        await page.waitForTimeout(1000);
    });

    test('collect baseline metrics for all examples', async ({ page }) => {
        const examples = await getAvailableExamples();
        
        console.log(`\n=== Collecting Baseline Metrics for ${examples.length} Examples ===\n`);
        
        const results = await collectBaselineForAllExamples(page, examples, {
            includePerformance: true,
            includeFullMetrics: false // Exclude full metrics to reduce file size
        });
        
        // Save results
        const resultsDir = path.join(process.cwd(), 'tests', 'results');
        if (!fs.existsSync(resultsDir)) {
            fs.mkdirSync(resultsDir, { recursive: true });
        }
        
        const resultsFile = path.join(resultsDir, 'baseline-metrics.json');
        const data = {
            timestamp: new Date().toISOString(),
            totalExamples: examples.length,
            collectedMetrics: results.size,
            metrics: Array.from(results.entries()).map(([name, m]) => ({
                exampleName: name,
                category: m.category,
                weightedScore: m.weightedScore,
                overallScore: m.overallScore,
                grade: m.grade,
                overlappingNodes: m.overlappingNodes,
                edgeCrossings: m.edgeCrossings,
                edgesOverNodes: m.edgesOverNodes,
                edgeBends: m.edgeBends,
                spacingViolations: m.spacingViolations,
                parentChildViolations: m.parentChildViolations,
                nodeCount: m.nodeCount,
                edgeCount: m.edgeCount,
                hasHierarchy: m.hasHierarchy,
                currentLevel: m.currentLevel,
                selectedEngine: m.selectedEngine,
                selectedDirection: m.selectedDirection,
                layoutTime: m.layoutTime,
                renderTime: m.renderTime,
                viewportUtilization: m.viewportUtilization,
                aspectRatio: m.aspectRatio
            }))
        };
        
        fs.writeFileSync(resultsFile, JSON.stringify(data, null, 2));
        console.log(`\n✅ Saved baseline metrics to ${resultsFile}`);
        
        // Generate summary
        const scores = Array.from(results.values()).map(m => m.weightedScore);
        const avgScore = scores.reduce((a, b) => a + b, 0) / scores.length;
        const minScore = Math.min(...scores);
        const maxScore = Math.max(...scores);
        const passingCount = scores.filter(s => s >= 70).length;
        const passingRate = (passingCount / scores.length) * 100;
        
        console.log('\n=== Baseline Summary ===');
        console.log(`Total Examples: ${results.size}`);
        console.log(`Average Score: ${avgScore.toFixed(1)}/100`);
        console.log(`Min Score: ${minScore.toFixed(1)}/100`);
        console.log(`Max Score: ${maxScore.toFixed(1)}/100`);
        console.log(`Passing (≥70): ${passingCount}/${results.size} (${passingRate.toFixed(1)}%)`);
        
        // Categorize by grade
        const byGrade = Array.from(results.values()).reduce((acc, m) => {
            acc[m.grade] = (acc[m.grade] || 0) + 1;
            return acc;
        }, {} as Record<string, number>);
        
        console.log('\nGrade Distribution:');
        Object.entries(byGrade).forEach(([grade, count]) => {
            console.log(`  ${grade}: ${count}`);
        });
        
        // Assertions
        expect(results.size).toBeGreaterThan(0);
        expect(avgScore).toBeGreaterThan(0);
        
        // Log failing examples
        const failing = Array.from(results.entries())
            .filter(([_, m]) => m.weightedScore < 70)
            .sort((a, b) => a[1].weightedScore - b[1].weightedScore)
            .slice(0, 10);
        
        if (failing.length > 0) {
            console.log('\n⚠️  Top 10 Lowest Scoring Examples:');
            failing.forEach(([name, m]) => {
                console.log(`  ${name}: ${m.weightedScore.toFixed(1)}/100 (${m.grade})`);
            });
        }
    });

    test('collect detailed metrics for specific examples', async ({ page }) => {
        // Collect detailed metrics for a few examples to analyze patterns
        const examples = await getAvailableExamples();
        const sampleExamples = examples.slice(0, 5); // First 5 examples
        
        const detailedResults: LayoutMetrics[] = [];
        
        for (const example of sampleExamples) {
            try {
                // Load example
                await page.click('button:has-text("Examples")');
                await page.waitForTimeout(500);
                
                const exampleButton = page.locator(`.example-item:has-text("${example.name}")`).first();
                await exampleButton.waitFor({ timeout: 5000 }).catch(() => {});
                await exampleButton.scrollIntoViewIfNeeded();
                await exampleButton.click();
                
                await page.waitForTimeout(2000);
                
                const metrics = await collectLayoutMetrics(page, example.name, example.category, {
                    includePerformance: true,
                    includeFullMetrics: true // Include full metrics for detailed analysis
                });
                
                detailedResults.push(metrics);
                
                console.log(`\n${example.name}:`);
                console.log(`  Score: ${metrics.weightedScore.toFixed(1)}/100`);
                console.log(`  Overlaps: ${metrics.overlappingNodes}`);
                console.log(`  Edge Crossings: ${metrics.edgeCrossings}`);
                console.log(`  Edges Over Nodes: ${metrics.edgesOverNodes}`);
                console.log(`  Engine: ${metrics.selectedEngine}-${metrics.selectedDirection}`);
                
            } catch (error) {
                console.error(`Error collecting metrics for ${example.name}:`, error);
            }
        }
        
        expect(detailedResults.length).toBeGreaterThan(0);
    });
});
