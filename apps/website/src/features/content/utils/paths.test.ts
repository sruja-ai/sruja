import { describe, it, expect } from 'vitest';
import { generateContentPaths } from './paths';

function makeEntry(slug: string, data: any) {
  return { slug, data } as any;
}

describe('content paths', () => {
  const entries = [
    makeEntry('a', { title: 'A', weight: 2, pubDate: '2024-01-01' }),
    makeEntry('b', { title: 'B', weight: 1, pubDate: '2024-02-01' }),
    makeEntry('c', { title: 'C', weight: 3, pubDate: '2023-12-01' }),
  ];

  it('generates weighted navigation with previous and next', () => {
    const paths = generateContentPaths(entries, '/docs', 'weight');
    expect(paths.length).toBe(3);
    // sorted by weight: b(1), a(2), c(3)
    expect(paths[0].params.slug).toBe('b');
    expect(paths[0].props.previous).toBeNull();
    expect(paths[0].props.next?.href).toBe('/docs/a');
    expect(paths[1].props.previous?.href).toBe('/docs/b');
    expect(paths[1].props.next?.href).toBe('/docs/c');
    expect(paths[2].props.previous?.href).toBe('/docs/a');
    expect(paths[2].props.next).toBeNull();
  });

  it('supports date sort and filtering function', () => {
    const filtered = generateContentPaths(entries, '/docs', 'date', (e) => e.slug !== 'b');
    // filtered out b; date sort: a(2024-01-01) before c(2023-12-01)
    expect(filtered.map(p => p.params.slug)).toEqual(['a', 'c']);
    expect(filtered[0].props.next?.href).toBe('/docs/c');
  });
});
