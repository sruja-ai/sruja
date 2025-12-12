// tests/optimization-loop.spec.ts
// Iterative optimization test that compares baseline with current metrics
import { test, expect } from '@playwright/test';
import { getAvailableExamples } from '@sruja/shared';
import {
    collectLayoutMetrics,
    type LayoutMetrics
} from './utils/metrics-collector';
import { compareMetrics, generateComparisonReport } from './utils/comparison';
import * as fs from 'fs';
import * as path from 'path';

test.describe('Layout Optimization Loop', () => {
    test.setTimeout(180000);
    test.beforeEach(async ({ page }) => {
        await page.goto('/?tab=diagram');
        await page.waitForSelector('.app', { timeout: 30000 });
        await page.waitForTimeout(1000);
    });

    test('compare current metrics with baseline', async ({ page }) => {
        const examples = await getAvailableExamples();
        const resultsDir = path.join(process.cwd(), 'tests', 'results');

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

        // Collect current metrics
        console.log(`\n=== Collecting Current Metrics for ${examples.length} Examples ===\n`);
        const currentMetrics = new Map<string, LayoutMetrics>();

        for (const example of examples) {
            try {
                console.log(`Collecting metrics for: ${example.name}`);

                await page.click('button:has-text("Examples")');
                await page.waitForTimeout(500);

                const exampleButton = page.locator(`.example-item:has-text("${example.name}")`).first();
                await exampleButton.waitFor({ timeout: 5000 }).catch(() => { });
                await exampleButton.scrollIntoViewIfNeeded();
                await exampleButton.click();

                await page.waitForTimeout(2000);

                // Ensure React Flow is ready before collecting metrics
                await page.waitForSelector('.react-flow', { timeout: 15000 }).catch(() => {
                    console.warn(`React Flow not ready for ${example.name}`);
                });

                const metrics = await collectLayoutMetrics(page, example.name, example.category, {
                    includePerformance: true,
                    includeFullMetrics: false
                });

                currentMetrics.set(example.name, metrics);
                console.log(`  Score: ${metrics.weightedScore.toFixed(1)}/100 (${metrics.grade})`);

            } catch (error) {
                console.error(`Failed to collect metrics for ${example.name}:`, error);
            }

            await page.waitForTimeout(500);
        }

        // Compare
        const comparison = compareMetrics(baselineMetrics, currentMetrics);

        // Save comparison
        const comparisonFile = path.join(resultsDir, `comparison-${Date.now()}.json`);
        fs.writeFileSync(comparisonFile, JSON.stringify(comparison, null, 2));

        // Generate report
        const report = generateComparisonReport(comparison);
        const reportFile = path.join(resultsDir, `comparison-report-${Date.now()}.md`);
        fs.writeFileSync(reportFile, report);

        console.log('\n=== Comparison Summary ===');
        console.log(`Total Examples: ${comparison.summary.totalExamples}`);
        console.log(`Improved: ${comparison.summary.improved} (${((comparison.summary.improved / comparison.summary.totalExamples) * 100).toFixed(1)}%)`);
        console.log(`Regressed: ${comparison.summary.regressed} (${((comparison.summary.regressed / comparison.summary.totalExamples) * 100).toFixed(1)}%)`);
        console.log(`Average Score Delta: ${comparison.summary.avgScoreDelta > 0 ? '+' : ''}${comparison.summary.avgScoreDelta.toFixed(2)} points`);
        console.log(`Average Score: ${comparison.summary.avgScoreBaseline.toFixed(1)} → ${comparison.summary.avgScoreCurrent.toFixed(1)}`);

        // Assertions
        expect(comparison.summary.totalExamples).toBeGreaterThan(0);

        // Check for improvements (at least some examples should improve or stay same)
        const improvementRate = comparison.summary.improved / comparison.summary.totalExamples;
        console.log(`\nImprovement Rate: ${(improvementRate * 100).toFixed(1)}%`);

        // Log top improvements
        const topImprovements = comparison.comparisons
            .filter(c => c.improvement)
            .sort((a, b) => b.scoreDelta - a.scoreDelta)
            .slice(0, 5);

        if (topImprovements.length > 0) {
            console.log('\nTop 5 Improvements:');
            topImprovements.forEach((c, i) => {
                console.log(`  ${i + 1}. ${c.exampleName}: ${c.baselineScore.toFixed(1)} → ${c.currentScore.toFixed(1)} (+${c.scoreDelta.toFixed(1)})`);
            });
        }

        // Log regressions
        const regressions = comparison.comparisons
            .filter(c => !c.improvement && c.scoreDelta < -1)
            .sort((a, b) => a.scoreDelta - b.scoreDelta)
            .slice(0, 5);

        if (regressions.length > 0) {
            console.log('\n⚠️  Top 5 Regressions:');
            regressions.forEach((c, i) => {
                console.log(`  ${i + 1}. ${c.exampleName}: ${c.baselineScore.toFixed(1)} → ${c.currentScore.toFixed(1)} (${c.scoreDelta.toFixed(1)})`);
            });
        }

        console.log(`\n✅ Comparison saved to: ${comparisonFile}`);
        console.log(`✅ Report saved to: ${reportFile}`);
    });
});
