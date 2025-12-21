// packages/shared/src/utils/pathValidation.ts
// File path validation utilities for security

import { ValidationError } from './errors';
import type { Result } from './result';
import { err, ok } from './result';

/**
 * Validates that a file path is safe and does not contain path traversal attempts.
 * 
 * @public
 * @param path - File path to validate
 * @param allowAbsolute - Whether to allow absolute paths (default: false)
 * @returns Result with validated path or ValidationError
 * 
 * @remarks
 * This function prevents path traversal attacks by:
 * - Rejecting paths containing '..'
 * - Rejecting paths starting with '/' (if allowAbsolute is false)
 * - Rejecting paths containing null bytes
 * - Rejecting paths with control characters
 * 
 * @example
 * ```typescript
 * const result = validateFilePath('../etc/passwd');
 * if (!result.ok) {
 *   console.error('Path traversal detected!');
 * }
 * ```
 */
export function validateFilePath(
  path: string,
  allowAbsolute = false
): Result<string, ValidationError> {
  if (typeof path !== 'string') {
    return err(
      new ValidationError(
        'Path must be a string',
        { field: 'path', value: path }
      )
    );
  }

  // Reject empty paths
  if (path.length === 0) {
    return err(
      new ValidationError(
        'Path cannot be empty',
        { field: 'path', value: path }
      )
    );
  }

  // Reject paths containing null bytes (potential injection)
  if (path.includes('\0')) {
    return err(
      new ValidationError(
        'Path cannot contain null bytes',
        { field: 'path', value: path }
      )
    );
  }

  // Reject paths containing control characters
  if (/[\x00-\x1F\x7F]/.test(path)) {
    return err(
      new ValidationError(
        'Path cannot contain control characters',
        { field: 'path', value: path }
      )
    );
  }

  // Reject path traversal attempts
  if (path.includes('..')) {
    return err(
      new ValidationError(
        'Path traversal detected: path cannot contain ".."',
        { field: 'path', value: path }
      )
    );
  }

  // Reject absolute paths if not allowed
  if (!allowAbsolute && (path.startsWith('/') || /^[A-Za-z]:/.test(path))) {
    return err(
      new ValidationError(
        'Absolute paths are not allowed',
        { field: 'path', value: path }
      )
    );
  }

  // Reject paths that are too long (prevent DoS)
  const MAX_PATH_LENGTH = 4096; // Common OS limit
  if (path.length > MAX_PATH_LENGTH) {
    return err(
      new ValidationError(
        `Path exceeds maximum length of ${MAX_PATH_LENGTH} characters`,
        { field: 'path', value: path }
      )
    );
  }

  return ok(path);
}

/**
 * Sanitizes a file path by removing dangerous characters and normalizing separators.
 * 
 * @public
 * @param path - File path to sanitize
 * @returns Sanitized path
 * 
 * @remarks
 * This function:
 * - Normalizes path separators to '/'
 * - Removes leading/trailing whitespace
 * - Removes duplicate separators
 * - Does NOT prevent path traversal - use validateFilePath for that
 * 
 * @example
 * ```typescript
 * const sanitized = sanitizeFilePath('  path/to//file  ');
 * // Returns: 'path/to/file'
 * ```
 */
export function sanitizeFilePath(path: string): string {
  if (typeof path !== 'string') {
    return '';
  }

  return path
    .trim()
    .replace(/[\\/]+/g, '/') // Normalize separators
    .replace(/^\/+|\/+$/g, ''); // Remove leading/trailing slashes
}

/**
 * Type guard to check if a path is safe.
 * 
 * @public
 * @param path - Path to check
 * @returns true if path appears safe (basic check only)
 * 
 * @remarks
 * This is a quick check. For full validation, use validateFilePath.
 */
export function isSafePath(path: unknown): path is string {
  if (typeof path !== 'string') {
    return false;
  }
  return !path.includes('..') && !path.includes('\0');
}

