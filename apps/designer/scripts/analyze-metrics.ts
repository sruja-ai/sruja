// scripts/analyze-metrics.ts
// Analyze collected layout metrics to identify problem patterns
import * as fs from "fs";
import * as path from "path";

export interface LayoutMetrics {
  exampleName: string;
  category: string;
  weightedScore: number;
  overallScore: number;
  grade: "A" | "B" | "C" | "D" | "F";
  overlappingNodes: number;
  edgeCrossings: number;
  edgesOverNodes: number;
  edgeBends: number;
  spacingViolations: number;
  parentChildViolations: number;
  nodeCount: number;
  edgeCount: number;
  hasHierarchy: boolean;
  currentLevel: string;
  selectedEngine: "sruja" | "c4level";
  selectedDirection: string;
  viewportUtilization: number;
  aspectRatio: number;
}

export interface AnalysisResult {
  timestamp: string;
  totalExamples: number;
  averageScore: number;
  passingRate: number;
  topProblems: ProblemCase[];
  commonViolations: ViolationSummary;
  engineMismatches: EngineMismatch[];
  levelSpecificIssues: LevelIssues;
  sizeRelatedIssues: SizeIssues;
  hierarchyIssues: HierarchyIssues;
  recommendations: string[];
}

export interface ProblemCase {
  exampleName: string;
  score: number;
  grade: string;
  issues: string[];
  nodeCount: number;
  edgeCount: number;
  level: string;
  engine: string;
}

export interface ViolationSummary {
  overlappingNodes: { count: number; examples: string[] };
  edgeCrossings: { count: number; examples: string[] };
  edgesOverNodes: { count: number; examples: string[] };
  spacingViolations: { count: number; examples: string[] };
  parentChildViolations: { count: number; examples: string[] };
}

export interface EngineMismatch {
  exampleName: string;
  currentLevel: string;
  selectedEngine: string;
  suggestedEngine: string;
  reason: string;
}

export interface LevelIssues {
  L0: { avgScore: number; count: number; problems: string[] };
  L1: { avgScore: number; count: number; problems: string[] };
  L2: { avgScore: number; count: number; problems: string[] };
  L3: { avgScore: number; count: number; problems: string[] };
}

export interface SizeIssues {
  small: { avgScore: number; count: number; examples: string[] };
  medium: { avgScore: number; count: number; examples: string[] };
  large: { avgScore: number; count: number; examples: string[] };
}

export interface HierarchyIssues {
  withHierarchy: { avgScore: number; count: number; problems: string[] };
  withoutHierarchy: { avgScore: number; count: number; problems: string[] };
}

/**
 * Load metrics from JSON file
 */
function loadMetrics(filepath: string): LayoutMetrics[] {
  const content = fs.readFileSync(filepath, "utf-8");
  const data = JSON.parse(content);
  return data.metrics || [];
}

/**
 * Analyze metrics and identify patterns
 */
export function analyzeMetrics(metrics: LayoutMetrics[]): AnalysisResult {
  const totalExamples = metrics.length;
  const scores = metrics.map((m) => m.weightedScore);
  const averageScore = scores.reduce((a, b) => a + b, 0) / scores.length;
  const passingCount = scores.filter((s) => s >= 70).length;
  const passingRate = (passingCount / totalExamples) * 100;

  // Identify top problem cases
  const topProblems = identifyTopProblems(metrics);

  // Analyze common violations
  const commonViolations = analyzeViolations(metrics);

  // Detect engine mismatches
  const engineMismatches = detectEngineMismatches(metrics);

  // Level-specific issues
  const levelSpecificIssues = analyzeLevelIssues(metrics);

  // Size-related issues
  const sizeRelatedIssues = analyzeSizeIssues(metrics);

  // Hierarchy issues
  const hierarchyIssues = analyzeHierarchyIssues(metrics);

  // Generate recommendations
  const recommendations = generateRecommendations(
    topProblems,
    commonViolations,
    engineMismatches,
    levelSpecificIssues,
    sizeRelatedIssues,
    hierarchyIssues
  );

  return {
    timestamp: new Date().toISOString(),
    totalExamples,
    averageScore,
    passingRate,
    topProblems,
    commonViolations,
    engineMismatches,
    levelSpecificIssues,
    sizeRelatedIssues,
    hierarchyIssues,
    recommendations,
  };
}

function identifyTopProblems(metrics: LayoutMetrics[]): ProblemCase[] {
  return metrics
    .filter((m) => m.weightedScore < 70)
    .map((m) => ({
      exampleName: m.exampleName,
      score: m.weightedScore,
      grade: m.grade,
      issues: identifyIssues(m),
      nodeCount: m.nodeCount,
      edgeCount: m.edgeCount,
      level: m.currentLevel,
      engine: m.selectedEngine,
    }))
    .sort((a, b) => a.score - b.score)
    .slice(0, 10);
}

function identifyIssues(metric: LayoutMetrics): string[] {
  const issues: string[] = [];
  if (metric.overlappingNodes > 0) issues.push(`${metric.overlappingNodes} overlapping nodes`);
  if (metric.edgeCrossings > 10) issues.push(`${metric.edgeCrossings} edge crossings`);
  if (metric.edgesOverNodes > 5) issues.push(`${metric.edgesOverNodes} edges over nodes`);
  if (metric.spacingViolations > 5) issues.push(`${metric.spacingViolations} spacing violations`);
  if (metric.parentChildViolations > 0)
    issues.push(`${metric.parentChildViolations} parent-child violations`);
  if (metric.aspectRatio < 0.3 || metric.aspectRatio > 3.0)
    issues.push(`extreme aspect ratio: ${metric.aspectRatio.toFixed(2)}`);
  if (metric.viewportUtilization < 0.3)
    issues.push(`low viewport utilization: ${(metric.viewportUtilization * 100).toFixed(1)}%`);
  return issues;
}

function analyzeViolations(metrics: LayoutMetrics[]): ViolationSummary {
  const overlapping = metrics.filter((m) => m.overlappingNodes > 0);
  const crossings = metrics.filter((m) => m.edgeCrossings > 10);
  const edgesOver = metrics.filter((m) => m.edgesOverNodes > 5);
  const spacing = metrics.filter((m) => m.spacingViolations > 5);
  const parentChild = metrics.filter((m) => m.parentChildViolations > 0);

  return {
    overlappingNodes: {
      count: overlapping.length,
      examples: overlapping.map((m) => m.exampleName).slice(0, 10),
    },
    edgeCrossings: {
      count: crossings.length,
      examples: crossings.map((m) => m.exampleName).slice(0, 10),
    },
    edgesOverNodes: {
      count: edgesOver.length,
      examples: edgesOver.map((m) => m.exampleName).slice(0, 10),
    },
    spacingViolations: {
      count: spacing.length,
      examples: spacing.map((m) => m.exampleName).slice(0, 10),
    },
    parentChildViolations: {
      count: parentChild.length,
      examples: parentChild.map((m) => m.exampleName).slice(0, 10),
    },
  };
}

function detectEngineMismatches(metrics: LayoutMetrics[]): EngineMismatch[] {
  const mismatches: EngineMismatch[] = [];

  metrics.forEach((m) => {
    // L0/L1 should prefer c4level unless expanded
    if ((m.currentLevel === "L0" || m.currentLevel === "L1") && m.selectedEngine === "sruja") {
      mismatches.push({
        exampleName: m.exampleName,
        currentLevel: m.currentLevel,
        selectedEngine: m.selectedEngine,
        suggestedEngine: "c4level",
        reason: "L0/L1 levels typically work better with c4level engine",
      });
    }

    // Complex diagrams with hierarchy should use sruja
    if (m.hasHierarchy && m.nodeCount > 20 && m.selectedEngine === "c4level") {
      mismatches.push({
        exampleName: m.exampleName,
        currentLevel: m.currentLevel,
        selectedEngine: m.selectedEngine,
        suggestedEngine: "sruja",
        reason: "Complex hierarchical diagrams should use sruja engine",
      });
    }
  });

  return mismatches.slice(0, 10);
}

function analyzeLevelIssues(metrics: LayoutMetrics[]): LevelIssues {
  const byLevel = {
    L0: metrics.filter((m) => m.currentLevel === "L0"),
    L1: metrics.filter((m) => m.currentLevel === "L1"),
    L2: metrics.filter((m) => m.currentLevel === "L2"),
    L3: metrics.filter((m) => m.currentLevel === "L3"),
  };

  const analyzeLevel = (levelMetrics: LayoutMetrics[]) => {
    if (levelMetrics.length === 0) {
      return { avgScore: 0, count: 0, problems: [] };
    }
    const avgScore =
      levelMetrics.reduce((sum, m) => sum + m.weightedScore, 0) / levelMetrics.length;
    const problems = levelMetrics
      .filter((m) => m.weightedScore < 70)
      .map((m) => m.exampleName)
      .slice(0, 5);
    return { avgScore, count: levelMetrics.length, problems };
  };

  return {
    L0: analyzeLevel(byLevel.L0),
    L1: analyzeLevel(byLevel.L1),
    L2: analyzeLevel(byLevel.L2),
    L3: analyzeLevel(byLevel.L3),
  };
}

function analyzeSizeIssues(metrics: LayoutMetrics[]): SizeIssues {
  const small = metrics.filter((m) => m.nodeCount < 10);
  const medium = metrics.filter((m) => m.nodeCount >= 10 && m.nodeCount < 30);
  const large = metrics.filter((m) => m.nodeCount >= 30);

  const analyzeSize = (sizeMetrics: LayoutMetrics[]) => {
    if (sizeMetrics.length === 0) {
      return { avgScore: 0, count: 0, examples: [] };
    }
    const avgScore = sizeMetrics.reduce((sum, m) => sum + m.weightedScore, 0) / sizeMetrics.length;
    const examples = sizeMetrics
      .filter((m) => m.weightedScore < 70)
      .map((m) => m.exampleName)
      .slice(0, 5);
    return { avgScore, count: sizeMetrics.length, examples };
  };

  return {
    small: analyzeSize(small),
    medium: analyzeSize(medium),
    large: analyzeSize(large),
  };
}

function analyzeHierarchyIssues(metrics: LayoutMetrics[]): HierarchyIssues {
  const withHierarchy = metrics.filter((m) => m.hasHierarchy);
  const withoutHierarchy = metrics.filter((m) => !m.hasHierarchy);

  const analyze = (hierarchyMetrics: LayoutMetrics[]) => {
    if (hierarchyMetrics.length === 0) {
      return { avgScore: 0, count: 0, problems: [] };
    }
    const avgScore =
      hierarchyMetrics.reduce((sum, m) => sum + m.weightedScore, 0) / hierarchyMetrics.length;
    const problems = hierarchyMetrics
      .filter((m) => m.weightedScore < 70 || m.parentChildViolations > 0)
      .map((m) => m.exampleName)
      .slice(0, 5);
    return { avgScore, count: hierarchyMetrics.length, problems };
  };

  return {
    withHierarchy: analyze(withHierarchy),
    withoutHierarchy: analyze(withoutHierarchy),
  };
}

function generateRecommendations(
  topProblems: ProblemCase[],
  violations: ViolationSummary,
  engineMismatches: EngineMismatch[],
  levelIssues: LevelIssues,
  sizeIssues: SizeIssues,
  hierarchyIssues: HierarchyIssues
): string[] {
  const recommendations: string[] = [];

  if (violations.overlappingNodes.count > 0) {
    recommendations.push(
      `Fix overlapping nodes: ${violations.overlappingNodes.count} examples affected. Increase node spacing or improve overlap detection.`
    );
  }

  if (violations.edgeCrossings.count > 0) {
    recommendations.push(
      `Reduce edge crossings: ${violations.edgeCrossings.count} examples affected. Improve edge routing algorithm.`
    );
  }

  if (violations.parentChildViolations.count > 0) {
    recommendations.push(
      `Fix parent-child containment: ${violations.parentChildViolations.count} examples affected. Ensure parent nodes properly contain children.`
    );
  }

  if (engineMismatches.length > 0) {
    recommendations.push(
      `Fix engine selection: ${engineMismatches.length} examples may benefit from different engine. Review layout rules.`
    );
  }

  if (levelIssues.L2.avgScore < 70) {
    recommendations.push(
      `Improve L2 level layouts: Average score ${levelIssues.L2.avgScore.toFixed(1)}. Consider specialized rules for container level.`
    );
  }

  if (hierarchyIssues.withHierarchy.avgScore < hierarchyIssues.withoutHierarchy.avgScore) {
    recommendations.push(
      `Improve hierarchical layouts: Average score ${hierarchyIssues.withHierarchy.avgScore.toFixed(1)} vs ${hierarchyIssues.withoutHierarchy.avgScore.toFixed(1)}. Enhance parent-child sizing.`
    );
  }

  return recommendations;
}

/**
 * Main execution
 */
if (require.main === module) {
  const resultsDir = path.join(__dirname, "../tests/results");
  const baselineFile = path.join(resultsDir, "baseline-metrics.json");

  if (!fs.existsSync(baselineFile)) {
    console.error(`Baseline metrics file not found: ${baselineFile}`);
    console.error('Run "npm run test:baseline" first to collect metrics.');
    process.exit(1);
  }

  const metrics = loadMetrics(baselineFile);
  const analysis = analyzeMetrics(metrics);

  // Save analysis
  const analysisFile = path.join(resultsDir, `analysis-${Date.now()}.json`);
  fs.writeFileSync(analysisFile, JSON.stringify(analysis, null, 2));

  // Print summary
  console.log("\n=== Metrics Analysis ===\n");
  console.log(`Total Examples: ${analysis.totalExamples}`);
  console.log(`Average Score: ${analysis.averageScore.toFixed(1)}/100`);
  console.log(`Passing Rate: ${analysis.passingRate.toFixed(1)}%`);
  console.log(`\nTop 10 Problem Cases:`);
  analysis.topProblems.forEach((p, i) => {
    console.log(`  ${i + 1}. ${p.exampleName}: ${p.score.toFixed(1)}/100 (${p.grade})`);
    console.log(`     Issues: ${p.issues.join(", ")}`);
  });

  console.log(`\nRecommendations:`);
  analysis.recommendations.forEach((r, i) => {
    console.log(`  ${i + 1}. ${r}`);
  });

  console.log(`\n✅ Analysis saved to: ${analysisFile}`);
}
