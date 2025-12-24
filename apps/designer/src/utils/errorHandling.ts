// apps/designer/src/utils/errorHandling.ts
// Centralized error handling utilities

/**
 * Error types for better error categorization and handling.
 */
export const ErrorType = {
  NETWORK: "NETWORK",
  VALIDATION: "VALIDATION",
  PERMISSION: "PERMISSION",
  NOT_FOUND: "NOT_FOUND",
  UNKNOWN: "UNKNOWN",
} as const;

export type ErrorType = (typeof ErrorType)[keyof typeof ErrorType];

/**
 * Custom error class with type and context for better error handling.
 *
 * Extends the base Error class to include error type categorization
 * and additional context information for debugging.
 *
 * @class
 * @extends {Error}
 *
 * @example
 * ```ts
 * throw new AppError("Failed to load data", ErrorType.NETWORK, { url: "/api/data" });
 * ```
 */
export class AppError extends Error {
  public type: ErrorType;
  public context?: Record<string, unknown>;

  constructor(
    message: string,
    type: ErrorType = ErrorType.UNKNOWN,
    context?: Record<string, unknown>
  ) {
    super(message);
    this.name = "AppError";
    this.type = type;
    this.context = context;
    Object.setPrototypeOf(this, AppError.prototype);
  }
}

/**
 * Network-specific error wrapper.
 *
 * Used for errors related to network operations (fetch, API calls, etc.).
 * Includes optional HTTP status code for better error handling.
 *
 * @class
 * @extends {AppError}
 *
 * @example
 * ```ts
 * throw new NetworkError("Failed to fetch", 404, { endpoint: "/api/users" });
 * ```
 */
export class NetworkError extends AppError {
  public statusCode?: number;

  constructor(message: string, statusCode?: number, context?: Record<string, unknown>) {
    super(message, ErrorType.NETWORK, { ...context, statusCode });
    this.name = "NetworkError";
    this.statusCode = statusCode;
  }
}

/**
 * Validation-specific error wrapper.
 *
 * Used for errors related to data validation (form inputs, schema validation, etc.).
 * Includes optional field name for field-specific error messages.
 *
 * @class
 * @extends {AppError}
 *
 * @example
 * ```ts
 * throw new ValidationError("Email is required", "email");
 * ```
 */
export class ValidationError extends AppError {
  public field?: string;

  constructor(message: string, field?: string, context?: Record<string, unknown>) {
    super(message, ErrorType.VALIDATION, { ...context, field });
    this.name = "ValidationError";
    this.field = field;
  }
}

/**
 * Safely execute async operation with error handling.
 *
 * Wraps an async operation in try-catch and returns a result object
 * with either data or error, preventing unhandled promise rejections.
 *
 * @template T - The return type of the operation
 * @param operation - Async function to execute
 * @param errorMessage - User-friendly error message if operation fails
 * @param errorType - Type of error to categorize (default: UNKNOWN)
 * @returns Promise resolving to object with `data` and `error` properties
 *
 * @example
 * ```ts
 * const { data, error } = await safeAsync(
 *   () => fetch("/api/data").then(r => r.json()),
 *   "Failed to load data",
 *   ErrorType.NETWORK
 * );
 *
 * if (error) {
 *   console.error(error);
 *   return;
 * }
 *
 * // Use data safely
 * console.log(data);
 * ```
 */
export async function safeAsync<T>(
  operation: () => Promise<T>,
  errorMessage = "Operation failed",
  errorType: ErrorType = ErrorType.UNKNOWN
): Promise<{ data: T | null; error: AppError | null }> {
  try {
    const data = await operation();
    return { data, error: null };
  } catch (error) {
    const appError =
      error instanceof AppError
        ? error
        : new AppError(errorMessage, errorType, {
            originalError: error instanceof Error ? error.message : String(error),
          });
    return { data: null, error: appError };
  }
}

/**
 * Execute async operation with retry logic.
 *
 * Retries a failed operation up to a specified number of times with
 * exponential backoff. Only retries if `shouldRetry` returns true.
 *
 * @template T - The return type of the operation
 * @param operation - Async function to execute with retries
 * @param options - Retry configuration options
 * @param options.maxRetries - Maximum number of retry attempts (default: 3)
 * @param options.retryDelay - Base delay between retries in ms (default: 1000)
 * @param options.shouldRetry - Function to determine if error is retryable (default: always retry)
 * @returns Promise resolving to operation result
 *
 * @example
 * ```ts
 * const data = await withRetry(
 *   () => fetch("/api/data").then(r => r.json()),
 *   { maxRetries: 3, retryDelay: 1000 }
 * );
 * ```
 */
export async function withRetry<T>(
  operation: () => Promise<T>,
  options: {
    maxRetries?: number;
    retryDelay?: number;
    shouldRetry?: (error: unknown) => boolean;
  } = {}
): Promise<T> {
  const { maxRetries = 3, retryDelay = 1000, shouldRetry = () => true } = options;

  let lastError: unknown;

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await operation();
    } catch (error) {
      lastError = error;

      if (attempt < maxRetries && shouldRetry(error)) {
        await new Promise((resolve) => setTimeout(resolve, retryDelay * (attempt + 1)));
        continue;
      }

      throw error;
    }
  }

  throw lastError;
}

/**
 * Execute async operation with timeout.
 *
 * Wraps an async operation with a timeout. If the operation doesn't
 * complete within the specified time, it rejects with a timeout error.
 *
 * @template T - The return type of the operation
 * @param operation - Async function to execute
 * @param timeoutMs - Timeout duration in milliseconds
 * @param timeoutMessage - Error message for timeout (default: "Operation timed out")
 * @returns Promise resolving to operation result
 *
 * @example
 * ```ts
 * const data = await withTimeout(
 *   () => fetch("/api/data").then(r => r.json()),
 *   5000,
 *   "Request timed out after 5 seconds"
 * );
 * ```
 */
export async function withTimeout<T>(
  operation: () => Promise<T>,
  timeoutMs: number,
  timeoutMessage = "Operation timed out"
): Promise<T> {
  return Promise.race([
    operation(),
    new Promise<T>((_, reject) =>
      setTimeout(() => reject(new AppError(timeoutMessage, ErrorType.NETWORK)), timeoutMs)
    ),
  ]);
}

/**
 * Handle and log error consistently.
 *
 * Converts any error (Error, string, unknown) to an AppError instance,
 * logs it with context information for debugging, and sends it to
 * Sentry and Posthog for monitoring.
 *
 * @param error - Error to handle (can be Error, string, or unknown)
 * @param context - Optional context string for error logging (e.g., function name)
 * @returns AppError instance for further handling
 *
 * @example
 * ```ts
 * try {
 *   await riskyOperation();
 * } catch (error) {
 *   const appError = handleError(error, "riskyOperation");
 *   showToast(getUserFriendlyMessage(appError), "error");
 * }
 * ```
 */
export function handleError(error: unknown, context?: string): AppError {
  let appError: AppError;

  if (error instanceof AppError) {
    appError = error;
  } else if (error instanceof Error) {
    appError = new AppError(error.message, ErrorType.UNKNOWN, {
      originalError: error.name,
      stack: error.stack,
    });
  } else {
    appError = new AppError(String(error), ErrorType.UNKNOWN);
  }

  // Log error with context
  console.error(`[Error${context ? `: ${context}` : ""}]`, {
    message: appError.message,
    type: appError.type,
    context: appError.context,
  });

  // Send error to Sentry and Posthog for monitoring
  try {
    // Dynamic import to avoid bundling issues if analytics packages aren't available
    import("@sruja/shared")
      .then((shared) => {
        if (shared.trackError) {
          shared.trackError(appError, {
            component: context || "unknown",
            action: "error",
            errorType: appError.type,
            severity: "error",
            ...appError.context,
          });
        }
      })
      .catch(() => {
        // Silently fail if analytics package isn't available
      });
  } catch {
    // Silently fail if import fails
  }

  return appError;
}

/**
 * Check if error is retryable.
 *
 * Determines if an error should be retried based on error type.
 * Network errors (5xx) are retryable, validation errors (4xx) are not.
 *
 * @param error - Error to check
 * @returns True if error is retryable, false otherwise
 *
 * @example
 * ```ts
 * if (isRetryableError(error)) {
 *   await withRetry(() => retryOperation());
 * }
 * ```
 */
export function isRetryableError(error: unknown): boolean {
  if (error instanceof NetworkError) {
    // Retry on 5xx errors, not on 4xx
    return error.statusCode !== undefined && error.statusCode >= 500;
  }
  if (error instanceof AppError) {
    return error.type === ErrorType.NETWORK;
  }
  return false;
}

/**
 * Extract user-friendly error message.
 *
 * Converts any error to a user-friendly message string suitable for
 * displaying to end users. Handles AppError types with specific messages
 * and falls back to error message or generic message.
 *
 * @param error - Error to extract message from
 * @returns User-friendly error message string
 *
 * @example
 * ```ts
 * try {
 *   await operation();
 * } catch (error) {
 *   const message = getUserFriendlyMessage(error);
 *   showToast(message, "error");
 * }
 * ```
 */
export function getUserFriendlyMessage(error: unknown): string {
  if (error instanceof AppError) {
    switch (error.type) {
      case ErrorType.NETWORK:
        return "Network error. Please check your connection and try again.";
      case ErrorType.VALIDATION:
        return error.message || "Invalid input. Please check your data.";
      case ErrorType.PERMISSION:
        return "You don't have permission to perform this action.";
      case ErrorType.NOT_FOUND:
        return "The requested resource was not found.";
      default:
        return error.message || "An unexpected error occurred.";
    }
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "An unexpected error occurred.";
}
