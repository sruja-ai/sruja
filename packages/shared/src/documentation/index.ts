// packages/shared/src/documentation/index.ts
// Shared documentation utilities - uses glossary as single source of truth

export * from './glossary';

// Re-export DocSection and loader functions
export type { DocSection } from './loader';
export { loadDocSection, getAllDocSections } from './loader';

import type { DocSection } from './loader';
import { getConceptById, getAllConcepts, getConceptUrl } from './glossary';

/**
 * Get documentation URL for a node type.
 * 
 * @public
 * @param nodeType - Node type identifier
 * @returns Documentation URL or default concepts URL
 * 
 * @remarks
 * Alias for getConceptUrl() for backward compatibility.
 */
export function getDocUrl(nodeType: string): string {
  return getConceptUrl(nodeType);
}

/**
 * Get all documentation section IDs.
 * 
 * @public
 * @returns Array of all concept IDs
 */
export function getAllDocSectionIds(): ReadonlyArray<string> {
  return getAllConcepts().map((concept) => concept.id);
}

/**
 * Get documentation section for a node type.
 * 
 * @public
 * @param nodeType - Node type identifier
 * @returns DocSection or null if not found
 * 
 * @remarks
 * Converts glossary entry to DocSection format for backward compatibility.
 */
export function getDocSection(nodeType: string): DocSection | null {
  const concept = getConceptById(nodeType);
  if (!concept) return null;

  return {
    id: concept.id,
    title: concept.title,
    content: concept.description,
    url: concept.url,
    summary: concept.summary,
  };
}
