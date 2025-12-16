/**
 * LayoutAuditor - Playwright-based visual auditor for C4 diagram layouts
 *
 * This module provides structured feedback on layout quality by analyzing
 * rendered diagrams in a Playwright browser context. It integrates with
 * the existing diagramQuality.ts metrics system.
 */
import type { Page } from "playwright";
import type { Node, Edge } from "@xyflow/react";
import type { C4NodeData } from "../types";
import {
  calculateDiagramQuality,
  DEFAULT_QUALITY_WEIGHTS,
  type DiagramQualityMetrics,
} from "../utils/diagramQuality";

/**
 * Result of a layout audit
 */
export interface AuditResult {
  /** Score from 0.0 (worst) to 1.0 (perfect) */
  score: number;
  /** Human-readable violation messages for feedback */
  violations: string[];
  /** Path to screenshot if captured */
  screenshotPath?: string;
  /** Full quality metrics from diagramQuality.ts */
  metrics: DiagramQualityMetrics;
}

/**
 * Options for the audit
 */
export interface AuditOptions {
  /** Timeout for waiting for layout to stabilize (ms) */
  timeout?: number;
  /** Whether to capture a screenshot on failure */
  captureScreenshot?: boolean;
  /** Path for screenshot if captured */
  screenshotDir?: string;
}

const DEFAULT_OPTIONS: AuditOptions = {
  timeout: 10000,
  captureScreenshot: true,
  screenshotDir: "./test-results",
};

/**
 * LayoutAuditor - Provides structured feedback on layout quality
 *
 * Usage:
 * ```typescript
 * const auditor = new LayoutAuditor(page);
 * const result = await auditor.auditLayout();
 * if (result.score < 1.0) {
 *   console.log("Violations:", result.violations);
 * }
 * ```
 */
export class LayoutAuditor {
  private page: Page;
  constructor(page: Page) {
    this.page = page;
  }

  /**
   * Audit the current layout and return structured feedback
   */
  async auditLayout(options: AuditOptions = {}): Promise<AuditResult> {
    const opts = { ...DEFAULT_OPTIONS, ...options };

    // Wait for the graph to be exposed
    await this.waitForGraphReady(opts.timeout!);

    // Get nodes and edges from the exposed state
    const { nodes, edges } = await this.getGraphState();

    if (nodes.length === 0) {
      return {
        score: 0,
        violations: ["CRITICAL: No nodes found in the diagram."],
        metrics: {} as DiagramQualityMetrics,
      };
    }

    // Get viewport size
    const viewportSize = (await this.page.viewportSize()) || {
      width: 1920,
      height: 1080,
    };

    // Calculate quality metrics using existing infrastructure
    const metrics = calculateDiagramQuality(nodes, edges, viewportSize, DEFAULT_QUALITY_WEIGHTS);

    // Generate violation messages from metrics
    const violations = this.generateViolationMessages(metrics);

    // Calculate normalized score (0.0 to 1.0)
    const score = this.calculateScore(metrics, violations);

    // Capture screenshot if there are violations
    let screenshotPath: string | undefined;
    if (opts.captureScreenshot && violations.length > 0) {
      screenshotPath = await this.captureScreenshot(opts.screenshotDir!);
    }

    return {
      score,
      violations,
      screenshotPath,
      metrics,
    };
  }

  /**
   * Wait for the graph to be ready and exposed via window.__CYBER_GRAPH__
   */
  private async waitForGraphReady(timeout: number): Promise<void> {
    try {
      // Wait for loading overlay to disappear
      await this.page.waitForSelector(".loading-overlay", {
        state: "hidden",
        timeout: timeout / 2,
      });
    } catch {
      // Loading overlay might not exist, continue
    }

    // Wait for React Flow to be ready
    await this.page.waitForSelector(".react-flow", { timeout: timeout / 2 });

    // Wait for graph state to be exposed
    await this.page.waitForFunction(
      () => {
        const graph = (window as any).__CYBER_GRAPH__;
        return graph && Array.isArray(graph.nodes) && graph.nodes.length > 0;
      },
      { timeout }
    );

    // Additional stabilization wait
    await this.page.waitForTimeout(300);
  }

  /**
   * Get nodes and edges from the exposed graph state
   */
  private async getGraphState(): Promise<{
    nodes: Node<C4NodeData>[];
    edges: Edge[];
  }> {
    return await this.page.evaluate(() => {
      const graph = (window as any).__CYBER_GRAPH__;
      return {
        nodes: graph?.nodes || [],
        edges: graph?.edges || [],
      };
    });
  }

  /**
   * Generate human-readable violation messages from quality metrics
   */
  private generateViolationMessages(metrics: DiagramQualityMetrics): string[] {
    const violations: string[] = [];

    // CRITICAL: Overlapping nodes
    if (metrics.overlappingNodes.length > 0) {
      metrics.overlappingNodes.forEach((overlap) => {
        violations.push(
          `CRITICAL: Node '${overlap.node1}' overlaps with '${overlap.node2}' ` +
            `(${overlap.overlapPercentage.toFixed(1)}% overlap).`
        );
      });
    }

    // CRITICAL: Parent-child containment violations
    if (metrics.parentChildContainment.length > 0) {
      metrics.parentChildContainment.forEach((v) => {
        violations.push(
          `CONSTRAINT: Node '${v.childId}' is ${v.violation} its parent '${v.parentId}'. ` +
            `${v.details}`
        );
      });
    }

    // WARNING: Spacing violations
    if (metrics.spacingViolations.length > 0) {
      const spacingCount = metrics.spacingViolations.length;
      if (spacingCount > 3) {
        violations.push(
          `WARNING: ${spacingCount} nodes are too close together (< ${metrics.spacingViolations[0]?.minRequired}px).`
        );
      } else {
        metrics.spacingViolations.forEach((v) => {
          violations.push(
            `WARNING: Nodes '${v.node1}' and '${v.node2}' are too close ` +
              `(${v.distance.toFixed(0)}px, min required: ${v.minRequired}px).`
          );
        });
      }
    }

    // WARNING: Edge crossings
    if (metrics.edgeCrossings > 0) {
      violations.push(
        `WARNING: ${metrics.edgeCrossings} edge crossing(s) detected. Consider repositioning nodes.`
      );
    }

    // WARNING: Edges over nodes
    if (metrics.edgesOverNodes > 0) {
      violations.push(`WARNING: ${metrics.edgesOverNodes} edge(s) passing through nodes.`);
    }

    // INFO: Edge label overlaps
    if (metrics.edgeLabelOverlaps > 0) {
      violations.push(`INFO: ${metrics.edgeLabelOverlaps} edge label(s) overlapping with nodes.`);
    }

    // INFO: Clipped labels
    if (metrics.clippedNodeLabels > 0) {
      violations.push(
        `INFO: ${metrics.clippedNodeLabels} node label(s) may be clipped or cramped.`
      );
    }

    return violations;
  }

  /**
   * Calculate normalized score from metrics
   * Returns 1.0 only if there are zero critical violations
   */
  private calculateScore(
    metrics: DiagramQualityMetrics,
    _violations: string[] // Reserved for future severity-based scoring
  ): number {
    // Check for critical violations (overlaps and containment)
    const criticalViolations =
      metrics.overlappingNodes.length + metrics.parentChildContainment.length;

    if (criticalViolations > 0) {
      // Critical violations cap the score
      return Math.max(0, Math.min(0.5, metrics.weightedScore / 100 - 0.2));
    }

    // No critical violations - use weighted score
    return Math.min(1.0, metrics.weightedScore / 100);
  }

  /**
   * Capture a screenshot and return the path
   */
  private async captureScreenshot(dir: string): Promise<string> {
    const timestamp = Date.now();
    const path = `${dir}/audit-${timestamp}.png`;
    await this.page.screenshot({ path, fullPage: true });
    return path;
  }
}

/**
 * Helper function to run a quick audit without instantiating the class
 */
export async function auditLayout(page: Page, options?: AuditOptions): Promise<AuditResult> {
  const auditor = new LayoutAuditor(page);
  return auditor.auditLayout(options);
}
