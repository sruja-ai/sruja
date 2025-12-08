import { describe, it, expect } from 'vitest';
import { SrujaViewer } from './viewer';

describe('SrujaViewer (pure methods)', () => {
  it('escapeXml escapes special characters', () => {
    const v = new SrujaViewer({ container: document.createElement('div'), data: {} as any });
    expect(v.escapeXml('<a>&"\'')).toBe('&lt;a&gt;&amp;&quot;&apos;');
  });

  it('getStyleValue returns default for missing/empty and value when present', () => {
    const v = new SrujaViewer({ container: document.createElement('div'), data: {} as any });
    const ele1 = { data: (k: string) => undefined } as any;
    expect((v as any).getStyleValue(ele1, 'style.icon', 'none')).toBe('none');
    const ele2 = { data: (k: string) => [{ key: 'style.icon', value: '' }] } as any;
    expect((v as any).getStyleValue(ele2, 'style.icon', 'none')).toBe('none');
    const ele3 = { data: (k: string) => [{ key: 'style.icon', value: 'container' }] } as any;
    expect((v as any).getStyleValue(ele3, 'style.icon', 'none')).toBe('container');
  });

  it('getIconDataUri encodes SVG content', () => {
    const v = new SrujaViewer({ container: document.createElement('div'), data: {} as any });
    const uri = (v as any).getIconDataUri('<svg></svg>');
    expect(uri.startsWith('data:image/svg+xml,')).toBe(true);
    expect(uri.includes('%3Csvg%3E')).toBe(true);
  });

  it('getLayout computes rounded positions and sizes', () => {
    const v = new SrujaViewer({ container: document.createElement('div'), data: {} as any });
    const nodes = [
      {
        data: (k: string) => (k === 'id' ? 'A' : undefined),
        position: () => ({ x: 10.6, y: 20.4 }),
        width: () => 99.9,
        height: () => 50.1,
      },
      {
        data: (k: string) => (k === 'id' ? 'B' : undefined),
        position: () => ({ x: 0, y: 0 }),
        width: () => 10,
        height: () => 10,
      },
    ] as any;
    (v as any).cy = { nodes: () => nodes } as any;
    const layout = v.getLayout();
    expect(layout).toEqual({ A: { x: 11, y: 20, width: 100, height: 50 }, B: { x: 0, y: 0, width: 10, height: 10 } });
  });

  it('toJSON clones data and adds layout metadata', () => {
    const data: any = { metadata: { name: 'Arch' }, architecture: { systems: [] } };
    const v = new SrujaViewer({ container: document.createElement('div'), data });
    (v as any).cy = { nodes: () => [{ data: (k: string) => 'A', position: () => ({ x: 1, y: 2 }), width: () => 3, height: () => 4 }] } as any;
    const json = v.toJSON();
    expect(json.metadata.layout.A).toEqual({ x: 1, y: 2, width: 3, height: 4 });
    expect((data.metadata as any).layout).toBeUndefined();
  });

  it('addNode ensures unique id and uses parentId', () => {
    const v = new SrujaViewer({ container: document.createElement('div'), data: {} as any });
    const added: any[] = [];
    (v as any).cy = {
      getElementById: (id: string) => ({ length: id === 'Sys.Web' ? 1 : 0 }),
      add: (node: any) => { added.push(node); },
      layout: () => ({ run: () => {} }),
    } as any;
    v.addNode('container', 'Web', 'Sys');
    expect(added[0].data.id).toBe('Sys.Web1');
    expect(added[0].data.parent).toBe('Sys');
  });
});
