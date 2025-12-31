// apps/designer/src/types/windowGlobals.ts
// Type definitions for window global variables used for debugging and development

import type { Node, Edge } from "@xyflow/react";
import type { PerformanceProfiler } from "../utils/performanceProfiler";
import type { C4NodeData } from "./index";

/**
 * Window global variables augmentation
 * These are used for debugging and development tools
 */
declare global {
  interface Window {
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
}

/**
 * Type-safe getter for layout engine flag
 */
export function getLayoutEngineFlag(): boolean {
  if (import.meta.env.DEV) {
    return (window as any)["__USE_NEW_LAYOUT_ENGINE__"] === "true";
  }
  return false;
}

/**
 * Type-safe setter for layout engine flag
 */
export function setLayoutEngineFlag(enabled: boolean): void {
  if (import.meta.env.DEV) {
    if (enabled) {
      (window as any)["__USE_NEW_LAYOUT_ENGINE__"] = "true";
    } else {
      delete (window as any)["__USE_NEW_LAYOUT_ENGINE__"];
    }
  }
}

/**
 * Type-safe getter for performance profiler
 */
export function getPerformanceProfiler(): PerformanceProfiler | null {
  if (import.meta.env.DEV) {
    return (window as any)["__PERFORMANCE_PROFILER__"] ?? null;
  }
  return null;
}

/**
 * Type-safe setter for performance profiler
 */
export function setPerformanceProfiler(profiler: PerformanceProfiler): void {
  if (import.meta.env.DEV) {
    (window as any)["__PERFORMANCE_PROFILER__"] = profiler;
  }
}

/**
 * Type-safe setter for graph state
 */
export function setGraphState(nodes: Node<C4NodeData>[], edges: Edge[]): void {
  if (import.meta.env.DEV) {
    (window as any)["__CYBER_GRAPH__"] = { nodes, edges };
  }
}
