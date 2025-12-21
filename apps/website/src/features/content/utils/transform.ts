// apps/website/src/features/content/utils/transform.ts
import type { CollectionEntry } from "astro:content";
import { formatContentDate } from "./dates";

export interface ContentListItem {
  title: string;
  href: string;
  summary?: string;
  tags?: string[];
  date?: string;
  linkText?: string;
}

type AnyCollectionEntry = CollectionEntry<"blog" | "docs" | "courses" | "tutorials">;

/**
 * Transform a content entry to a ContentList item
 */
export function transformToContentListItem<T extends AnyCollectionEntry>(
  entry: T,
  basePath: string,
  options: {
    linkText?: string;
    includeDate?: boolean;
  } = {}
): ContentListItem {
  const { linkText, includeDate = false } = options;
  const entryTyped = entry as AnyCollectionEntry;

  const item: ContentListItem = {
    title: entryTyped.data.title,
    href: `${basePath}/${entryTyped.slug}`,
    summary: entryTyped.data.summary,
    tags: entryTyped.data.tags,
  };

  if (linkText) {
    item.linkText = linkText;
  }

  if (includeDate && entryTyped.data.pubDate) {
    const date = formatContentDate(entryTyped.data.pubDate);
    if (date) {
      item.date = date;
    }
  }

  return item;
}

/**
 * Transform multiple content entries to ContentList items
 */
export function transformToContentListItems<T extends AnyCollectionEntry>(
  entries: T[],
  basePath: string,
  options: {
    linkText?: string;
    includeDate?: boolean;
  } = {}
): ContentListItem[] {
  return entries.map((entry) => transformToContentListItem(entry, basePath, options));
}
