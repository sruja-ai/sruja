/**
 * Debug Collector - Collects debug information during layout process
 */

import type { DebugInfo, DebugOptions, PhaseDebugInfo } from "../core/types";

export function createDebugCollector(options: DebugOptions) {
  const phases: PhaseDebugInfo[] = [];
  const warnings: string[] = [];
  const errors: string[] = [];
  const metrics = new Map<string, unknown>();
  const heatMap = options.showHeatmap ? new Map<string, number>() : undefined;
  const intermediateResults = options.saveIntermediates ? new Map<string, unknown>() : undefined;

  return (): DebugInfo => ({
    phases: [...phases],
    warnings: [...warnings],
    errors: [...errors],
    metrics: new Map(metrics),
    heatMap,
    intermediateResults,
  });
}
