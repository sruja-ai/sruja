// packages/shared/src/utils/index.ts
// Shared utilities for Sruja frontend

/**
 * @packageDocumentation
 * 
 * # Utils Module
 * 
 * Utility functions for common operations across Sruja applications.
 * 
 * ## Available Utilities
 * 
 * - **logger**: Structured logging with analytics integration
 * - **markdown**: Markdown processing and manipulation utilities
 * - **cssVars**: CSS custom property access utilities
 * 
 * ## Usage
 * 
 * ```typescript
 * import { logger } from '@sruja/shared/utils';
 * import { getCssVar, Colors } from '@sruja/shared/utils/cssVars';
 * ```
 * 
 * @module utils
 */

// ============================================================================
// Logging
// ============================================================================
/**
 * Structured logging utility with analytics integration.
 */
export * from './logger';

// ============================================================================
// Markdown
// ============================================================================
/**
 * Markdown processing and manipulation utilities.
 */
export * from './markdown';

// ============================================================================
// CSS Variables
// ============================================================================
/**
 * CSS custom property access utilities.
 * 
 * @remarks
 * Import directly for better tree-shaking:
 * ```typescript
 * import { getCssVar, Colors } from '@sruja/shared/utils/cssVars';
 * ```
 */
export { getCssVar, Colors } from './cssVars';

// ============================================================================
// Environment Detection
// ============================================================================
/**
 * Environment detection utilities.
 */
export { isBrowser, isNode, isSSR, getBaseUrl } from './env';

// ============================================================================
// RichText Utilities
// ============================================================================
/**
 * RichText and MarkdownOrString utility functions.
 */
export { extractText, isRichText, toStringOrFallback } from './richtext';

// ============================================================================
// Branded Type Utilities
// ============================================================================
/**
 * Utilities for converting branded types to plain strings.
 */
export { toString, toStringOr } from './branded';

// ============================================================================
// Validation Utilities
// ============================================================================
/**
 * Validation utilities for architecture types.
 */
export {
  isNonEmptyString,
  isValidElementId,
  isValidPercentage,
  validateNonEmptyString,
  validateElementId,
  validatePercentage,
  validatePositiveInteger,
} from './validation';

// ============================================================================
// Error Handling
// ============================================================================
/**
 * Comprehensive error handling system with custom error types.
 */
export {
  SrujaError,
  ValidationError,
  ConfigurationError,
  NetworkError,
  isSrujaError,
  isValidationError,
  getErrorMessage,
  getErrorStack,
} from './errors';

// ============================================================================
// Result Type
// ============================================================================
/**
 * Result/Either pattern for functional error handling.
 */
export {
  ok,
  err,
  tryCatch,
  tryCatchAsync,
  map,
  mapErr,
  andThen,
  unwrap,
  unwrapOr,
  unwrapOrElse,
  type Result,
} from './result';

// ============================================================================
// Performance Utilities
// ============================================================================
/**
 * Memoization utilities for performance optimization.
 */
export { memoize, weakMemoize, type MemoizeOptions } from './memoize';

// ============================================================================
// Constants
// ============================================================================
/**
 * Application-wide constants to avoid magic numbers and strings.
 */
export {
  DEFAULT_PROJECT_ID,
  DEFAULT_PROJECT_NAME,
  PERCENTAGE,
  READING_TIME,
  RETRY,
  STORAGE_KEYS,
  ERROR_STACK_TRUNCATE_LENGTH,
  CACHE,
} from './constants';

// ============================================================================
// Path Validation
// ============================================================================
/**
 * File path validation utilities for security.
 */
export {
  validateFilePath,
  sanitizeFilePath,
  isSafePath,
} from './pathValidation';
