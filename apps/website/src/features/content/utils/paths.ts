// apps/website/src/features/content/utils/paths.ts
import type { CollectionEntry } from 'astro:content';
import { sortByDate, sortByWeight } from './sorting';

interface NavigationItem {
  title: string;
  href: string;
}

interface StaticPathProps<T extends CollectionEntry<any>> {
  entry: T;
  previous: NavigationItem | null;
  next: NavigationItem | null;
}

type SortStrategy = 'date' | 'weight';

/**
 * Generate static paths for content collections with navigation
 */
export function generateContentPaths<T extends CollectionEntry<any>>(
  entries: T[],
  basePath: string,
  sortStrategy: SortStrategy = 'weight',
  filterFn?: (entry: T) => boolean
): Array<{
  params: { slug: string };
  props: StaticPathProps<T>;
}> {
  let filtered = entries;
  
  if (filterFn) {
    filtered = entries.filter(filterFn);
  }
  
  const sorted = sortStrategy === 'date' 
    ? sortByDate(filtered)
    : sortByWeight(filtered);
  
  return sorted.map((entry: T, index: number) => {
    const prevEntry: T | undefined = index > 0 ? sorted[index - 1] : undefined;
    const nextEntry: T | undefined = index < sorted.length - 1 ? sorted[index + 1] : undefined;
    
    const previous = prevEntry
      ? { title: (prevEntry as CollectionEntry<any>).data.title, href: `${basePath}/${(prevEntry as CollectionEntry<any>).slug}` }
      : null;
    const next = nextEntry
      ? { title: (nextEntry as CollectionEntry<any>).data.title, href: `${basePath}/${(nextEntry as CollectionEntry<any>).slug}` }
      : null;
    
    const entryAsCollection = entry as CollectionEntry<any>;
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

















