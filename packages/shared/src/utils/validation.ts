// packages/shared/src/utils/validation.ts
// Validation utilities for architecture types with enhanced error handling

import { ValidationError } from './errors';
import type { Result } from './result';
import { err, ok } from './result';
import { PERCENTAGE } from './constants';

/**
 * Regular expression for valid element ID format.
 * 
 * @internal
 * @remarks
 * Element IDs must contain only alphanumeric characters, colons, underscores, and hyphens.
 */
const ELEMENT_ID_REGEX = /^[a-zA-Z0-9:_-]+$/;

/**
 * Validates that a value is a non-empty string.
 * 
 * @public
 * @param value - The value to validate
 * @returns true if the value is a non-empty string
 * 
 * @example
 * ```typescript
 * if (isNonEmptyString(value)) {
 *   // value is guaranteed to be a non-empty string
 * }
 * ```
 */
export function isNonEmptyString(value: unknown): value is string {
  return typeof value === 'string' && value.length > 0;
}

/**
 * Validates that a value is a non-empty string, returning a Result.
 * 
 * @public
 * @param value - The value to validate
 * @param fieldName - Optional field name for error message
 * @returns Result with string value or ValidationError
 * 
 * @example
 * ```typescript
 * const result = validateNonEmptyString(value, 'name');
 * if (result.ok) {
 *   console.log(result.value);
 * }
 * ```
 */
export function validateNonEmptyString(
  value: unknown,
  fieldName = 'value'
): Result<string, ValidationError> {
  if (typeof value === 'string' && value.length > 0) {
    return ok(value);
  }
  return err(
    new ValidationError(
      `${fieldName} must be a non-empty string`,
      { field: fieldName, value }
    )
  );
}

/**
 * Validates that a value is a valid element ID format.
 * 
 * @public
 * @param id - The ID to validate
 * @returns true if the ID is valid
 * 
 * @remarks
 * Element IDs must be non-empty strings containing only:
 * - Alphanumeric characters
 * - Colons (:)
 * - Underscores (_)
 * - Hyphens (-)
 * 
 * @example
 * ```typescript
 * if (isValidElementId(id)) {
 *   // id is a valid element identifier
 * }
 * ```
 */
export function isValidElementId(id: unknown): id is string {
  return isNonEmptyString(id) && ELEMENT_ID_REGEX.test(id);
}

/**
 * Validates that a value is a valid element ID, returning a Result.
 * 
 * @public
 * @param id - The ID to validate
 * @param fieldName - Optional field name for error message
 * @returns Result with validated ID or ValidationError
 * 
 * @example
 * ```typescript
 * const result = validateElementId(id, 'elementId');
 * if (result.ok) {
 *   processElement(result.value);
 * }
 * ```
 */
export function validateElementId(
  id: unknown,
  fieldName = 'id'
): Result<string, ValidationError> {
  if (!isNonEmptyString(id)) {
    return err(
      new ValidationError(
        `${fieldName} must be a non-empty string`,
        { field: fieldName, value: id }
      )
    );
  }
  if (!ELEMENT_ID_REGEX.test(id)) {
    return err(
      new ValidationError(
        `${fieldName} contains invalid characters. Only alphanumeric, colon, underscore, and hyphen are allowed`,
        { field: fieldName, value: id }
      )
    );
  }
  return ok(id);
}

/**
 * Validates that a value is a valid percentage (0-100).
 * 
 * @public
 * @param value - The value to validate
 * @returns true if the value is a valid percentage
 * 
 * @example
 * ```typescript
 * if (isValidPercentage(value)) {
 *   // value is between 0 and 100
 * }
 * ```
 */
export function isValidPercentage(value: unknown): value is number {
  return (
    typeof value === 'number' &&
    !Number.isNaN(value) &&
    Number.isFinite(value) &&
    value >= PERCENTAGE.MIN &&
    value <= PERCENTAGE.MAX
  );
}

/**
 * Validates that a value is a valid percentage, returning a Result.
 * 
 * @public
 * @param value - The value to validate
 * @param fieldName - Optional field name for error message
 * @returns Result with validated percentage or ValidationError
 * 
 * @example
 * ```typescript
 * const result = validatePercentage(value, 'availability');
 * if (result.ok) {
 *   setAvailability(result.value);
 * }
 * ```
 */
export function validatePercentage(
  value: unknown,
  fieldName = 'percentage'
): Result<number, ValidationError> {
  if (typeof value !== 'number' || Number.isNaN(value)) {
    return err(
      new ValidationError(
        `${fieldName} must be a number`,
        { field: fieldName, value }
      )
    );
  }
  if (!Number.isFinite(value)) {
    return err(
      new ValidationError(
        `${fieldName} must be a finite number, got ${value === Infinity ? 'Infinity' : '-Infinity'}`,
        { field: fieldName, value }
      )
    );
  }
  if (value < PERCENTAGE.MIN || value > PERCENTAGE.MAX) {
    return err(
      new ValidationError(
        `${fieldName} must be between ${PERCENTAGE.MIN} and ${PERCENTAGE.MAX}, got ${value}`,
        { field: fieldName, value }
      )
    );
  }
  return ok(value);
}

/**
 * Validates that a value is a positive integer (greater than 0).
 * 
 * @public
 * @param value - The value to validate
 * @param fieldName - Optional field name for error message
 * @returns Result with validated positive integer or ValidationError
 * 
 * @remarks
 * A positive integer is strictly greater than 0. Zero and negative numbers are invalid.
 * 
 * @example
 * ```typescript
 * const result = validatePositiveInteger(5, 'count');
 * if (result.ok) {
 *   console.log(result.value); // 5
 * }
 * ```
 */
export function validatePositiveInteger(
  value: unknown,
  fieldName = 'value'
): Result<number, ValidationError> {
  if (typeof value !== 'number' || Number.isNaN(value)) {
    return err(
      new ValidationError(
        `${fieldName} must be a number`,
        { field: fieldName, value }
      )
    );
  }
  if (!Number.isFinite(value)) {
    return err(
      new ValidationError(
        `${fieldName} must be a finite number, got ${value === Infinity ? 'Infinity' : '-Infinity'}`,
        { field: fieldName, value }
      )
    );
  }
  if (!Number.isInteger(value)) {
    return err(
      new ValidationError(
        `${fieldName} must be an integer, got ${value}`,
        { field: fieldName, value }
      )
    );
  }
  if (value <= 0) {
    return err(
      new ValidationError(
        `${fieldName} must be a positive integer (greater than 0), got ${value}`,
        { field: fieldName, value }
      )
    );
  }
  return ok(value);
}

