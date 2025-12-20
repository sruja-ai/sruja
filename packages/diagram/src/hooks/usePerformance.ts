/**
 * React Performance Hooks
 * 
 * Custom hooks for optimizing React component performance.
 * FAANG best practice: Measure renders, prevent unnecessary work.
 */
import { useCallback, useRef, useEffect, useState } from "react";
import type React from "react";

/**
 * Hook that returns a stable callback that always has access to the latest closure values.
 * Prevents unnecessary child re-renders when passing callbacks as props.
 * 
 * @example
 * ```tsx
 * const handleClick = useStableCallback((id: string) => {
 *   setSelectedId(id); // Always uses latest setSelectedId
 * });
 * ```
 */
export function useStableCallback<T extends (...args: never[]) => unknown>(callback: T): T {
    const callbackRef = useRef(callback);
    callbackRef.current = callback;

    return useCallback((...args: Parameters<T>) => {
        return callbackRef.current(...args);
    }, []) as T;
}

/**
 * Hook for shallow comparison of objects to memoize derived values.
 * Only recalculates when object properties actually change.
 * 
 * @example
 * ```tsx
 * const nodeData = useShallowMemo({ x, y, label }, [x, y, label]);
 * ```
 */
export function useShallowMemo<T extends object>(value: T, deps: React.DependencyList): T {
    const ref = useRef<T>(value);

    const changed = deps.length !== Object.keys(ref.current).length ||
        deps.some((dep, i) => {
            const keys = Object.keys(ref.current);
            return ref.current[keys[i] as keyof T] !== dep;
        });

    if (changed) {
        ref.current = value;
    }

    return ref.current;
}

/**
 * Hook that logs when component re-renders and why (in development).
 * Debug tool for identifying unnecessary renders.
 * 
 * @example
 * ```tsx
 * useRenderTracker("NodeComponent", { data, selected });
 * ```
 */
export function useRenderTracker(
    componentName: string,
    props: Record<string, unknown>
): void {
    const previousProps = useRef<Record<string, unknown>>({});

    useEffect(() => {
        if (process.env.NODE_ENV === "development") {
            const changedProps: string[] = [];

            for (const key of Object.keys(props)) {
                if (previousProps.current[key] !== props[key]) {
                    changedProps.push(key);
                }
            }

            if (changedProps.length > 0) {
                console.log(`[RenderTracker] ${componentName} re-rendered due to:`, changedProps);
            }
        }
        previousProps.current = { ...props };
    });
}

/**
 * Hook for debounced values.
 * Returns value that only updates after delay.
 * 
 * @example
 * ```tsx
 * const debouncedSearch = useDebouncedValue(searchTerm, 300);
 * ```
 */
export function useDebouncedValue<T>(value: T, delayMs: number): T {
    const [debouncedValue, setDebouncedValue] = useState<T>(value);
    const timeoutRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

    useEffect(() => {
        timeoutRef.current = setTimeout(() => {
            setDebouncedValue(value);
        }, delayMs);

        return () => {
            if (timeoutRef.current) {
                clearTimeout(timeoutRef.current);
            }
        };
    }, [value, delayMs]);

    return debouncedValue;
}

/**
 * Hook for throttled callbacks.
 * Limits how often a callback can be invoked.
 */
export function useThrottledCallback<T extends (...args: never[]) => void>(
    callback: T,
    limitMs: number
): T {
    const lastCall = useRef<number>(0);
    const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    return useCallback((...args: Parameters<T>) => {
        const now = Date.now();
        const elapsed = now - lastCall.current;

        if (elapsed >= limitMs) {
            lastCall.current = now;
            callback(...args);
        } else if (!timeoutRef.current) {
            timeoutRef.current = setTimeout(() => {
                lastCall.current = Date.now();
                callback(...args);
                timeoutRef.current = null;
            }, limitMs - elapsed);
        }
    }, [callback, limitMs]) as T;
}

/**
 * Hook that runs expensive calculations during idle time.
 * Result is undefined until calculation completes.
 * 
 * @example
 * ```tsx
 * const expensiveResult = useIdleCalculation(
 *   () => computeExpensiveValue(data),
 *   [data]
 * );
 * ```
 */
export function useIdleCalculation<T>(
    calculate: () => T,
    deps: React.DependencyList
): T | undefined {
    const [result, setResult] = useState<T | undefined>(undefined);

    useEffect(() => {
        let cancelled = false;

        const runCalculation = () => {
            if (!cancelled) {
                const value = calculate();
                setResult(value);
            }
        };

        if (typeof requestIdleCallback !== "undefined") {
            const id = requestIdleCallback(runCalculation, { timeout: 1000 });
            return () => {
                cancelled = true;
                cancelIdleCallback(id);
            };
        } else {
            const id = setTimeout(runCalculation, 0);
            return () => {
                cancelled = true;
                clearTimeout(id);
            };
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, deps);

    return result;
}

/**
 * Create a comparison function for React.memo that only compares specific props.
 * 
 * @example
 * ```tsx
 * export const MyComponent = memo(MyComponentInner, createPropsComparator(['id', 'label']));
 * ```
 */
export function createPropsComparator<T extends object>(
    keys: (keyof T)[]
): (prev: T, next: T) => boolean {
    return (prev, next) => {
        for (const key of keys) {
            if (prev[key] !== next[key]) {
                return false;
            }
        }
        return true;
    };
}

/**
 * Hook that provides a previous value of a prop.
 * Useful for detecting changes and transitions.
 */
export function usePrevious<T>(value: T): T | undefined {
    const ref = useRef<T | undefined>(undefined);

    useEffect(() => {
        ref.current = value;
    }, [value]);

    return ref.current;
}
