// apps/website/src/features/content/utils/sorting.ts
import type { CollectionEntry } from "astro:content";

type AnyCollectionEntry = CollectionEntry<"blog" | "docs" | "courses" | "tutorials">;

/**
 * Sort entries by publication date (newest first)
 */
export function sortByDate<T extends AnyCollectionEntry>(entries: T[]): T[] {
  return entries.sort((a, b) => {
    const entryA = a as AnyCollectionEntry;
    const entryB = b as AnyCollectionEntry;
    const dateA = entryA.data.pubDate
      ? typeof entryA.data.pubDate === "string"
        ? new Date(entryA.data.pubDate)
        : entryA.data.pubDate
      : new Date(0);
    const dateB = entryB.data.pubDate
      ? typeof entryB.data.pubDate === "string"
        ? new Date(entryB.data.pubDate)
        : entryB.data.pubDate
      : new Date(0);
    return dateB.getTime() - dateA.getTime();
  });
}

/**
 * Sort entries by weight property (lower weight first)
 */
export function sortByWeight<T>(entries: T[]): T[] {
  return [...entries].sort((a: any, b: any) => {
    const weightA = a.data?.weight ?? 999;
    const weightB = b.data?.weight ?? 999;
    return weightA - weightB;
  });
}
