// apps/designer/src/components/shared/forms/utils.ts
// Shared utility functions for form components

import type { SrujaModelDump, ElementDump } from "@sruja/shared";

/**
 * Convert text to URL-friendly slug
 */
export function slugify(text: string): string {
  return text
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9_-]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

/**
 * Collect all node IDs from architecture data
 */
/**
 * Collects all node IDs from an architecture JSON structure.
 *
 * Iterates through the elements map to collect IDs.
 *
 * @param data - Architecture JSON data (can be null)
 * @returns Set of all node IDs found in the architecture
 */
export function collectIds(data: SrujaModelDump | null): Set<string> {
  const ids = new Set<string>();
  if (!data?.elements) return ids;

  Object.values(data.elements).forEach((el: ElementDump) => {
    ids.add(el.id);
  });

  // Also collect IDs from relations if they have them (RelationDump has id)
  data.relations?.forEach((rel) => {
    if (rel.id) ids.add(rel.id);
  });

  return ids;
}

/**
 * Generate a unique ID based on a base string
 */
/**
 * Generates a unique ID for a new node based on a base name.
 *
 * Creates a slug from the base name and ensures uniqueness by appending
 * a number suffix if the ID already exists in the architecture.
 *
 * @param base - Base name to generate ID from
 * @param data - Architecture JSON data to check for existing IDs
 * @param type - Type of node (affects ID format, default: "system")
 * @returns Unique ID string
 */
export function generateUniqueId(
  base: string,
  data: SrujaModelDump | null,
  type: string = "system"
): string {
  const ids = collectIds(data);
  const defaultId = type === "person" ? "person" : type;
  let candidate = slugify(base) || defaultId;

  // If we want to be safe and avoid collisions with existing global IDs (though SrujaModelDump IDs are global)
  // we just check if candidate is in ids.

  let i = 1;
  const originalCandidate = candidate;
  while (ids.has(candidate)) {
    candidate = `${originalCandidate}-${i++}`;
  }
  return candidate;
}

/**
 * Extract description text from an ElementDump.
 *
 * Handles both string descriptions and object descriptions with a `txt` property.
 * This is a common pattern in the codebase where descriptions can be either:
 * - A plain string: "Description text"
 * - An object: { txt: "Description text" }
 *
 * @param element - ElementDump to extract description from (optional)
 * @returns Extracted description string, or empty string if not found
 *
 * @example
 * ```tsx
 * const description = extractDescription(element);
 * // Returns "Description text" or ""
 * ```
 */
export function extractDescription(element?: ElementDump): string {
  if (!element?.description) return "";

  if (typeof element.description === "string") {
    return element.description;
  }

  // Handle object format: { txt: "..." }
  const descriptionObj = element.description as unknown as { txt?: string };
  return descriptionObj?.txt || "";
}
