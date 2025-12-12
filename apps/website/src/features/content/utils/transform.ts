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

/**
 * Transform a content entry to a ContentList item
 */
export function transformToContentListItem<T extends CollectionEntry<any>>(
  entry: T,
  basePath: string,
  options: {
    linkText?: string;
    includeDate?: boolean;
  } = {}
): ContentListItem {
  const { linkText, includeDate = false } = options;

  const item: ContentListItem = {
    title: entry.data.title,
    href: `${basePath}/${entry.slug}`,
    summary: entry.data.summary,
    tags: entry.data.tags,
  };

  if (linkText) {
    item.linkText = linkText;
  }

  if (includeDate && entry.data.pubDate) {
    const date = formatContentDate(entry.data.pubDate);
    if (date) {
      item.date = date;
    }
  }

  return item;
}

/**
 * Transform multiple content entries to ContentList items
 */
export function transformToContentListItems<T extends CollectionEntry<any>>(
  entries: T[],
  basePath: string,
  options: {
    linkText?: string;
    includeDate?: boolean;
  } = {}
): ContentListItem[] {
  return entries.map((entry) => transformToContentListItem(entry, basePath, options));
}
