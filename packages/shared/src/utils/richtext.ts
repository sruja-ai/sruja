// packages/shared/src/utils/richtext.ts
// RichText and MarkdownOrString utility functions

import type { MarkdownOrString } from '@likec4/core/types';

/**
 * Extract text content from MarkdownOrString.
 * 
 * @public
 * @param description - MarkdownOrString value (string, RichText object, null, or undefined)
 * @returns Extracted string or null if not available
 * 
 * @remarks
 * Handles LikeC4's MarkdownOrString type which can be:
 * - A plain string
 * - A RichText object with `txt` property
 * - null or undefined
 * 
 * @example
 * ```typescript
 * const text1 = extractText('Hello'); // Returns 'Hello'
 * const text2 = extractText({ txt: 'World' }); // Returns 'World'
 * const text3 = extractText(null); // Returns null
 * ```
 */
export function extractText(
  description: MarkdownOrString | null | undefined
): string | null {
  if (typeof description === 'string') {
    return description;
  }
  if (
    description &&
    typeof description === 'object' &&
    'txt' in description
  ) {
    return description.txt ?? null;
  }
  return null;
}

/**
 * Check if a value is a RichText object.
 * 
 * @public
 * @param value - Value to check
 * @returns true if value is a RichText object
 * 
 * @example
 * ```typescript
 * if (isRichText(desc)) {
 *   const text = desc.txt;
 * }
 * ```
 */
export function isRichText(
  value: unknown
): value is { txt: string } {
  return (
    typeof value === 'object' &&
    value !== null &&
    'txt' in value &&
    typeof (value as { txt: unknown }).txt === 'string'
  );
}

/**
 * Convert MarkdownOrString to string, with fallback.
 * 
 * @public
 * @param description - MarkdownOrString value
 * @param fallback - Fallback value if description is null/undefined (default: '')
 * @returns String representation
 * 
 * @example
 * ```typescript
 * const text = toStringOrFallback(desc, 'No description');
 * ```
 */
export function toStringOrFallback(
  description: MarkdownOrString | null | undefined,
  fallback: string = ''
): string {
  const extracted = extractText(description);
  return extracted ?? fallback;
}

