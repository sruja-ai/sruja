// packages/shared/src/utils/richtext.ts
// RichText and MarkdownOrString utility functions

import type { MarkdownOrString } from "../types/core";

/**
 * Extract text content from MarkdownOrString.
 * Since Sruja uses plain strings for descriptions, this is a pass-through
 * or handles legacy object format if present (though types say string only).
 */
export function extractText(description: MarkdownOrString | null | undefined): string | null {
  if (typeof description === "string") {
    return description;
  }
  // Fallback for runtime safety if legacy objects exist
  if (description && typeof description === "object" && "txt" in description) {
    return (description as { txt: string }).txt ?? null;
  }
  return null;
}

/**
 * Check if a value is a RichText object.
 * @deprecated Sruja uses plain strings. Usage suggests migration needed.
 */
export function isRichText(value: unknown): value is { txt: string } {
  return (
    typeof value === "object" &&
    value !== null &&
    "txt" in value &&
    typeof (value as { txt: unknown }).txt === "string"
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
  fallback: string = ""
): string {
  const extracted = extractText(description);
  return extracted ?? fallback;
}
