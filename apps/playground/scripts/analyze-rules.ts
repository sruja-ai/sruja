// scripts/analyze-rules.ts
// Analyze rule effectiveness from metrics data
import * as fs from 'fs';
import * as path from 'path';
import type { LayoutMetrics } from './analyze-metrics';

export interface RuleEffectiveness {
    ruleId: string;
    ruleName: string;
    triggerCount: number;
    avgScore: number;
    successRate: number; // Percentage of triggered cases with score >= 70
    examples: string[];
}

export interface RuleAnalysis {
    timestamp: string;
    rules: RuleEffectiveness[];
    recommendations: string[];
}

/**
 * Map layout config to rule ID (heuristic based on engine and level)
 */
function inferRuleId(metric: LayoutMetrics): string {
    // Determine complexity from node/edge count
    const isSimple = metric.nodeCount < 10 && metric.edgeCount < 15;
    const isComplex = metric.nodeCount >= 30 || metric.edgeCount >= 50;
    
    if (isSimple && !metric.hasHierarchy) {
        return 'simple-c4';
    }
    if (metric.hasHierarchy) {
        return 'hierarchical-sruja';
    }
    if ((metric.currentLevel === 'L0' || metric.currentLevel === 'L1')) {
        return 'level-l0-l1';
    }
    if (metric.currentLevel === 'L2') {
        return 'level-l2';
    }
    if (metric.currentLevel === 'L3') {
        return 'level-l3';
    }
    if (isComplex && metric.edgeCount > 30) {
        return 'complex-sruja';
    }
    return 'default-sruja';
}

/**
 * Analyze rule effectiveness from metrics
 */
export function analyzeRules(metrics: LayoutMetrics[]): RuleAnalysis {
    const ruleMap = new Map<string, LayoutMetrics[]>();

    // Group metrics by inferred rule
    metrics.forEach(m => {
        const ruleId = inferRuleId(m);
        if (!ruleMap.has(ruleId)) {
            ruleMap.set(ruleId, []);
        }
        ruleMap.get(ruleId)!.push(m);
    });

    // Calculate effectiveness for each rule
    const rules: RuleEffectiveness[] = [];
    const ruleNames: Record<string, string> = {
        'simple-c4': 'Simple C4 Layout',
        'hierarchical-sruja': 'Hierarchical Sruja Layout',
        'level-l0-l1': 'L0/L1 C4 Layout',
        'level-l2': 'L2 Container Layout',
        'level-l3': 'L3 Component Layout',
        'complex-sruja': 'Complex Sruja Layout',
        'default-sruja': 'Default Sruja Layout'
    };

    ruleMap.forEach((ruleMetrics, ruleId) => {
        const scores = ruleMetrics.map(m => m.weightedScore);
        const avgScore = scores.reduce((a, b) => a + b, 0) / scores.length;
        const passingCount = scores.filter(s => s >= 70).length;
        const successRate = (passingCount / scores.length) * 100;

        rules.push({
            ruleId,
            ruleName: ruleNames[ruleId] || ruleId,
            triggerCount: ruleMetrics.length,
            avgScore,
            successRate,
            examples: ruleMetrics.map(m => m.exampleName).slice(0, 10)
        });
    });

    // Sort by trigger count (most used first)
    rules.sort((a, b) => b.triggerCount - a.triggerCount);

    // Generate recommendations
    const recommendations = generateRuleRecommendations(rules);

    return {
        timestamp: new Date().toISOString(),
        rules,
        recommendations
    };
}

function generateRuleRecommendations(rules: RuleEffectiveness[]): string[] {
    const recommendations: string[] = [];

    // Find rules with low success rates
    const lowPerforming = rules.filter(r => r.successRate < 70 && r.triggerCount > 5);
    if (lowPerforming.length > 0) {
        lowPerforming.forEach(r => {
            recommendations.push(`Rule "${r.ruleName}" has low success rate: ${r.successRate.toFixed(1)}% (${r.triggerCount} triggers). Consider refining conditions or adjusting layout parameters.`);
        });
    }

    // Find rules with high success rates that could be prioritized
    const highPerforming = rules.filter(r => r.successRate >= 85 && r.avgScore >= 75);
    if (highPerforming.length > 0) {
        recommendations.push(`High-performing rules (${highPerforming.length}) could be prioritized: ${highPerforming.map(r => r.ruleName).join(', ')}`);
    }

    // Check for rules that rarely trigger
    const rarelyUsed = rules.filter(r => r.triggerCount < 3);
    if (rarelyUsed.length > 0) {
        recommendations.push(`Rarely used rules (${rarelyUsed.length}): Consider removing or merging: ${rarelyUsed.map(r => r.ruleName).join(', ')}`);
    }

    return recommendations;
}

/**
 * Main execution
 */
if (require.main === module) {
    const resultsDir = path.join(__dirname, '../tests/results');
    const baselineFile = path.join(resultsDir, 'baseline-metrics.json');

    if (!fs.existsSync(baselineFile)) {
        console.error(`Baseline metrics file not found: ${baselineFile}`);
        console.error('Run "npm run test:baseline" first to collect metrics.');
        process.exit(1);
    }

    const content = fs.readFileSync(baselineFile, 'utf-8');
    const data = JSON.parse(content);
    const metrics: LayoutMetrics[] = data.metrics || [];

    const analysis = analyzeRules(metrics);

    // Save analysis
    const analysisFile = path.join(resultsDir, `rule-analysis-${Date.now()}.json`);
    fs.writeFileSync(analysisFile, JSON.stringify(analysis, null, 2));

    // Print summary
    console.log('\n=== Rule Effectiveness Analysis ===\n');
    analysis.rules.forEach(r => {
        console.log(`${r.ruleName} (${r.ruleId}):`);
        console.log(`  Triggers: ${r.triggerCount}`);
        console.log(`  Avg Score: ${r.avgScore.toFixed(1)}/100`);
        console.log(`  Success Rate: ${r.successRate.toFixed(1)}%`);
        console.log('');
    });

    if (analysis.recommendations.length > 0) {
        console.log('Recommendations:');
        analysis.recommendations.forEach((r, i) => {
            console.log(`  ${i + 1}. ${r}`);
        });
    }

    console.log(`\n✅ Analysis saved to: ${analysisFile}`);
}
