import { SrujaElement } from "./wasm";

/**
 * Finds a single element by ID with priority for exact matches.
 * Returns undefined if ambiguous (multiple suffix matches) or not found.
 */
export function findElementById(elements: SrujaElement[], id: string): SrujaElement | undefined {
  if (!id) return undefined;
  
  // 1. Exact match (highest priority)
  const exact = elements.find(e => e.id === id);
  if (exact) return exact;

  // 2. Suffix match
  const matches = elements.filter(e => e.id.endsWith(`.${id}`));
  if (matches.length === 1) {
    return matches[0];
  }
  
  return undefined;
}

/**
 * Finds all elements that match an ID (exact or suffix).
 * Useful for providers that can return multiple results (Definition, References).
 */
export function findAllElementsById(elements: SrujaElement[], id: string): SrujaElement[] {
  if (!id) return [];
  
  // 1. Exact match
  const exact = elements.find(e => e.id === id);
  if (exact) return [exact];

  // 2. Suffix matches
  return elements.filter(e => e.id.endsWith(`.${id}`));
}
