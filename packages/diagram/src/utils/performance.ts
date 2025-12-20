/**
 * Performance Monitoring Utilities
 * 
 * FAANG best practice: Measure, don't guess. Provides utilities for
 * profiling layout operations and component renders.
 */

/** Performance measurement result */
export interface PerformanceMetrics {
    /** Name of the operation */
    name: string;
    /** Duration in milliseconds */
    durationMs: number;
    /** Timestamp when measurement started */
    startTime: number;
    /** Optional metadata about the operation */
    metadata?: Record<string, unknown>;
}

/** Performance measurement callback */
export type PerformanceCallback = (metrics: PerformanceMetrics) => void;

// Global performance callback for centralized logging
let globalCallback: PerformanceCallback | null = null;

/**
 * Set a global callback to receive all performance measurements.
 * Useful for sending metrics to analytics or logging.
 */
export function setPerformanceCallback(callback: PerformanceCallback | null): void {
    globalCallback = callback;
}

/**
 * Measure execution time of a synchronous function.
 * 
 * @example
 * ```ts
 * const result = measure("calculateLayout", () => {
 *   return complexLayoutCalculation(nodes);
 * });
 * ```
 */
export function measure<T>(name: string, fn: () => T, metadata?: Record<string, unknown>): T {
    const startTime = performance.now();
    try {
        return fn();
    } finally {
        const durationMs = performance.now() - startTime;
        const metrics: PerformanceMetrics = { name, durationMs, startTime, metadata };
        globalCallback?.(metrics);

        // Log slow operations in development
        if (process.env.NODE_ENV === "development" && durationMs > 16) {
            console.warn(`[Perf] ${name} took ${durationMs.toFixed(2)}ms`, metadata);
        }
    }
}

/**
 * Measure execution time of an async function.
 * 
 * @example
 * ```ts
 * const result = await measureAsync("fetchData", async () => {
 *   return await fetch('/api/data');
 * });
 * ```
 */
export async function measureAsync<T>(
    name: string,
    fn: () => Promise<T>,
    metadata?: Record<string, unknown>
): Promise<T> {
    const startTime = performance.now();
    try {
        return await fn();
    } finally {
        const durationMs = performance.now() - startTime;
        const metrics: PerformanceMetrics = { name, durationMs, startTime, metadata };
        globalCallback?.(metrics);

        if (process.env.NODE_ENV === "development" && durationMs > 100) {
            console.warn(`[Perf] ${name} took ${durationMs.toFixed(2)}ms`, metadata);
        }
    }
}

/**
 * Create a profiler for tracking multiple related measurements.
 * Useful for profiling multi-phase operations.
 * 
 * @example
 * ```ts
 * const profiler = createProfiler("layoutPipeline");
 * profiler.mark("buildHierarchy");
 * const tree = buildHierarchy(graph);
 * profiler.mark("calculateSizes");
 * const sizes = calculateSizes(tree);
 * const report = profiler.end();
 * console.log(report.phases); // [{name: "buildHierarchy", durationMs: 5}, ...]
 * ```
 */
export function createProfiler(name: string): {
    mark: (phaseName: string) => void;
    end: () => ProfilerReport;
} {
    const phases: Array<{ name: string; startTime: number; durationMs?: number }> = [];
    const startTime = performance.now();
    let lastMark = startTime;

    return {
        mark(phaseName: string) {
            const now = performance.now();
            if (phases.length > 0) {
                phases[phases.length - 1].durationMs = now - lastMark;
            }
            phases.push({ name: phaseName, startTime: now });
            lastMark = now;
        },
        end() {
            const endTime = performance.now();
            if (phases.length > 0) {
                phases[phases.length - 1].durationMs = endTime - lastMark;
            }
            const report: ProfilerReport = {
                name,
                totalDurationMs: endTime - startTime,
                phases: phases.map((p) => ({ name: p.name, durationMs: p.durationMs ?? 0 })),
            };
            globalCallback?.({
                name,
                durationMs: report.totalDurationMs,
                startTime,
                metadata: { phases: report.phases },
            });
            return report;
        },
    };
}

export interface ProfilerReport {
    name: string;
    totalDurationMs: number;
    phases: Array<{ name: string; durationMs: number }>;
}

/**
 * Memoization with size-limited cache.
 * FAANG pattern: Bounded caches to prevent memory leaks.
 * 
 * @example
 * ```ts
 * const expensiveCalc = memoizeWithLimit(
 *   (x: number, y: number) => x * y * Math.random(),
 *   { maxSize: 100, keyFn: (x, y) => `${x}-${y}` }
 * );
 * ```
 */
export function memoizeWithLimit<Args extends unknown[], Result>(
    fn: (...args: Args) => Result,
    options: {
        maxSize?: number;
        keyFn?: (...args: Args) => string;
    } = {}
): (...args: Args) => Result {
    const { maxSize = 100, keyFn = (...args) => JSON.stringify(args) } = options;
    const cache = new Map<string, { value: Result; timestamp: number }>();

    return (...args: Args): Result => {
        const key = keyFn(...args);
        const cached = cache.get(key);

        if (cached) {
            return cached.value;
        }

        const result = fn(...args);

        // Evict oldest entries if at capacity
        if (cache.size >= maxSize) {
            const oldestKey = [...cache.entries()]
                .sort((a, b) => a[1].timestamp - b[1].timestamp)[0]?.[0];
            if (oldestKey) {
                cache.delete(oldestKey);
            }
        }

        cache.set(key, { value: result, timestamp: Date.now() });
        return result;
    };
}

/**
 * Debounce a function to prevent rapid successive calls.
 * 
 * @param fn - Function to debounce
 * @param delayMs - Delay in milliseconds
 * @returns Debounced function
 */
export function debounce<Args extends unknown[]>(
    fn: (...args: Args) => void,
    delayMs: number
): (...args: Args) => void {
    let timeoutId: ReturnType<typeof setTimeout> | null = null;

    return (...args: Args) => {
        if (timeoutId) {
            clearTimeout(timeoutId);
        }
        timeoutId = setTimeout(() => {
            fn(...args);
            timeoutId = null;
        }, delayMs);
    };
}

/**
 * Throttle a function to limit call frequency.
 * 
 * @param fn - Function to throttle
 * @param limitMs - Minimum time between calls
 * @returns Throttled function
 */
export function throttle<Args extends unknown[]>(
    fn: (...args: Args) => void,
    limitMs: number
): (...args: Args) => void {
    let lastCall = 0;
    let timeoutId: ReturnType<typeof setTimeout> | null = null;

    return (...args: Args) => {
        const now = Date.now();
        const elapsed = now - lastCall;

        if (elapsed >= limitMs) {
            lastCall = now;
            fn(...args);
        } else if (!timeoutId) {
            timeoutId = setTimeout(() => {
                lastCall = Date.now();
                fn(...args);
                timeoutId = null;
            }, limitMs - elapsed);
        }
    };
}

/**
 * Request idle callback with fallback.
 * Schedule work during browser idle time.
 */
export function scheduleIdleWork(
    work: () => void,
    options: { timeout?: number } = {}
): void {
    const { timeout = 1000 } = options;

    if (typeof requestIdleCallback !== "undefined") {
        requestIdleCallback(() => work(), { timeout });
    } else {
        // Fallback for environments without requestIdleCallback
        setTimeout(work, 0);
    }
}

/**
 * Batch multiple updates into a single frame.
 * Useful for preventing layout thrashing.
 */
export function batchUpdates(): {
    add: (update: () => void) => void;
    flush: () => void;
} {
    const updates: Array<() => void> = [];
    let scheduled = false;

    return {
        add(update: () => void) {
            updates.push(update);
            if (!scheduled) {
                scheduled = true;
                requestAnimationFrame(() => {
                    scheduled = false;
                    const batch = updates.splice(0, updates.length);
                    batch.forEach((fn) => fn());
                });
            }
        },
        flush() {
            const batch = updates.splice(0, updates.length);
            batch.forEach((fn) => fn());
        },
    };
}
