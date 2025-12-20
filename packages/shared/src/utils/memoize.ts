// packages/shared/src/utils/memoize.ts
// Memoization utilities for performance optimization

/**
 * Memoization options.
 * 
 * @public
 */
export interface MemoizeOptions {
  /**
   * Maximum cache size (default: unlimited).
   * When exceeded, oldest entries are evicted (LRU).
   */
  readonly maxSize?: number;

  /**
   * Custom key generator function.
   * If not provided, uses JSON.stringify of arguments.
   */
  readonly keyGenerator?: (...args: unknown[]) => string;
}

/**
 * Memoize a function with optional cache size limit.
 * 
 * @public
 * @param fn - Function to memoize
 * @param options - Memoization options
 * @returns Memoized function
 * 
 * @remarks
 * Uses LRU eviction when maxSize is exceeded.
 * 
 * @example
 * ```typescript
 * const expensiveFn = memoize((x: number) => {
 *   // Expensive computation
 *   return x * x;
 * }, { maxSize: 100 });
 * ```
 */
export function memoize<T extends (...args: unknown[]) => unknown>(
  fn: T,
  options?: MemoizeOptions
): T {
  const cache = new Map<string, ReturnType<T>>();
  const maxSize = options?.maxSize ?? Infinity;
  const keyGen = options?.keyGenerator ?? ((...args) => JSON.stringify(args));

  return ((...args: Parameters<T>): ReturnType<T> => {
    const key = keyGen(...args);

    if (cache.has(key)) {
      // Move to end (LRU)
      const value = cache.get(key)!;
      cache.delete(key);
      cache.set(key, value);
      return value;
    }

    const result = fn(...args) as ReturnType<T>;

    // Evict oldest if at capacity
    if (cache.size >= maxSize) {
      const firstKey = cache.keys().next().value;
      if (firstKey !== undefined) {
        cache.delete(firstKey);
      }
    }

    cache.set(key, result);
    return result;
  }) as T;
}

/**
 * Weak memoization using WeakMap (for object keys only).
 * 
 * @public
 * @param fn - Function to memoize
 * @returns Memoized function
 * 
 * @remarks
 * Only works with object arguments. Cache is automatically garbage collected.
 * 
 * @example
 * ```typescript
 * const compute = weakMemoize((obj: { x: number }) => obj.x * 2);
 * ```
 */
export function weakMemoize<T extends (arg: object) => unknown>(
  fn: T
): T {
  const cache = new WeakMap<object, ReturnType<T>>();

  return ((arg: object): ReturnType<T> => {
    if (cache.has(arg)) {
      return cache.get(arg)!;
    }

    const result = fn(arg) as ReturnType<T>;
    cache.set(arg, result);
    return result;
  }) as T;
}

