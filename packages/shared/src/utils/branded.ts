// packages/shared/src/utils/branded.ts
// Utilities for converting branded types to plain strings

/**
 * Convert a branded type (or any value) to a plain string.
 * 
 * @public
 * @param value - Value to convert (branded type, string, number, etc.)
 * @returns String representation
 * 
 * @remarks
 * Safely converts LikeC4 branded types (Fqn, RelationId, ViewId) to plain strings
 * for JSON serialization. Handles null/undefined gracefully.
 * 
 * @example
 * ```typescript
 * const fqn: Fqn = 'system:api' as Fqn;
 * const plain = toString(fqn); // Returns 'system:api'
 * ```
 */
export function toString(value: unknown): string {
  if (value === null || value === undefined) {
    return '';
  }
  return String(value);
}

/**
 * Convert a branded type to string, with optional fallback.
 * 
 * @public
 * @param value - Value to convert
 * @param fallback - Fallback if value is null/undefined (default: '')
 * @returns String representation or fallback
 * 
 * @example
 * ```typescript
 * const id = toStringOr('system:api', 'unknown'); // Returns 'system:api'
 * const empty = toStringOr(null, 'default'); // Returns 'default'
 * ```
 */
export function toStringOr(value: unknown, fallback: string = ''): string {
  if (value === null || value === undefined) {
    return fallback;
  }
  return String(value);
}

