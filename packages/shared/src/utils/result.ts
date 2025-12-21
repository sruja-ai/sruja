// packages/shared/src/utils/result.ts
// Result/Either pattern for functional error handling (FAANG best practice)

/**
 * Result type for functional error handling.
 * 
 * @public
 * @remarks
 * Inspired by Rust's Result type and functional programming patterns.
 * Prevents throwing exceptions and provides explicit error handling.
 * 
 * @example
 * ```typescript
 * const result = parseData(input);
 * if (result.ok) {
 *   console.log(result.value);
 * } else {
 *   console.error(result.error);
 * }
 * ```
 */
export type Result<T, E = Error> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly error: E };

/**
 * Create a successful Result.
 * 
 * @public
 * @param value - The success value
 * @returns Result with ok=true
 * 
 * @example
 * ```typescript
 * const result = ok(42);
 * ```
 */
export function ok<T>(value: T): Result<T, never> {
  return { ok: true, value };
}

/**
 * Create a failed Result.
 * 
 * @public
 * @param error - The error value
 * @returns Result with ok=false
 * 
 * @example
 * ```typescript
 * const result = err(new Error('Failed'));
 * ```
 */
export function err<E>(error: E): Result<never, E> {
  return { ok: false, error };
}

/**
 * Wrap a function that may throw into a Result.
 * 
 * @public
 * @param fn - Function that may throw
 * @returns Result containing the return value or error
 * 
 * @example
 * ```typescript
 * const result = tryCatch(() => JSON.parse(input));
 * ```
 */
export function tryCatch<T, E = Error>(
  fn: () => T
): Result<T, E> {
  try {
    return ok(fn());
  } catch (error) {
    return err(error as E);
  }
}

/**
 * Wrap an async function that may throw into a Result.
 * 
 * @public
 * @param fn - Async function that may throw
 * @returns Promise resolving to Result
 * 
 * @example
 * ```typescript
 * const result = await tryCatchAsync(async () => await fetch(url));
 * ```
 */
export async function tryCatchAsync<T, E = Error>(
  fn: () => Promise<T>
): Promise<Result<T, E>> {
  try {
    const value = await fn();
    return ok(value);
  } catch (error) {
    return err(error as E);
  }
}

/**
 * Map over a Result's value.
 * 
 * @public
 * @param result - Result to map over
 * @param fn - Mapping function
 * @returns New Result with transformed value
 * 
 * @example
 * ```typescript
 * const doubled = map(ok(21), x => x * 2); // ok(42)
 * ```
 */
export function map<T, U, E>(
  result: Result<T, E>,
  fn: (value: T) => U
): Result<U, E> {
  return result.ok ? ok(fn(result.value)) : result;
}

/**
 * Map over a Result's error.
 * 
 * @public
 * @param result - Result to map over
 * @param fn - Error mapping function
 * @returns New Result with transformed error
 * 
 * @example
 * ```typescript
 * const mapped = mapErr(err('error'), e => new Error(e)); // err(Error)
 * ```
 */
export function mapErr<T, E, F>(
  result: Result<T, E>,
  fn: (error: E) => F
): Result<T, F> {
  return result.ok ? result : err(fn(result.error));
}

/**
 * Chain Results together (flatMap/bind).
 * 
 * @public
 * @param result - Result to chain from
 * @param fn - Function returning a Result
 * @returns New Result
 * 
 * @example
 * ```typescript
 * const chained = andThen(ok(2), x => ok(x * 2)); // ok(4)
 * ```
 */
export function andThen<T, U, E>(
  result: Result<T, E>,
  fn: (value: T) => Result<U, E>
): Result<U, E> {
  return result.ok ? fn(result.value) : result;
}

/**
 * Unwrap Result, throwing if error.
 * 
 * @public
 * @param result - Result to unwrap
 * @returns The value if ok, throws if error
 * @throws The error if result is not ok
 * 
 * @example
 * ```typescript
 * const value = unwrap(ok(42)); // 42
 * unwrap(err(new Error())); // throws
 * ```
 */
export function unwrap<T, E>(result: Result<T, E>): T {
  if (result.ok) {
    return result.value;
  }
  throw result.error;
}

/**
 * Unwrap Result with default value if error.
 * 
 * @public
 * @param result - Result to unwrap
 * @param defaultValue - Default value if error
 * @returns The value if ok, defaultValue if error
 * 
 * @example
 * ```typescript
 * const value = unwrapOr(ok(42), 0); // 42
 * const value2 = unwrapOr(err(new Error()), 0); // 0
 * ```
 */
export function unwrapOr<T, E>(result: Result<T, E>, defaultValue: T): T {
  return result.ok ? result.value : defaultValue;
}

/**
 * Unwrap Result with function to compute default if error.
 * 
 * @public
 * @param result - Result to unwrap
 * @param fn - Function to compute default value
 * @returns The value if ok, fn(error) if error
 * 
 * @example
 * ```typescript
 * const value = unwrapOrElse(err(new Error()), e => 0); // 0
 * ```
 */
export function unwrapOrElse<T, E>(
  result: Result<T, E>,
  fn: (error: E) => T
): T {
  return result.ok ? result.value : fn(result.error);
}

