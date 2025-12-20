/**
 * Core Layout Engine - Orchestrates modular layout pipeline
 */

import type {
  C4Graph,
  C4ViewState,
  LayoutOptions,
  LayoutResult,
  LayoutContext,
  LayoutPhase,
  ProgressCallback,
  LayoutValidationError,
  SpatialIndex,
} from "./types";

import { DefaultLayoutOptions } from "./types";
import { createSpatialIndex } from "../spatial/quadtree";
import { createQualityEvaluator } from "../quality/evaluator";
import { createMetricsCalculator } from "../metrics/calculator";
import { createDebugCollector } from "../debug/collector";

import { createHierarchyPhase } from "../phases/hierarchy";
import { createSizingPhase } from "../phases/sizing";
import { createLayoutPhase } from "../phases/layout";
import { createEdgeRoutingPhase } from "../phases/edge-routing";
import { createOptimizationPhase } from "../phases/optimization";
import { createValidationPhase } from "../phases/validation";

export class LayoutEngine {
  private readonly phases: Map<string, LayoutPhase> = new Map();
  private readonly spatialIndexFactory: () => SpatialIndex;
  private readonly options: LayoutOptions;

  constructor(options: LayoutOptions = DefaultLayoutOptions) {
    this.options = options;
    this.spatialIndexFactory = () => createSpatialIndex(options.performance.spatialIndexing);
    this.initializePhases();
  }

  public async layout(
    graph: C4Graph,
    view: C4ViewState,
    customOptions?: Partial<LayoutOptions>,
    onProgress?: ProgressCallback
  ): Promise<LayoutResult> {
    const startTime = Date.now();
    const mergedOptions = this.mergeOptions(customOptions);

    const validationErrors = this.validateInput(graph, mergedOptions);
    if (validationErrors.length > 0) {
      throw new Error(
        `Layout validation failed: ${validationErrors.map((e) => e.message).join(", ")}`
      );
    }

    let context = this.createInitialContext(graph, view, mergedOptions);

    try {
      for (const phase of this.getExecutionOrder()) {
        if (onProgress) {
          onProgress(phase.name, 0, `Starting ${phase.description}`);
        }

        context = await this.executePhase(phase, context, onProgress);

        if (
          mergedOptions.quality.earlyExit &&
          context.qualityScore.score >= this.getTargetScore(mergedOptions.quality.targetGrade)
        ) {
          if (mergedOptions.debug.enabled) {
            context.debug.warnings.push(
              `Early exit: achieved target grade ${mergedOptions.quality.targetGrade}`
            );
          }
          break;
        }
      }

      const result = this.createLayoutResult(context, Date.now() - startTime);
      const finalValidation = this.validateResult(result, mergedOptions);
      if (finalValidation.length > 0) {
        result.debug.warnings.push(...finalValidation.map((v) => v.message));
      }

      return result;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      throw new Error(`Layout failed: ${errorMessage}`);
    }
  }

  public async layoutSync(
    graph: C4Graph,
    view: C4ViewState,
    customOptions?: Partial<LayoutOptions>
  ): Promise<LayoutResult> {
    return this.layout(graph, view, customOptions);
  }

  public addPhase(phase: LayoutPhase): void {
    if (this.phases.has(phase.name)) {
      throw new Error(`Phase '${phase.name}' already exists`);
    }
    this.phases.set(phase.name, phase);
  }

  public removePhase(phaseName: string): boolean {
    return this.phases.delete(phaseName);
  }

  public getPhase(phaseName: string): LayoutPhase | undefined {
    return this.phases.get(phaseName);
  }

  public listPhases(): readonly string[] {
    return Array.from(this.phases.keys());
  }

  private initializePhases(): void {
    const defaultPhases = [
      createHierarchyPhase(),
      createSizingPhase(),
      createLayoutPhase(),
      createEdgeRoutingPhase(),
      createOptimizationPhase(),
      createValidationPhase(),
    ];

    for (const phase of defaultPhases) {
      this.phases.set(phase.name, phase);
    }
  }

  private getExecutionOrder(): LayoutPhase[] {
    const phases = Array.from(this.phases.values());
    const ordered: LayoutPhase[] = [];
    const visited = new Set<string>();

    const visit = (phase: LayoutPhase) => {
      if (visited.has(phase.name)) return;

      for (const depName of phase.dependencies) {
        const dep = this.phases.get(depName);
        if (dep) {
          visit(dep);
        }
      }

      visited.add(phase.name);
      ordered.push(phase);
    };

    for (const phase of phases) {
      visit(phase);
    }

    return ordered;
  }

  private async executePhase(
    phase: LayoutPhase,
    context: LayoutContext,
    onProgress?: ProgressCallback
  ): Promise<LayoutContext> {
    const phaseStart = Date.now();

    try {
      if (phase.validate && !phase.validate(context)) {
        if (this.options.debug.enabled) {
          context.debug.warnings.push(`Phase '${phase.name}' validation failed, skipping`);
        }
        return context;
      }

      let newContext = phase.execute(context);
      if (newContext instanceof Promise) {
        newContext = await newContext;
      }

      const duration = Date.now() - phaseStart;
      context.debug.phases.push({
        name: phase.name,
        duration,
        nodesProcessed: newContext.nodes.size,
        improvements: new Map(),
        issues: [],
      });

      if (onProgress) {
        onProgress(phase.name, 100, `Completed ${phase.description}`);
      }

      return newContext;
    } catch (error) {
      if (phase.rollback) {
        try {
          context = phase.rollback(context);
          context.debug.warnings.push(`Phase '${phase.name}' failed but rollback succeeded`);
          return context;
        } catch {
          throw new Error(`Phase '${phase.name}' failed and rollback also failed`);
        }
      } else {
        throw error;
      }
    }
  }

  private createInitialContext(
    graph: C4Graph,
    view: C4ViewState,
    options: LayoutOptions
  ): LayoutContext {
    const qualityEvaluator = createQualityEvaluator(options.quality);
    const metricsCalculator = createMetricsCalculator();

    return {
      graph,
      view,
      options,
      nodes: new Map(),
      edges: new Map(),
      spatialIndex: this.spatialIndexFactory(),
      qualityScore: {
        grade: "F",
        score: 0,
        metrics: new Map(),
        violations: [],
        recommendations: [],
      },
      metrics: {
        totalNodes: graph.nodes.size,
        visibleNodes: 0,
        totalEdges: graph.relationships.length,
        edgeCrossings: 0,
        totalEdgeLength: 0,
        averageEdgeLength: 0,
        edgeBends: 0,
        averageBends: 0,
        overlaps: 0,
        containmentViolations: 0,
        aspectRatio: 0,
        coverage: 0,
        uniformity: 0,
        balance: 0,
        compactness: 0,
        processingTime: 0,
        memoryUsage: 0,
      },
      debug: createDebugCollector(options.debug)(),
      timestamp: Date.now(),
      qualityEvaluator,
      metricsCalculator,
    };
  }

  private mergeOptions(custom?: Partial<LayoutOptions>): LayoutOptions {
    if (!custom) return this.options;

    return {
      ...this.options,
      ...custom,
      optimization: {
        enabled: custom.optimization?.enabled ?? this.options.optimization?.enabled ?? false,
        maxIterations:
          custom.optimization?.maxIterations ?? this.options.optimization?.maxIterations ?? 10,
        tolerance: custom.optimization?.tolerance ?? this.options.optimization?.tolerance ?? 0.01,
        aggressiveness:
          custom.optimization?.aggressiveness ?? this.options.optimization?.aggressiveness ?? 1.0,
        phases: custom.optimization?.phases ?? this.options.optimization?.phases ?? [],
      },
      edgeRouting: { ...this.options.edgeRouting, ...custom.edgeRouting },
      spacing: { ...this.options.spacing, ...custom.spacing },
      alignment: { ...this.options.alignment, ...custom.alignment },
      quality: { ...this.options.quality, ...custom.quality },
      performance: { ...this.options.performance, ...custom.performance },
      debug: { ...this.options.debug, ...custom.debug },
    };
  }

  private validateInput(graph: C4Graph, options: LayoutOptions): LayoutValidationError[] {
    const errors: LayoutValidationError[] = [];

    if (graph.nodes.size === 0) {
      errors.push({
        type: "error",
        code: "EMPTY_GRAPH",
        message: "Graph must contain at least one node",
        fixable: false,
      });
    }

    for (const rel of graph.relationships) {
      if (!graph.nodes.has(rel.from)) {
        errors.push({
          type: "error",
          code: "MISSING_SOURCE_NODE",
          message: `Relationship '${rel.id}' references non-existent source node '${rel.from}'`,
          nodeIds: [rel.from],
          edgeIds: [rel.id],
          fixable: false,
        });
      }

      if (!graph.nodes.has(rel.to)) {
        errors.push({
          type: "error",
          code: "MISSING_TARGET_NODE",
          message: `Relationship '${rel.id}' references non-existent target node '${rel.to}'`,
          nodeIds: [rel.to],
          edgeIds: [rel.id],
          fixable: false,
        });
      }
    }

    if (graph.nodes.size > options.performance.maxNodes) {
      errors.push({
        type: "warning",
        code: "EXCESSIVE_NODES",
        message: `Graph has ${graph.nodes.size} nodes, exceeding recommended limit of ${options.performance.maxNodes}`,
        fixable: true,
        suggestion: "Consider filtering nodes or increasing performance.maxNodes",
      });
    }

    if (graph.relationships.length > options.performance.maxEdges) {
      errors.push({
        type: "warning",
        code: "EXCESSIVE_EDGES",
        message: `Graph has ${graph.relationships.length} edges, exceeding recommended limit of ${options.performance.maxEdges}`,
        fixable: true,
        suggestion: "Consider filtering edges or increasing performance.maxEdges",
      });
    }

    return errors;
  }

  private validateResult(result: LayoutResult, options: LayoutOptions): LayoutValidationError[] {
    const errors: LayoutValidationError[] = [];

    if (result.quality.violations.some((v) => v.severity === "critical")) {
      errors.push({
        type: "error",
        code: "CRITICAL_QUALITY_ISSUES",
        message: "Layout has critical quality violations",
        fixable: true,
        suggestion: "Enable optimization phases or adjust quality settings",
      });
    }

    const targetScore = this.getTargetScore(options.quality.targetGrade);
    if (result.quality.score < targetScore) {
      errors.push({
        type: "warning",
        code: "QUALITY_TARGET_NOT_MET",
        message: `Layout score ${result.quality.score} is below target ${targetScore} for grade ${options.quality.targetGrade}`,
        fixable: true,
        suggestion: "Increase optimization aggressiveness or run additional optimization passes",
      });
    }

    return errors;
  }

  private getTargetScore(grade: "A" | "B" | "C"): number {
    switch (grade) {
      case "A":
        return 90;
      case "B":
        return 80;
      case "C":
        return 70;
      default:
        return 80;
    }
  }

  private createLayoutResult(context: LayoutContext, processingTime: number): LayoutResult {
    let minX = Infinity,
      minY = Infinity,
      maxX = -Infinity,
      maxY = -Infinity;

    for (const node of context.nodes.values()) {
      if (!node.visible) continue;
      minX = Math.min(minX, node.bbox.x);
      minY = Math.min(minY, node.bbox.y);
      maxX = Math.max(maxX, node.bbox.x + node.bbox.width);
      maxY = Math.max(maxY, node.bbox.y + node.bbox.height);
    }

    const bbox = {
      x: minX,
      y: minY,
      width: maxX - minX,
      height: maxY - minY,
    };

    const center = {
      x: bbox.x + bbox.width / 2,
      y: bbox.y + bbox.height / 2,
    };

    const finalMetrics = {
      ...context.metrics,
      processingTime,
      memoryUsage: this.estimateMemoryUsage(context),
    };

    return {
      nodes: context.nodes,
      edges: Array.from(context.edges.values()),
      bbox,
      center,
      metrics: finalMetrics,
      quality: context.qualityScore,
      debug: context.debug,
      timestamp: Date.now(),
    };
  }

  private estimateMemoryUsage(context: LayoutContext): number {
    let size = 0;
    size += context.nodes.size * 200;
    size += context.edges.size * 100;
    size += context.spatialIndex.size * 50;
    if (context.debug.intermediateResults) {
      size += context.debug.intermediateResults.size * 1000;
    }
    return size;
  }
}

export function createLayoutEngine(options?: Partial<LayoutOptions>): LayoutEngine {
  return new LayoutEngine({ ...DefaultLayoutOptions, ...options });
}

export type { LayoutOptions };
