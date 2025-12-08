import { describe, it, expect } from 'vitest';
import { extractLayout, updateLayoutMetadata } from '../serialize/layout';

function makeNode(id: string, pos: { x: number; y: number }, size: { w: number; h: number }) {
  return {
    data: (k: string) => (k === 'id' ? id : undefined),
    position: () => pos,
    width: () => size.w,
    height: () => size.h,
  } as any;
}

function makeCy(nodes: any[]) {
  return {
    nodes: () => nodes,
  } as any;
}

describe('serialize/layout', () => {
  it('extractLayout rounds and returns layout per node', () => {
    const cy = makeCy([
      makeNode('A', { x: 10.6, y: 20.4 }, { w: 99.9, h: 50.1 }),
      makeNode('B', { x: 0, y: 0 }, { w: 10, h: 10 }),
    ]);
    const layout = extractLayout(cy as any);
    expect(layout).toEqual({
      A: { x: 11, y: 20, width: 100, height: 50 },
      B: { x: 0, y: 0, width: 10, height: 10 },
    });
  });

  it('updateLayoutMetadata merges into metadata.layout', () => {
    const cy = makeCy([makeNode('A', { x: 1, y: 2 }, { w: 3, h: 4 })]);
    const metadata: any = { layout: { B: { x: 9, y: 9, width: 9, height: 9 } } };
    updateLayoutMetadata(cy as any, metadata);
    expect(metadata.layout.A).toEqual({ x: 1, y: 2, width: 3, height: 4 });
    expect(metadata.layout.B).toEqual({ x: 9, y: 9, width: 9, height: 9 });
  });
});
