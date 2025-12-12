// tests/layout-comparison.spec.ts
// Compare different layout engines/configurations
import { test, expect } from '@playwright/test';
import { getAvailableExamples } from '@sruja/shared';
import {
    collectLayoutMetrics,
    type LayoutMetrics
} from './utils/metrics-collector';
import * as fs from 'fs';
import * as path from 'path';

test.describe('Layout Engine Comparison', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('/?tab=diagram');
        await page.waitForSelector('.app', { timeout: 30000 });
        await page.waitForTimeout(1000);
    });

    test('compare layout engines across examples', async ({ page }) => {
        const examples = await getAvailableExamples();
        const sampleExamples = examples.slice(0, 10); // Test with first 10 examples
        
        const results: Array<{
            exampleName: string;
            srujaMetrics?: LayoutMetrics;
            c4levelMetrics?: LayoutMetrics;
            comparison: {
                scoreDelta: number;
                betterEngine: 'sruja' | 'c4level' | 'tie';
            };
        }> = [];

        for (const example of sampleExamples) {
            try {
                console.log(`\nComparing engines for: ${example.name}`);
                
                // Load example
                await page.click('button:has-text("Examples")');
                await page.waitForTimeout(500);
                
                const exampleButton = page.locator(`.example-item:has-text("${example.name}")`).first();
                await exampleButton.waitFor({ timeout: 5000 }).catch(() => {});
                await exampleButton.scrollIntoViewIfNeeded();
                await exampleButton.click();
                
                await page.waitForTimeout(2000);
                
                // Wait for layout selector to be available
                const layoutSelector = page.locator('.layout-selector select').first();
                const selectorExists = await layoutSelector.isVisible().catch(() => false);
                
                if (!selectorExists) {
                    console.warn(`Layout selector not found for ${example.name}, skipping engine comparison`);
                    continue;
                }
                
                await layoutSelector.waitFor({ timeout: 10000, state: 'visible' });
                
                let c4levelMetrics: LayoutMetrics | undefined;
                let srujaMetrics: LayoutMetrics | undefined;
                
                // Collect metrics for C4Level
                try {
                    await layoutSelector.selectOption('c4level');
                    await page.waitForTimeout(2000); // Wait for layout to update
                    await page.waitForSelector('.react-flow', { timeout: 15000 });
                    c4levelMetrics = await collectLayoutMetrics(page, example.name, example.category, {
                        includePerformance: true,
                        includeFullMetrics: false
                    });
                } catch (error) {
                    console.error(`Failed to collect C4Level metrics for ${example.name}:`, error);
                    continue;
                }

                // Collect metrics for Sruja
                try {
                    await layoutSelector.selectOption('sruja');
                    await page.waitForTimeout(2000); // Wait for layout to update
                    await page.waitForSelector('.react-flow', { timeout: 15000 });
                    srujaMetrics = await collectLayoutMetrics(page, example.name, example.category, {
                        includePerformance: true,
                        includeFullMetrics: false
                    });
                } catch (error) {
                    console.error(`Failed to collect Sruja metrics for ${example.name}:`, error);
                    continue;
                }

                if (!c4levelMetrics || !srujaMetrics) {
                    console.warn(`Missing metrics for ${example.name}, skipping`);
                    continue;
                }

                const scoreDelta = srujaMetrics.weightedScore - c4levelMetrics.weightedScore;
                const betterEngine: 'sruja' | 'c4level' | 'tie' = Math.abs(scoreDelta) < 0.5
                    ? 'tie'
                    : scoreDelta > 0 ? 'sruja' : 'c4level';

                results.push({
                    exampleName: example.name,
                    srujaMetrics,
                    c4levelMetrics,
                    comparison: { scoreDelta, betterEngine }
                });

                console.log(`  Sruja: ${srujaMetrics.weightedScore.toFixed(1)} / C4Level: ${c4levelMetrics.weightedScore.toFixed(1)} (Δ ${scoreDelta.toFixed(1)})`);
                
            } catch (error) {
                console.error(`Failed to compare engines for ${example.name}:`, error);
            }
            
            await page.waitForTimeout(500);
        }

        // Analyze results
        const srujaCount = results.filter(r => r.srujaMetrics).length;
        const c4levelCount = results.filter(r => r.c4levelMetrics).length;
        const srujaAvgScore = results
            .filter(r => r.srujaMetrics)
            .reduce((sum, r) => sum + (r.srujaMetrics?.weightedScore || 0), 0) / srujaCount || 0;
        const c4levelAvgScore = results
            .filter(r => r.c4levelMetrics)
            .reduce((sum, r) => sum + (r.c4levelMetrics?.weightedScore || 0), 0) / c4levelCount || 0;

        console.log('\n=== Engine Comparison Summary ===');
        console.log(`Sruja Engine: ${srujaCount} examples, Avg Score: ${srujaAvgScore.toFixed(1)}/100`);
        console.log(`C4Level Engine: ${c4levelCount} examples, Avg Score: ${c4levelAvgScore.toFixed(1)}/100`);

        // Save results
        const resultsDir = path.join(process.cwd(), 'tests', 'results');
        if (!fs.existsSync(resultsDir)) {
            fs.mkdirSync(resultsDir, { recursive: true });
        }

        const comparisonFile = path.join(resultsDir, `engine-comparison-${Date.now()}.json`);
        fs.writeFileSync(comparisonFile, JSON.stringify({
            timestamp: new Date().toISOString(),
            summary: {
                srujaCount,
                c4levelCount,
                srujaAvgScore,
                c4levelAvgScore
            },
            results
        }, null, 2));

        console.log(`\n✅ Comparison saved to: ${comparisonFile}`);

        expect(results.length).toBeGreaterThan(0);
    });
});
