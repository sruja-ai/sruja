// apps/designer/src/types/windowGlobals.ts
// Type definitions for window global variables used for debugging and development

import type { Node, Edge } from "@xyflow/react";
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
