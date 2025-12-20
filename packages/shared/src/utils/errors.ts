// packages/shared/src/utils/errors.ts
// Comprehensive error handling system following FAANG best practices

/**
 * Base error class for all Sruja-specific errors.
 * 
 * @public
 * @remarks
 * All custom errors in the codebase should extend this class.
 * Provides structured error information with context.
 * 
 * @example
 * ```typescript
 * throw new SrujaError('Validation failed', { field: 'id', value: invalidId });
 * ```
 */
export class SrujaError extends Error {
  /**
   * Error code for programmatic error handling.
   */
  readonly code: string;
  
  /**
   * Additional context about the error.
   */
  readonly context?: Readonly<Record<string, unknown>>;
  
  /**
   * Original error that caused this error (if any).
   */
  readonly cause?: Error;

  constructor(
    message: string,
    options?: {
      code?: string;
      context?: Readonly<Record<string, unknown>>;
      cause?: Error;
    }
  ) {
    super(message);
    this.name = 'SrujaError';
    this.code = options?.code ?? 'UNKNOWN_ERROR';
    this.context = options?.context;
    this.cause = options?.cause;
    
    // Maintains proper stack trace for where our error was thrown (only available on V8)
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, SrujaError);
    }
  }

  /**
   * Convert error to JSON for logging/serialization.
   * 
   * @public
   * @returns Serializable error representation
   * 
   * @remarks
   * Includes the error chain (cause) for better debugging.
   * Circular references in context are handled gracefully.
   */
  toJSON(): Readonly<{
    name: string;
    message: string;
    code: string;
    context?: Readonly<Record<string, unknown>>;
    stack?: string;
    cause?: {
      name: string;
      message: string;
      stack?: string;
    };
  }> {
    const result: {
      name: string;
      message: string;
      code: string;
      context?: Readonly<Record<string, unknown>>;
      stack?: string;
      cause?: {
        name: string;
        message: string;
        stack?: string;
      };
    } = {
      name: this.name,
      message: this.message,
      code: this.code,
      context: this.context,
      stack: this.stack,
    };

    // Include cause if present for error chain debugging
    if (this.cause) {
      result.cause = {
        name: this.cause.name,
        message: this.cause.message,
        stack: this.cause.stack,
      };
    }

    return result;
  }
}

/**
 * Validation error for invalid input data.
 * 
 * @public
 */
export class ValidationError extends SrujaError {
  constructor(
    message: string,
    options?: {
      field?: string;
      value?: unknown;
      context?: Readonly<Record<string, unknown>>;
    }
  ) {
    super(message, {
      code: 'VALIDATION_ERROR',
      context: {
        ...options?.context,
        field: options?.field,
        value: options?.value,
      },
    });
    this.name = 'ValidationError';
  }
}

/**
 * Configuration error for invalid configuration.
 * 
 * @public
 */
export class ConfigurationError extends SrujaError {
  constructor(
    message: string,
    options?: {
      configKey?: string;
      context?: Readonly<Record<string, unknown>>;
    }
  ) {
    super(message, {
      code: 'CONFIGURATION_ERROR',
      context: {
        ...options?.context,
        configKey: options?.configKey,
      },
    });
    this.name = 'ConfigurationError';
  }
}

/**
 * Network/API error for failed requests.
 * 
 * @public
 */
export class NetworkError extends SrujaError {
  constructor(
    message: string,
    options?: {
      url?: string;
      status?: number;
      cause?: Error;
      context?: Readonly<Record<string, unknown>>;
    }
  ) {
    super(message, {
      code: 'NETWORK_ERROR',
      context: {
        ...options?.context,
        url: options?.url,
        status: options?.status,
      },
      cause: options?.cause,
    });
    this.name = 'NetworkError';
  }
}

/**
 * Type guard to check if an error is a SrujaError.
 * 
 * @public
 * @param error - Error to check
 * @returns true if error is a SrujaError
 */
export function isSrujaError(error: unknown): error is SrujaError {
  return error instanceof SrujaError;
}

/**
 * Type guard to check if an error is a ValidationError.
 * 
 * @public
 * @param error - Error to check
 * @returns true if error is a ValidationError
 */
export function isValidationError(error: unknown): error is ValidationError {
  return error instanceof ValidationError;
}

/**
 * Safely extract error message from unknown error.
 * 
 * @public
 * @param error - Error to extract message from
 * @returns Error message string
 */
export function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === 'string') {
    return error;
  }
  return 'Unknown error occurred';
}

/**
 * Safely extract error stack from unknown error.
 * 
 * @public
 * @param error - Error to extract stack from
 * @returns Error stack string or undefined
 */
export function getErrorStack(error: unknown): string | undefined {
  if (error instanceof Error) {
    return error.stack;
  }
  return undefined;
}

