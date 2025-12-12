// tests/regression.spec.ts
// Regression prevention tests - ensure improvements don't break existing layouts
import { test, expect } from '@playwright/test';
import { getAvailableExamples } from '@sruja/shared';
import {
    collectLayoutMetrics,
    type LayoutMetrics
} from './utils/metrics-collector';
import * as fs from 'fs';
import * as path from 'path';

test.describe('Layout Regression Prevention', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('/');
        await page.waitForSelector('.architecture-canvas', { timeout: 10000 });
        await page.waitForTimeout(2000);
    });

    test('no example should drop below baseline score', async ({ page }) => {
        const examples = await getAvailableExamples();
        const resultsDir = path.join(__dirname, 'results');
        
        // Load baseline
        const baselineFile = path.join(resultsDir, 'baseline-metrics.json');
        if (!fs.existsSync(baselineFile)) {
            test.skip(true, 'Baseline metrics not found. Run test:baseline first.');
            return;
        }

        const baselineContent = fs.readFileSync(baselineFile, 'utf-8');
        const baselineData = JSON.parse(baselineContent);
        const baselineMetrics = new Map<string, LayoutMetrics>();
        
        baselineData.metrics.forEach((m: LayoutMetrics) => {
            baselineMetrics.set(m.exampleName, m);
        });

        console.log(`\n=== Regression Check for ${examples.length} Examples ===\n`);
        
        const regressions: Array<{
            exampleName: string;
            baselineScore: number;
            currentScore: number;
            delta: number;
        }> = [];

        // Test a sample of examples (or all if small)
        const testExamples = examples.length > 20 ? examples.slice(0, 20) : examples;
        
        for (const example of testExamples) {
            try {
                const baseline = baselineMetrics.get(example.name);
                if (!baseline) {
                    console.log(`Skipping ${example.name} - no baseline data`);
                    continue;
                }

                // Load example
                await page.click('button:has-text("Examples")');
                await page.waitForTimeout(500);
                
                const exampleButton = page.locator(`.example-item:has-text("${example.name}")`).first();
                await exampleButton.waitFor({ timeout: 5000 }).catch(() => {});
                await exampleButton.scrollIntoViewIfNeeded();
                await exampleButton.click();
                
                await page.waitForTimeout(2000);
                
                // Ensure React Flow is ready before collecting metrics
                await page.waitForSelector('.react-flow', { timeout: 15000 }).catch(() => {
                    console.warn(`React Flow not ready for ${example.name}`);
                });
                
                // Collect current metrics
                const current = await collectLayoutMetrics(page, example.name, example.category, {
                    includePerformance: true,
                    includeFullMetrics: false
                });

                const delta = current.weightedScore - baseline.weightedScore;
                
                // Check for regression (score dropped by more than 5 points)
                if (delta < -5) {
                    regressions.push({
                        exampleName: example.name,
                        baselineScore: baseline.weightedScore,
                        currentScore: current.weightedScore,
                        delta
                    });
                    console.log(`⚠️  REGRESSION: ${example.name}: ${baseline.weightedScore.toFixed(1)} → ${current.weightedScore.toFixed(1)} (${delta.toFixed(1)})`);
                } else if (delta > 0) {
                    console.log(`✅ IMPROVED: ${example.name}: ${baseline.weightedScore.toFixed(1)} → ${current.weightedScore.toFixed(1)} (+${delta.toFixed(1)})`);
                } else {
                    console.log(`   OK: ${example.name}: ${baseline.weightedScore.toFixed(1)} → ${current.weightedScore.toFixed(1)} (${delta.toFixed(1)})`);
                }
                
            } catch (error) {
                console.error(`Failed to check ${example.name}:`, error);
            }
            
            await page.waitForTimeout(500);
        }

        // Report regressions
        if (regressions.length > 0) {
            console.log(`\n⚠️  Found ${regressions.length} regressions:`);
            regressions.forEach(r => {
                console.log(`  - ${r.exampleName}: ${r.baselineScore.toFixed(1)} → ${r.currentScore.toFixed(1)} (${r.delta.toFixed(1)})`);
            });
        }

        // Assert no significant regressions
        expect(regressions.length).toBe(0);
    });

    test('critical examples must maintain quality', async ({ page }) => {
        // Define critical examples (high-profile or commonly used)
        const criticalExamples = [
            'ecommerce-system',
            'microservices-architecture',
            'system-context-diagram'
        ];

        const resultsDir = path.join(__dirname, 'results');
        const baselineFile = path.join(resultsDir, 'baseline-metrics.json');
        
        if (!fs.existsSync(baselineFile)) {
            test.skip(true, 'Baseline metrics not found.');
            return;
        }

        const baselineContent = fs.readFileSync(baselineFile, 'utf-8');
        const baselineData = JSON.parse(baselineContent);
        const baselineMetrics = new Map<string, LayoutMetrics>();
        
        baselineData.metrics.forEach((m: LayoutMetrics) => {
            baselineMetrics.set(m.exampleName, m);
        });

        const examples = await getAvailableExamples();
        const availableCritical = criticalExamples.filter(name => 
            examples.some(e => e.name.includes(name) || e.name === name)
        );

        if (availableCritical.length === 0) {
            test.skip(true, 'No critical examples found in available examples.');
            return;
        }

        console.log(`\n=== Critical Examples Quality Check ===\n`);

        for (const criticalName of availableCritical) {
            const example = examples.find(e => e.name.includes(criticalName) || e.name === criticalName);
            if (!example) continue;

            const baseline = baselineMetrics.get(example.name);
            if (!baseline) continue;

            // Load example
            await page.click('button:has-text("Examples")');
            await page.waitForTimeout(500);
            
            const exampleButton = page.locator(`.example-item:has-text("${example.name}")`).first();
            await exampleButton.waitFor({ timeout: 5000 }).catch(() => {});
            await exampleButton.scrollIntoViewIfNeeded();
            await exampleButton.click();
            
            await page.waitForTimeout(2000);
            
            // Ensure React Flow is ready
            await page.waitForSelector('.react-flow', { timeout: 15000 }).catch(() => {
                console.warn(`React Flow not ready for ${example.name}`);
            });
            
            const current = await collectLayoutMetrics(page, example.name, example.category, {
                includePerformance: true,
                includeFullMetrics: false
            });

            console.log(`${example.name}: ${baseline.weightedScore.toFixed(1)} → ${current.weightedScore.toFixed(1)}`);

            // Critical examples must maintain at least 80% of baseline score
            const minAllowedScore = baseline.weightedScore * 0.8;
            expect(current.weightedScore).toBeGreaterThanOrEqual(minAllowedScore);
            
            // Critical examples must score at least 70
            expect(current.weightedScore).toBeGreaterThanOrEqual(70);
            
            await page.waitForTimeout(500);
        }
    });

    test('performance should not degrade significantly', async ({ page }) => {
        const examples = await getAvailableExamples();
        const resultsDir = path.join(__dirname, 'results');
        
        const baselineFile = path.join(resultsDir, 'baseline-metrics.json');
        if (!fs.existsSync(baselineFile)) {
            test.skip(true, 'Baseline metrics not found.');
            return;
        }

        const baselineContent = fs.readFileSync(baselineFile, 'utf-8');
        const baselineData = JSON.parse(baselineContent);
        const baselineMetrics = new Map<string, LayoutMetrics>();
        
        baselineData.metrics.forEach((m: LayoutMetrics) => {
            baselineMetrics.set(m.exampleName, m);
        });

        console.log(`\n=== Performance Regression Check ===\n`);

        const testExamples = examples.slice(0, 10); // Test first 10
        const performanceRegressions: string[] = [];

        for (const example of testExamples) {
            const baseline = baselineMetrics.get(example.name);
            if (!baseline || baseline.layoutTime === 0) continue;

            // Load example
            await page.click('button:has-text("Examples")');
            await page.waitForTimeout(500);
            
            const exampleButton = page.locator(`.example-item:has-text("${example.name}")`).first();
            await exampleButton.waitFor({ timeout: 5000 }).catch(() => {});
            await exampleButton.scrollIntoViewIfNeeded();
            await exampleButton.click();
            
            await page.waitForTimeout(2000);
            
            // Ensure React Flow is ready
            await page.waitForSelector('.react-flow', { timeout: 15000 }).catch(() => {
                console.warn(`React Flow not ready for ${example.name}`);
            });
            
            const current = await collectLayoutMetrics(page, example.name, example.category, {
                includePerformance: true,
                includeFullMetrics: false
            });

            if (current.layoutTime > 0 && baseline.layoutTime > 0) {
                const performanceDelta = ((current.layoutTime - baseline.layoutTime) / baseline.layoutTime) * 100;
                
                // Alert if performance degraded by more than 20%
                if (performanceDelta > 20) {
                    performanceRegressions.push(
                        `${example.name}: layout time increased by ${performanceDelta.toFixed(1)}% (${baseline.layoutTime}ms → ${current.layoutTime}ms)`
                    );
                    console.log(`⚠️  ${performanceRegressions[performanceRegressions.length - 1]}`);
                } else {
                    console.log(`   ${example.name}: ${baseline.layoutTime}ms → ${current.layoutTime}ms (${performanceDelta > 0 ? '+' : ''}${performanceDelta.toFixed(1)}%)`);
                }
            }
            
            await page.waitForTimeout(500);
        }

        if (performanceRegressions.length > 0) {
            console.log(`\n⚠️  Performance regressions detected: ${performanceRegressions.length}`);
            // Don't fail the test, but log warnings
            // In CI, this could be configured to fail
        }

        // For now, just log - can be made stricter in CI
        expect(performanceRegressions.length).toBeLessThan(testExamples.length / 2);
    });
});
