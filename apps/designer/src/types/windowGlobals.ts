// apps/designer/src/types/windowGlobals.ts
// Type definitions for window global variables used for debugging and development

import type { Node, Edge } from "@xyflow/react";
import type { DiagramQualityMetrics } from "../utils/diagramQuality";
import type { PerformanceProfiler } from "../utils/performanceProfiler";
import type { C4NodeData } from "./index";

/**
 * Window global variables interface
 * These are used for debugging and development tools
 */
export interface WindowGlobals {
  /**
   * Flag to enable new layout engine
   */
  __USE_NEW_LAYOUT_ENGINE__?: string;

  /**
   * Current layout quality metrics
   */
  __LAYOUT_METRICS__?: DiagramQualityMetrics;

  /**
   * Whether heatmap overlay is visible
   */
  __LAYOUT_HEATMAP_VISIBLE__?: boolean;

  /**
   * Function to toggle heatmap visibility
   */
  __LAYOUT_TOGGLE_HEATMAP__?: (visible: boolean) => void;

  /**
   * Performance profiler instance
   */
  __PERFORMANCE_PROFILER__?: PerformanceProfiler;

  /**
   * Current graph state (nodes and edges)
   */
  __CYBER_GRAPH__?: {
    nodes: Node<C4NodeData>[];
    edges: Edge[];
  };
}

/**
 * Extend Window interface with our globals
 */
declare global {
  interface Window extends WindowGlobals {}
}

/**
 * Type-safe getter for layout engine flag
 */
export function getLayoutEngineFlag(): boolean {
  return window.__USE_NEW_LAYOUT_ENGINE__ === "true";
}

/**
 * Type-safe setter for layout engine flag
 */
export function setLayoutEngineFlag(enabled: boolean): void {
  if (enabled) {
    window.__USE_NEW_LAYOUT_ENGINE__ = "true";
  } else {
    delete window.__USE_NEW_LAYOUT_ENGINE__;
  }
}

/**
 * Type-safe getter for layout metrics
 */
export function getLayoutMetrics(): DiagramQualityMetrics | null {
  return window.__LAYOUT_METRICS__ ?? null;
}

/**
 * Type-safe setter for layout metrics
 */
export function setLayoutMetrics(metrics: DiagramQualityMetrics): void {
  window.__LAYOUT_METRICS__ = metrics;
}

/**
 * Type-safe getter for heatmap visibility
 */
export function getHeatmapVisible(): boolean {
  return window.__LAYOUT_HEATMAP_VISIBLE__ ?? false;
}

/**
 * Type-safe setter for heatmap visibility
 */
export function setHeatmapVisible(visible: boolean): void {
  window.__LAYOUT_HEATMAP_VISIBLE__ = visible;
}

/**
 * Type-safe setter for heatmap toggle function
 */
export function setHeatmapToggle(
  toggleFn: (visible: boolean) => void
): void {
  window.__LAYOUT_TOGGLE_HEATMAP__ = toggleFn;
}

/**
 * Type-safe getter for heatmap toggle function
 */
export function getHeatmapToggle(): ((visible: boolean) => void) | null {
  return window.__LAYOUT_TOGGLE_HEATMAP__ ?? null;
}

/**
 * Type-safe getter for performance profiler
 */
export function getPerformanceProfiler(): PerformanceProfiler | null {
  return window.__PERFORMANCE_PROFILER__ ?? null;
}

/**
 * Type-safe setter for performance profiler
 */
export function setPerformanceProfiler(profiler: PerformanceProfiler): void {
  window.__PERFORMANCE_PROFILER__ = profiler;
}

/**
 * Type-safe setter for graph state
 */
export function setGraphState(
  nodes: Node<C4NodeData>[],
  edges: Edge[]
): void {
  window.__CYBER_GRAPH__ = { nodes, edges };
}
