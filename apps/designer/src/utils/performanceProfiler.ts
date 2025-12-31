// apps/playground/src/utils/performanceProfiler.ts
// Performance profiling utilities for large diagram optimization

/**
 * Performance metrics for diagram rendering
 */
export interface PerformanceMetrics {
  layoutDurationMs: number;
  nodeCount: number;
  edgeCount: number;
  renderTimestamp: number;
  layoutEngine: string;
  usedWorker: boolean;
}

/**
 * Performance thresholds for warning/optimization triggers
 */
export const PERFORMANCE_THRESHOLDS = {
  // Use web worker for layout if nodes exceed this count
  WORKER_NODE_THRESHOLD: 80,
  WORKER_EDGE_THRESHOLD: 120,

  // Warn if layout takes longer than this (ms)
  SLOW_LAYOUT_WARNING_MS: 500,

  // Critical if layout takes longer than this (ms)
  SLOW_LAYOUT_CRITICAL_MS: 2000,

  // Maximum nodes before suggesting virtualization
  VIRTUALIZATION_THRESHOLD: 200,

  // Edge routing complexity threshold (nodes * edges)
  ROUTING_COMPLEXITY_WARN: 10000,
};

/**
 * Simple performance profiler for layout operations
 */
export class PerformanceProfiler {
  private metrics: PerformanceMetrics[] = [];
  private maxStoredMetrics = 50;

  /**
   * Record a layout operation
   */
  recordLayout(metrics: Omit<PerformanceMetrics, "renderTimestamp">): void {
    const fullMetrics: PerformanceMetrics = {
      ...metrics,
      renderTimestamp: Date.now(),
    };

    this.metrics.push(fullMetrics);

    // Keep only recent metrics
    if (this.metrics.length > this.maxStoredMetrics) {
      this.metrics.shift();
    }

    // Log warnings for slow layouts
    if (metrics.layoutDurationMs > PERFORMANCE_THRESHOLDS.SLOW_LAYOUT_CRITICAL_MS) {
      console.warn("[Performance] Critical: Layout took", metrics.layoutDurationMs, "ms");
    } else if (metrics.layoutDurationMs > PERFORMANCE_THRESHOLDS.SLOW_LAYOUT_WARNING_MS) {
      console.info("[Performance] Warning: Layout took", metrics.layoutDurationMs, "ms");
    }

    // Log virtualization suggestion
    if (metrics.nodeCount > PERFORMANCE_THRESHOLDS.VIRTUALIZATION_THRESHOLD) {
      console.info(
        "[Performance] Consider enabling virtualization for",
        metrics.nodeCount,
        "nodes"
      );
    }
  }

  /**
   * Get average layout duration over recent operations
   */
  getAverageLayoutDuration(): number {
    if (this.metrics.length === 0) return 0;
    const sum = this.metrics.reduce((acc, m) => acc + m.layoutDurationMs, 0);
    return sum / this.metrics.length;
  }

  /**
   * Get all recorded metrics
   */
  getMetrics(): PerformanceMetrics[] {
    return [...this.metrics];
  }

  /**
   * Get the most recent metrics
   */
  getLatestMetrics(): PerformanceMetrics | null {
    return this.metrics.length > 0 ? this.metrics[this.metrics.length - 1] : null;
  }

  /**
   * Check if worker should be used based on graph complexity
   */
  shouldUseWorker(nodeCount: number, edgeCount: number): boolean {
    return (
      nodeCount > PERFORMANCE_THRESHOLDS.WORKER_NODE_THRESHOLD ||
      edgeCount > PERFORMANCE_THRESHOLDS.WORKER_EDGE_THRESHOLD
    );
  }

  /**
   * Get routing complexity score
   */
  getRoutingComplexity(nodeCount: number, edgeCount: number): number {
    return nodeCount * edgeCount;
  }

  /**
   * Reset all metrics
   */
  clear(): void {
    this.metrics = [];
  }
}

// Singleton instance
export const performanceProfiler = new PerformanceProfiler();

/**
 * Debounce utility for layout updates
 */
export function debounce<T extends (...args: unknown[]) => unknown>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout> | null = null;

  return (...args: Parameters<T>) => {
    if (timeoutId) {
      clearTimeout(timeoutId);
    }
    timeoutId = setTimeout(() => {
      fn(...args);
      timeoutId = null;
    }, delay);
  };
}

/**
 * Throttle utility for high-frequency updates
 */
export function throttle<T extends (...args: unknown[]) => unknown>(
  fn: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle = false;

  return (...args: Parameters<T>) => {
    if (!inThrottle) {
      fn(...args);
      inThrottle = true;
      setTimeout(() => {
        inThrottle = false;
      }, limit);
    }
  };
}

// Expose for debugging
import { setPerformanceProfiler } from "../types/windowGlobals";

if (typeof window !== "undefined") {
  setPerformanceProfiler(performanceProfiler);
}
