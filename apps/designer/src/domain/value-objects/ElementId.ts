/**
 * ElementId Value Object
 *
 * A type-safe, validated identifier for architecture elements.
 * Ensures all element IDs follow consistent formatting rules.
 *
 * @module domain/value-objects
 */

import { ValidationError, ok, err, type Result } from '@sruja/shared/utils';

/**
 * Branded type for element IDs to prevent accidental misuse
 *
 * @remarks
 * This type cannot be constructed directly - use ElementId.create() or ElementId.unsafe()
 * to ensure validation.
 */
export type ElementId = string & { readonly __brand: unique symbol };

/**
 * Regular expression for valid element IDs
 *
 * Rules:
 * - Must start with a letter (a-z, A-Z)
 * - Can contain letters, numbers, hyphens, and underscores
 * - Must be between 1 and 100 characters
 */
const ELEMENT_ID_REGEX = /^[a-zA-Z][a-zA-Z0-9_-]{0,99}$/;

/**
 * ElementId value object
 *
 * Provides type-safe, validated element identifiers throughout the system.
 * All element IDs should pass through this class to ensure consistency.
 *
 * @example
 * ```typescript
 * // Create a new ElementId (validated)
 * const id1 = ElementId.create('user-service');
 *
 * // Or with error handling
 * const idResult = ElementId.create('invalid$id!');
 * if (!idResult.ok) {
 *   console.error(idResult.error);
 * }
 *
 * // Convert to string
 * console.log(id1.value); // 'user-service'
 * console.log(`${id1}`);  // 'user-service'
 *
 * // Parse from arbitrary string
 * const parsed = ElementId.parse('external-component');
 *
 * // Generate unique ID
 * const generated = ElementId.generate('component');
 * ```
 */
export class ElementId {
  /**
   * Creates a new validated ElementId
   *
   * @param value - The ID string to validate
   * @returns Result containing the ElementId or a ValidationError
   */
  static create(value: string): Result<ElementId, ValidationError> {
    // Check for empty or whitespace-only strings
    const trimmed = value.trim();
    if (trimmed.length === 0) {
      return err(new ValidationError('Element ID cannot be empty'));
    }

    // Check length (after trimming whitespace)
    if (trimmed.length > 100) {
      return err(new ValidationError('Element ID cannot exceed 100 characters'));
    }

    // Validate format
    if (!ELEMENT_ID_REGEX.test(trimmed)) {
      return err(
        new ValidationError(
          'Element ID must start with a letter and contain only letters, numbers, hyphens, and underscores'
        )
      );
    }

    // Check for reserved prefixes (future-proofing)
    const reservedPrefixes = ['system-', 'internal-', 'temp-'];
    for (const prefix of reservedPrefixes) {
      if (trimmed.startsWith(prefix)) {
        return err(new ValidationError(`Element ID cannot start with reserved prefix '${prefix}'`));
      }
    }

    return ok(trimmed as ElementId);
  }

  /**
   * Creates an ElementId without validation (use with caution!)
   *
   * @remarks
   * This method bypasses validation and should only be used when:
   * - You have already validated the ID elsewhere
   * - You're parsing trusted data (e.g., from a validated database)
   * - You're migrating from a system with existing IDs
   *
   * @param value - The ID string to convert
   * @returns An ElementId instance
   */
  static unsafe(value: string): ElementId {
    return value as ElementId;
  }

  /**
   * Parses a string into an ElementId, returning undefined if invalid
   *
   * @param value - The string to parse
   * @returns ElementId or undefined if invalid
   */
  static parse(value: string): ElementId | undefined {
    const result = this.create(value);
    return result.ok ? result.value : undefined;
  }

  /**
   * Generates a unique ElementId with a prefix
   *
   * @param prefix - The prefix for the ID (e.g., 'component', 'system')
   * @param suffix - Optional suffix (uses timestamp if not provided)
   * @returns A new ElementId
   */
  static generate(prefix: string, suffix?: string): ElementId {
    const safePrefix = prefix
      .toLowerCase()
      .replace(/[^a-z0-9]/g, '-')
      .replace(/-+/g, '-')
      .replace(/^-|-$|' + '/g, '');

    const uniqueSuffix =
      suffix ??
      `${Date.now().toString(36)}-${Math.random().toString(36).substring(2, 7)}`;

    return ElementId.unsafe(`${safePrefix}-${uniqueSuffix}`);
  }

  /**
   * Generates a ElementId based on a name
   *
   * @param name - The name to convert to an ID
   * @returns A new ElementId
   */
  static fromName(name: string): Result<ElementId, ValidationError> {
    // Convert name to a valid ID format
    const id = name
      .toLowerCase()
      .replace(/[^a-z0-9\s-]/g, '') // Remove special chars
      .trim()
      .replace(/\s+/g, '-') // Convert spaces to hyphens
      .replace(/-+/g, '-') // Collapse multiple hyphens
      .replace(/^-|-$|' + '/g, ''); // Remove leading/trailing hyphens

    return this.create(id);
  }

  /**
   * Checks if a string is a valid ElementId
   *
   * @param value - The string to check
   * @returns True if valid
   */
  static isValid(value: string): boolean {
    return ELEMENT_ID_REGEX.test(value.trim());
  }

  /**
   * The underlying string value
   *
   * @remarks
   * Access via .value to convert to string when needed
   */
  readonly value: ElementId;

  /**
   * Private constructor - use static factory methods
   *
   * @private
   */
  private constructor(value: ElementId) {
    this.value = value;
    Object.freeze(this); // Make immutable
  }

  /**
   * Checks if this ID starts with a prefix
   *
   * @param prefix - The prefix to check
   * @returns True if starts with prefix
   */
  startsWith(prefix: string): boolean {
    return this.value.startsWith(prefix);
  }

  /**
   * Checks if this ID ends with a suffix
   *
   * @param suffix - The suffix to check
   * @returns True if ends with suffix
   */
  endsWith(suffix: string): boolean {
    return this.value.endsWith(suffix);
  }

  /**
   * Checks if this ID equals another
   *
   * @param other - The other ElementId to compare
   * @returns True if equal
   */
  equals(other: ElementId | string): boolean {
    const otherValue = typeof other === 'string' ? other : other.value;
    return this.value === otherValue;
  }

  /**
   * Gets the prefix of the ID (before the last hyphen or underscore)
   *
   * @returns The prefix or empty string if none
   */
  getPrefix(): string {
    const lastSeparator = Math.max(
      this.value.lastIndexOf('-'),
      this.value.lastIndexOf('_')
    );
    return lastSeparator > 0 ? this.value.substring(0, lastSeparator) : '';
  }

  /**
   * Gets the suffix of the ID (after the last hyphen or underscore)
   *
   * @returns The suffix or the full ID if no separator
   */
  getSuffix(): string {
    const lastSeparator = Math.max(
      this.value.lastIndexOf('-'),
      this.value.lastIndexOf('_')
    );
    return lastSeparator >= 0 ? this.value.substring(lastSeparator + 1) : this.value;
  }

  /**
   * Converts to plain string
   *
   * @returns The string value
   */
  toString(): string {
    return this.value;
  }

  /**
   * String conversion for template literals
   *
   * @returns The string value
   */
  [Symbol.toStringTag](): string {
    return 'ElementId';
  }
}

/**
 * Type guard to check if a value is an ElementId
 *
 * @param value - The value to check
 * @returns True if the value is an ElementId
 */
export function isElementId(value: unknown): value is ElementId {
  return typeof value === 'string' && ELEMENT_ID_REGEX.test(value);
}

/**
 * Utility function to assert a value is an ElementId
 *
 * @throws ValidationError if the value is not a valid ElementId
 * @param value - The value to assert
 * @returns The value as ElementId
 */
export function assertElementId(value: unknown): ElementId {
  if (!isElementId(value)) {
    throw new ValidationError(`Invalid ElementId: ${String(value)}`);
  }
  return value as ElementId;
}
