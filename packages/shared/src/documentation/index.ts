// packages/shared/src/documentation/index.ts
// Shared documentation utilities - uses glossary as single source of truth

export * from './glossary';

// Re-export for backward compatibility
export interface DocSection {
  id: string;
  title: string;
  content: string;
  url: string;
  summary?: string;
}

// Helper to convert glossary entry to DocSection format
import { getConceptById, getAllConcepts } from './glossary';

export function getDocUrl(nodeType: string): string {
  const concept = getConceptById(nodeType);
  return concept?.url || '/docs/concepts';
}

export function getAllDocSectionIds(): string[] {
  return getAllConcepts().map(concept => concept.id);
}

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
