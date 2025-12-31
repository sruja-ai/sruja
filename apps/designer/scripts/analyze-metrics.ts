import { readFileSync, writeFileSync } from "fs";
import { join } from "path";

interface QualityMetric {
  exampleName: string;
  grade: string;
  weightedScore: number;
  overallScore: number;
  edgeCrossings: number;
  overlappingNodes: number;
  edgeLabelOverlaps: number;
  nodeCount: number;
  edgeCount: number;
  spacingViolations: number;
}

interface MetricsReport {
  timestamp: string;
  metrics: QualityMetric[];
  summary?: any;
}

function generateReport() {
  const jsonPath = join(process.cwd(), "tests/results/all-examples-metrics.json");
  const reportPath = join(process.cwd(), "DIAGRAM_QUALITY_REPORT.md"); // Save to cwd (app root)

  try {
    const rawData = readFileSync(jsonPath, "utf-8");
    const data: MetricsReport = JSON.parse(rawData);

    const metrics = data.metrics.filter((m) => m.grade !== "N/A" && m.grade !== "ERROR");
    const failures = data.metrics.filter((m) => m.grade === "N/A" || m.grade === "ERROR");

    const totalExamples = data.metrics.length;
    const passedExamples = metrics.length;
    const failedExamples = failures.length;

    const avgScore = metrics.reduce((sum, m) => sum + m.weightedScore, 0) / (passedExamples || 1);

    // Categorize grades
    const grades = { A: 0, B: 0, C: 0, D: 0, F: 0 };
    metrics.forEach((m) => {
      if (grades[m.grade as keyof typeof grades] !== undefined) {
        grades[m.grade as keyof typeof grades]++;
      } else {
        grades["F"]++;
      }
    });

    let md = `# Diagram Quality Report\n\n`;
    md += `**Timestamp:** ${data.timestamp}\n`;
    md += `**Total Examples:** ${totalExamples}\n`;
    md += `**Averages Score:** ${avgScore.toFixed(2)}\n\n`;

    md += `## Summary\n`;
    md += `- **Passed:** ${passedExamples}\n`;
    md += `- **Failed:** ${failedExamples}\n`;
    md += `- **Grade Distribution:** A:${grades.A}, B:${grades.B}, C:${grades.C}, D:${grades.D}, F:${grades.F}\n\n`;

    if (failures.length > 0) {
      md += `## Failed Examples\n`;
      failures.forEach((f) => {
        md += `- **${f.exampleName}**: ${f.grade}\n`;
      });
      md += `\n`;
    }

    md += `## Detailed Metrics\n`;
    md += `| Example | Grade | Score | Nodes | Edges | Crossings | Overlaps |\n`;
    md += `|---|---|---|---|---|---|---|\n`;

    metrics
      .sort((a, b) => a.weightedScore - b.weightedScore)
      .forEach((m) => {
        md += `| ${m.exampleName} | ${m.grade} | ${m.weightedScore.toFixed(1)} | ${m.nodeCount} | ${m.edgeCount} | ${m.edgeCrossings} | ${m.overlappingNodes} |\n`;
      });

    writeFileSync(reportPath, md);
    console.log(`Report generated at ${reportPath}`);
  } catch (e) {
    console.error("Failed to generate report:", e);
  }
}

generateReport();
