import { describe, it, expect } from 'vitest';
import { transformToContentListItem, transformToContentListItems } from './transform';

function makeEntry(slug: string, data: any) {
  return { slug, data } as any;
}

describe('content transform', () => {
  it('transforms single entry with linkText and date', () => {
    const entry = makeEntry('intro', { title: 'Intro', summary: 'S', tags: ['t'], pubDate: '2024-01-02' });
    const item = transformToContentListItem(entry, '/docs', { linkText: 'Read', includeDate: true });
    expect(item.title).toBe('Intro');
    expect(item.href).toBe('/docs/intro');
    expect(item.summary).toBe('S');
    expect(item.tags).toEqual(['t']);
    expect(item.linkText).toBe('Read');
    expect(item.date).toBeTruthy();
  });

  it('omits date when includeDate is false and preserves array mapping', () => {
    const entries = [
      makeEntry('a', { title: 'A' }),
      makeEntry('b', { title: 'B' }),
    ];
    const items = transformToContentListItems(entries, '/docs', { includeDate: false });
    expect(items.map(i => i.href)).toEqual(['/docs/a', '/docs/b']);
    expect(items.every(i => i.date === undefined)).toBe(true);
  });
});
