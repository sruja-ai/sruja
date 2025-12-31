// apps/website/src/features/content/utils/paths.ts
import type { CollectionEntry } from "astro:content";
import { sortByDate, sortByWeight, type AnyCollectionEntry } from "./sorting";

interface NavigationItem {
  title: string;
  href: string;
}

interface StaticPathProps<T extends CollectionEntry<string>> {
  entry: T;
  previous: NavigationItem | null;
  next: NavigationItem | null;
}

type SortStrategy = "date" | "weight";

/**
 * Generate static paths for content collections with navigation
 */
export function generateContentPaths<T extends CollectionEntry<string>>(
  entries: T[],
  basePath: string,
  sortStrategy: SortStrategy = "weight",
  filterFn?: (entry: T) => boolean
): Array<{
  params: { slug: string };
  props: StaticPathProps<T>;
}> {
  let filtered = entries;

  if (filterFn) {
    filtered = entries.filter(filterFn);
  }

  const sorted = (
    sortStrategy === "date"
      ? sortByDate(filtered as unknown as AnyCollectionEntry[])
      : sortByWeight(filtered as unknown as Array<{ data: { weight?: number } }>)
  ) as T[];

  return sorted.map((entry: T, index: number) => {
    const prevEntry: T | undefined = index > 0 ? sorted[index - 1] : undefined;
    const nextEntry: T | undefined = index < sorted.length - 1 ? sorted[index + 1] : undefined;

    const previous = prevEntry
      ? {
          title: (prevEntry as CollectionEntry<string>).data.title,
          href: `${basePath}/${(prevEntry as CollectionEntry<string>).slug}`,
        }
      : null;
    const next = nextEntry
      ? {
          title: (nextEntry as CollectionEntry<string>).data.title,
          href: `${basePath}/${(nextEntry as CollectionEntry<string>).slug}`,
        }
      : null;

    const entryAsCollection = entry as CollectionEntry<string>;
    return {
      params: { slug: entryAsCollection.slug },
      props: {
        entry,
        previous,
        next,
      },
    };
  });
}
