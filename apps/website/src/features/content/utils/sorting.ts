// apps/website/src/features/content/utils/sorting.ts
import type { CollectionEntry } from 'astro:content';

/**
 * Sort entries by publication date (newest first)
 */
export function sortByDate<T extends CollectionEntry<any>>(entries: T[]): T[] {
  return entries.sort((a: T, b: T) => {
    const dateA = a.data.pubDate 
      ? (typeof a.data.pubDate === 'string' ? new Date(a.data.pubDate) : a.data.pubDate)
      : new Date(0);
    const dateB = b.data.pubDate 
      ? (typeof b.data.pubDate === 'string' ? new Date(b.data.pubDate) : b.data.pubDate)
      : new Date(0);
    return dateB.getTime() - dateA.getTime();
  });
}

/**
 * Sort entries by weight property (lower weight first)
 */
export function sortByWeight<T extends CollectionEntry<any>>(entries: T[]): T[] {
  return entries.sort((a: T, b: T) => {
    const weightA = (a.data as any).weight ?? 999;
    const weightB = (b.data as any).weight ?? 999;
    return weightA - weightB;
  });
}

















