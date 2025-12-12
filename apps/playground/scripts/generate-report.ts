// scripts/generate-report.ts
// Generate comprehensive optimization report
import * as fs from 'fs';
import * as path from 'path';
import { analyzeMetrics, type AnalysisResult } from './analyze-metrics';
import { analyzeRules, type RuleAnalysis } from './analyze-rules';

export interface ReportData {
    timestamp: string;
    metricsAnalysis: AnalysisResult;
    rulesAnalysis: RuleAnalysis;
    summary: {
        overallScore: number;
        passingRate: number;
        topIssues: string[];
        nextSteps: string[];
    };
}

/**
 * Generate markdown report
 */
function generateMarkdownReport(data: ReportData): string {
    const { metricsAnalysis, rulesAnalysis, summary } = data;
    
    let report = `# Layout Engine Optimization Report\n\n`;
    report += `Generated: ${data.timestamp}\n\n`;
    
    report += `## Executive Summary\n\n`;
    report += `- **Overall Average Score**: ${summary.overallScore.toFixed(1)}/100\n`;
    report += `- **Passing Rate**: ${summary.passingRate.toFixed(1)}% (≥70 score)\n`;
    report += `- **Total Examples Analyzed**: ${metricsAnalysis.totalExamples}\n\n`;
    
    if (summary.topIssues.length > 0) {
        report += `### Top Issues\n\n`;
        summary.topIssues.forEach((issue, i) => {
            report += `${i + 1}. ${issue}\n`;
        });
        report += `\n`;
    }
    
    report += `## Quality Metrics\n\n`;
    report += `### Score Distribution\n\n`;
    report += `- **Average Score**: ${metricsAnalysis.averageScore.toFixed(1)}/100\n`;
    report += `- **Passing Rate**: ${metricsAnalysis.passingRate.toFixed(1)}%\n\n`;
    
    report += `### Common Violations\n\n`;
    report += `- **Overlapping Nodes**: ${metricsAnalysis.commonViolations.overlappingNodes.count} examples\n`;
    report += `- **Edge Crossings**: ${metricsAnalysis.commonViolations.edgeCrossings.count} examples\n`;
    report += `- **Edges Over Nodes**: ${metricsAnalysis.commonViolations.edgesOverNodes.count} examples\n`;
    report += `- **Spacing Violations**: ${metricsAnalysis.commonViolations.spacingViolations.count} examples\n`;
    report += `- **Parent-Child Violations**: ${metricsAnalysis.commonViolations.parentChildViolations.count} examples\n\n`;
    
    report += `## Top 10 Problem Cases\n\n`;
    metricsAnalysis.topProblems.forEach((problem, i) => {
        report += `${i + 1}. **${problem.exampleName}** (${problem.score.toFixed(1)}/100, ${problem.grade})\n`;
        report += `   - Level: ${problem.level}, Engine: ${problem.engine}\n`;
        report += `   - Nodes: ${problem.nodeCount}, Edges: ${problem.edgeCount}\n`;
        report += `   - Issues: ${problem.issues.join(', ')}\n\n`;
    });
    
    report += `## Rule Effectiveness\n\n`;
    rulesAnalysis.rules.forEach(rule => {
        report += `### ${rule.ruleName}\n\n`;
        report += `- **Trigger Count**: ${rule.triggerCount}\n`;
        report += `- **Average Score**: ${rule.avgScore.toFixed(1)}/100\n`;
        report += `- **Success Rate**: ${rule.successRate.toFixed(1)}%\n\n`;
    });
    
    report += `## Level-Specific Analysis\n\n`;
    ['L0', 'L1', 'L2', 'L3'].forEach(level => {
        const levelData = metricsAnalysis.levelSpecificIssues[level as keyof typeof metricsAnalysis.levelSpecificIssues];
        if (levelData.count > 0) {
            report += `### ${level}\n\n`;
            report += `- **Average Score**: ${levelData.avgScore.toFixed(1)}/100\n`;
            report += `- **Count**: ${levelData.count}\n`;
            if (levelData.problems.length > 0) {
                report += `- **Problem Examples**: ${levelData.problems.join(', ')}\n`;
            }
            report += `\n`;
        }
    });
    
    report += `## Recommendations\n\n`;
    metricsAnalysis.recommendations.forEach((rec, i) => {
        report += `${i + 1}. ${rec}\n`;
    });
    
    if (rulesAnalysis.recommendations.length > 0) {
        report += `\n### Rule Optimization\n\n`;
        rulesAnalysis.recommendations.forEach((rec, i) => {
            report += `${i + 1}. ${rec}\n`;
        });
    }
    
    if (summary.nextSteps.length > 0) {
        report += `\n## Next Steps\n\n`;
        summary.nextSteps.forEach((step, i) => {
            report += `${i + 1}. ${step}\n`;
        });
    }
    
    return report;
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
    const metrics = data.metrics || [];

    // Run analyses
    const metricsAnalysis = analyzeMetrics(metrics);
    const rulesAnalysis = analyzeRules(metrics);

    // Generate summary
    const summary = {
        overallScore: metricsAnalysis.averageScore,
        passingRate: metricsAnalysis.passingRate,
        topIssues: [
            ...metricsAnalysis.recommendations.slice(0, 3),
            ...rulesAnalysis.recommendations.slice(0, 2)
        ],
        nextSteps: [
            'Review top problem cases and implement fixes',
            'Refine layout rules based on effectiveness analysis',
            'Optimize spacing parameters for problematic examples',
            'Improve edge routing for high crossing counts',
            'Enhance parent-child containment logic'
        ]
    };

    const reportData: ReportData = {
        timestamp: new Date().toISOString(),
        metricsAnalysis,
        rulesAnalysis,
        summary
    };

    // Save JSON data
    const jsonFile = path.join(resultsDir, `optimization-report-${Date.now()}.json`);
    fs.writeFileSync(jsonFile, JSON.stringify(reportData, null, 2));

    // Save markdown report
    const markdownReport = generateMarkdownReport(reportData);
    const mdFile = path.join(resultsDir, `optimization-report-${Date.now()}.md`);
    fs.writeFileSync(mdFile, markdownReport);

    console.log('\n✅ Report generated:');
    console.log(`   JSON: ${jsonFile}`);
    console.log(`   Markdown: ${mdFile}`);
}
