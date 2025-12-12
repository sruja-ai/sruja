// Comparison utilities for layout optimization
import type { LayoutMetrics } from './metrics-collector';

export interface ComparisonResult {
    exampleName: string;
    baselineScore: number;
    currentScore: number;
    scoreDelta: number;
    improvement: boolean;
    regressions: string[];
    improvements: string[];
    details: {
        overlappingNodes: { baseline: number; current: number; delta: number };
        edgeCrossings: { baseline: number; current: number; delta: number };
        edgesOverNodes: { baseline: number; current: number; delta: number };
        spacingViolations: { baseline: number; current: number; delta: number };
    };
}

/**
 * Compare baseline metrics with current metrics
 */
export function compareMetrics(
    baseline: Map<string, LayoutMetrics>,
    current: Map<string, LayoutMetrics>
): {
    comparisons: ComparisonResult[];
    summary: {
        totalExamples: number;
        improved: number;
        regressed: number;
        unchanged: number;
        avgScoreDelta: number;
        avgScoreBaseline: number;
        avgScoreCurrent: number;
    };
} {
    const comparisons: ComparisonResult[] = [];
    
    for (const [name, baselineMetrics] of baseline.entries()) {
        const currentMetrics = current.get(name);
        
        if (!currentMetrics) {
            console.warn(`No current metrics found for ${name}`);
            continue;
        }
        
        const scoreDelta = currentMetrics.weightedScore - baselineMetrics.weightedScore;
        const improvement = scoreDelta > 0;
        
        const regressions: string[] = [];
        const improvements: string[] = [];
        
        // Check specific metrics
        if (currentMetrics.overlappingNodes > baselineMetrics.overlappingNodes) {
            regressions.push(`Overlapping nodes increased by ${currentMetrics.overlappingNodes - baselineMetrics.overlappingNodes}`);
        } else if (currentMetrics.overlappingNodes < baselineMetrics.overlappingNodes) {
            improvements.push(`Overlapping nodes decreased by ${baselineMetrics.overlappingNodes - currentMetrics.overlappingNodes}`);
        }
        
        if (currentMetrics.edgeCrossings > baselineMetrics.edgeCrossings) {
            regressions.push(`Edge crossings increased by ${currentMetrics.edgeCrossings - baselineMetrics.edgeCrossings}`);
        } else if (currentMetrics.edgeCrossings < baselineMetrics.edgeCrossings) {
            improvements.push(`Edge crossings decreased by ${baselineMetrics.edgeCrossings - currentMetrics.edgeCrossings}`);
        }
        
        if (currentMetrics.edgesOverNodes > baselineMetrics.edgesOverNodes) {
            regressions.push(`Edges over nodes increased by ${currentMetrics.edgesOverNodes - baselineMetrics.edgesOverNodes}`);
        } else if (currentMetrics.edgesOverNodes < baselineMetrics.edgesOverNodes) {
            improvements.push(`Edges over nodes decreased by ${baselineMetrics.edgesOverNodes - currentMetrics.edgesOverNodes}`);
        }
        
        if (currentMetrics.spacingViolations > baselineMetrics.spacingViolations) {
            regressions.push(`Spacing violations increased by ${currentMetrics.spacingViolations - baselineMetrics.spacingViolations}`);
        } else if (currentMetrics.spacingViolations < baselineMetrics.spacingViolations) {
            improvements.push(`Spacing violations decreased by ${baselineMetrics.spacingViolations - currentMetrics.spacingViolations}`);
        }
        
        comparisons.push({
            exampleName: name,
            baselineScore: baselineMetrics.weightedScore,
            currentScore: currentMetrics.weightedScore,
            scoreDelta,
            improvement,
            regressions,
            improvements,
            details: {
                overlappingNodes: {
                    baseline: baselineMetrics.overlappingNodes,
                    current: currentMetrics.overlappingNodes,
                    delta: currentMetrics.overlappingNodes - baselineMetrics.overlappingNodes
                },
                edgeCrossings: {
                    baseline: baselineMetrics.edgeCrossings,
                    current: currentMetrics.edgeCrossings,
                    delta: currentMetrics.edgeCrossings - baselineMetrics.edgeCrossings
                },
                edgesOverNodes: {
                    baseline: baselineMetrics.edgesOverNodes,
                    current: currentMetrics.edgesOverNodes,
                    delta: currentMetrics.edgesOverNodes - baselineMetrics.edgesOverNodes
                },
                spacingViolations: {
                    baseline: baselineMetrics.spacingViolations,
                    current: currentMetrics.spacingViolations,
                    delta: currentMetrics.spacingViolations - baselineMetrics.spacingViolations
                }
            }
        });
    }
    
    // Calculate summary
    const improved = comparisons.filter(c => c.improvement).length;
    const regressed = comparisons.filter(c => !c.improvement && c.scoreDelta < -1).length;
    const unchanged = comparisons.length - improved - regressed;
    
    const scores = Array.from(baseline.values()).map(m => m.weightedScore);
    const currentScores = Array.from(current.values()).map(m => m.weightedScore);
    
    const avgScoreBaseline = scores.reduce((a, b) => a + b, 0) / scores.length;
    const avgScoreCurrent = currentScores.reduce((a, b) => a + b, 0) / currentScores.length;
    const avgScoreDelta = avgScoreCurrent - avgScoreBaseline;
    
    return {
        comparisons,
        summary: {
            totalExamples: comparisons.length,
            improved,
            regressed,
            unchanged,
            avgScoreDelta,
            avgScoreBaseline,
            avgScoreCurrent
        }
    };
}

/**
 * Generate comparison report
 */
export function generateComparisonReport(comparison: ReturnType<typeof compareMetrics>): string {
    const { comparisons, summary } = comparison;
    
    let report = `# Layout Optimization Comparison Report\n\n`;
    report += `Generated: ${new Date().toISOString()}\n\n`;
    
    report += `## Summary\n\n`;
    report += `- **Total Examples**: ${summary.totalExamples}\n`;
    report += `- **Improved**: ${summary.improved} (${((summary.improved / summary.totalExamples) * 100).toFixed(1)}%)\n`;
    report += `- **Regressed**: ${summary.regressed} (${((summary.regressed / summary.totalExamples) * 100).toFixed(1)}%)\n`;
    report += `- **Unchanged**: ${summary.unchanged} (${((summary.unchanged / summary.totalExamples) * 100).toFixed(1)}%)\n`;
    report += `- **Average Score Delta**: ${summary.avgScoreDelta > 0 ? '+' : ''}${summary.avgScoreDelta.toFixed(2)} points\n`;
    report += `- **Average Score**: ${summary.avgScoreBaseline.toFixed(1)} → ${summary.avgScoreCurrent.toFixed(1)}\n\n`;
    
    // Top improvements
    const topImprovements = comparisons
        .filter(c => c.improvement)
        .sort((a, b) => b.scoreDelta - a.scoreDelta)
        .slice(0, 10);
    
    if (topImprovements.length > 0) {
        report += `## Top 10 Improvements\n\n`;
        topImprovements.forEach((c, i) => {
            report += `${i + 1}. **${c.exampleName}**: ${c.baselineScore.toFixed(1)} → ${c.currentScore.toFixed(1)} (+${c.scoreDelta.toFixed(1)})\n`;
            if (c.improvements.length > 0) {
                report += `   - ${c.improvements.join(', ')}\n`;
            }
        });
        report += `\n`;
    }
    
    // Top regressions
    const topRegressions = comparisons
        .filter(c => !c.improvement && c.scoreDelta < -1)
        .sort((a, b) => a.scoreDelta - b.scoreDelta)
        .slice(0, 10);
    
    if (topRegressions.length > 0) {
        report += `## Top 10 Regressions\n\n`;
        topRegressions.forEach((c, i) => {
            report += `${i + 1}. **${c.exampleName}**: ${c.baselineScore.toFixed(1)} → ${c.currentScore.toFixed(1)} (${c.scoreDelta.toFixed(1)})\n`;
            if (c.regressions.length > 0) {
                report += `   - ${c.regressions.join(', ')}\n`;
            }
        });
        report += `\n`;
    }
    
    return report;
}
