#!/usr/bin/env tsx
// packages/diagram/tests/improve-iteratively.ts
// Reinforcement learning-style iterative improvement
import * as fs from "fs";
import * as path from "path";
import { execSync } from "child_process";
import type { LayoutMetrics } from "./utils/metrics-collector";

const RESULTS_DIR = path.join(process.cwd(), "tests", "results");
const LATEST_FILE = path.join(RESULTS_DIR, "latest-metrics.json");
const ITERATION_FILE = path.join(RESULTS_DIR, "iteration-state.json");

interface IterationState {
  iteration: number;
  previousScores: Map<string, number>;
  improvements: Array<{
    iteration: number;
    changes: string[];
    before: Map<string, number>;
    after: Map<string, number>;
  }>;
}

function loadMetrics(): Map<string, LayoutMetrics> {
  if (!fs.existsSync(LATEST_FILE)) {
    throw new Error("No metrics found. Run e2e tests first.");
  }
  const data = JSON.parse(fs.readFileSync(LATEST_FILE, "utf-8"));
  const metrics = new Map<string, LayoutMetrics>();
  data.metrics.forEach((m: LayoutMetrics) => {
    metrics.set(m.exampleName, m);
  });
  return metrics;
}

function analyzeState(metrics: Map<string, LayoutMetrics>): {
  avgScore: number;
  minScore: number;
  maxScore: number;
  belowB: Array<{ name: string; score: number; issues: string[] }>;
  criticalIssues: Map<string, number>;
} {
  const scores = Array.from(metrics.values()).map((m) => m.weightedScore);
  const avgScore = scores.reduce((a, b) => a + b, 0) / scores.length;
  const minScore = Math.min(...scores);
  const maxScore = Math.max(...scores);

  const belowB: Array<{ name: string; score: number; issues: string[] }> = [];
  const criticalIssues = new Map<string, number>();

  metrics.forEach((m, name) => {
    if (m.weightedScore < 80) {
      const issues: string[] = [];
      if (m.overlappingNodes > 0) issues.push(`${m.overlappingNodes} overlaps`);
      if (m.edgeCrossings > 2) issues.push(`${m.edgeCrossings} crossings`);
      if (m.edgesOverNodes > 0) issues.push(`${m.edgesOverNodes} edges over nodes`);
      if (m.parentChildViolations > 0)
        issues.push(`${m.parentChildViolations} containment violations`);
      if (m.spacingViolations > 5) issues.push(`${m.spacingViolations} spacing violations`);
      if (m.qualityMetrics?.edgeLabelOverlaps > 0)
        issues.push(`${m.qualityMetrics.edgeLabelOverlaps} label overlaps`);
      if ((m.qualityMetrics?.edgeCongestionScore || 100) < 40)
        issues.push(`congestion: ${m.qualityMetrics?.edgeCongestionScore || 0}`);

      belowB.push({ name, score: m.weightedScore, issues });

      // Aggregate critical issues
      issues.forEach((issue) => {
        const key = issue.split(":")[0] || issue;
        criticalIssues.set(key, (criticalIssues.get(key) || 0) + 1);
      });
    }
  });

  return { avgScore, minScore, maxScore, belowB, criticalIssues };
}

function generateActionPlan(state: ReturnType<typeof analyzeState>): string[] {
  const actions: string[] = [];
  const { belowB, criticalIssues } = state;

  console.log(`\n📊 State Analysis:`);
  console.log(`   Average Score: ${state.avgScore.toFixed(1)}`);
  console.log(`   Min Score: ${state.minScore.toFixed(1)}`);
  console.log(`   Max Score: ${state.maxScore.toFixed(1)}`);
  console.log(`   Examples below B (80): ${belowB.length}`);

  if (belowB.length === 0) {
    console.log(`\n✅ All examples are at B or above!`);
    return [];
  }

  console.log(`\n🔍 Critical Issues:`);
  const sortedIssues = Array.from(criticalIssues.entries()).sort((a, b) => b[1] - a[1]);
  sortedIssues.forEach(([issue, count]) => {
    console.log(`   - ${issue}: ${count} examples`);
  });

  // Generate action plan based on most common issues
  if (criticalIssues.has("containment violations")) {
    actions.push("Fix parent-child containment violations in hierarchy.ts");
  }
  if (criticalIssues.has("overlaps")) {
    actions.push("Improve overlap detection/resolution in overlap.ts");
  }
  if (criticalIssues.has("crossings")) {
    actions.push("Enhance edge routing to reduce crossings in unified-router.ts");
  }
  if (criticalIssues.has("congestion")) {
    actions.push("Improve edge bundling and distribution in edge-bundler.ts");
  }
  if (criticalIssues.has("label overlaps")) {
    actions.push("Enhance label placement algorithm in label-placer.ts");
  }
  if (criticalIssues.has("spacing violations")) {
    actions.push("Increase spacing in layout rules or sizing algorithm");
  }

  // Always check edge congestion if it's low
  const lowCongestion = Array.from(belowB.values()).some(
    (b) => b.score < 80 && state.avgScore < 85
  );
  if (lowCongestion) {
    actions.push("Review edge congestion - may need better edge routing");
  }

  return actions;
}

async function main() {
  console.log("🔄 Reinforcement Learning-Style Iterative Improvement\n");

  // Step 1: Collect current state (run e2e tests)
  console.log("📊 Step 1: Collecting metrics (running e2e tests)...");
  try {
    execSync("npx playwright test tests/iterative-optimization.spec.ts", {
      stdio: "inherit",
      cwd: process.cwd(),
    });
  } catch (error) {
    console.error("❌ Failed to run e2e tests:", error);
    process.exit(1);
  }

  // Step 2: Analyze state
  console.log("\n🔍 Step 2: Analyzing current state...");
  const metrics = loadMetrics();
  const state = analyzeState(metrics);

  // Step 3: Generate action plan
  console.log("\n💡 Step 3: Generating action plan...");
  const actions = generateActionPlan(state);

  if (actions.length === 0) {
    console.log("\n✨ Goal achieved! All examples are at B or above.");
    return;
  }

  // Step 4: Display action plan
  console.log(`\n📋 Action Plan (${actions.length} actions):`);
  actions.forEach((action, i) => {
    console.log(`   ${i + 1}. ${action}`);
  });

  console.log(`\n⚠️  Next Steps:`);
  console.log(`   1. Review the action plan above`);
  console.log(`   2. Make code improvements based on critical issues`);
  console.log(`   3. Run this script again to test improvements`);
  console.log(`   4. Iterate until all examples reach B score (80+)`);

  // Save state for comparison
  const iterationState: IterationState = {
    iteration: 1,
    previousScores: new Map(Array.from(metrics.entries()).map(([k, v]) => [k, v.weightedScore])),
    improvements: [],
  };
  fs.writeFileSync(ITERATION_FILE, JSON.stringify(iterationState, null, 2));
}

main().catch((error) => {
  console.error("Fatal error:", error);
  process.exit(1);
});
