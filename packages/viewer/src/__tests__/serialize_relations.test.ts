import { describe, it, expect } from 'vitest';
import { serializeRelations } from '../serialize/relations';

function makeNode(id: string, type: string, parent?: string) {
  const data: Record<string, any> = { id, type };
  if (parent) data.parent = parent;
  return {
    data: (k: string) => data[k],
    id: () => id,
  } as any;
}

function makeEdge(source: string, target: string, label?: string) {
  return {
    data: () => ({ source, target, label }),
  } as any;
}

function makeCy(nodes: Record<string, any>, edges: any[]) {
  return {
    getElementById: (id: string) => nodes[id] || { data: () => ({}) },
    edges: () => edges,
  } as any;
}

describe('serialize/relations', () => {
  it('attaches relations under system with relative identifiers', () => {
    const nodes: Record<string, any> = {
      'Sys': makeNode('Sys', 'system'),
      'Sys.Web': makeNode('Sys.Web', 'container', 'Sys'),
      'Sys.API.Component': makeNode('Sys.API.Component', 'component', 'Sys.API'),
    };
    const edges = [makeEdge('Sys.Web', 'Sys.API.Component', 'calls')];
    const cy = makeCy(nodes, edges);
    const arch: any = { systems: [{ id: 'Sys' }] };
    serializeRelations(cy as any, arch);
    expect(arch.systems[0].relations).toEqual([
      { from: 'Web', to: 'API.Component', label: 'calls' },
    ]);
  });

  it('uses dot for system self relation', () => {
    const nodes: Record<string, any> = {
      'Sys': makeNode('Sys', 'system'),
      'Sys.API': makeNode('Sys.API', 'container', 'Sys'),
    };
    const edges = [makeEdge('Sys', 'Sys.API', 'uses')];
    const cy = makeCy(nodes, edges);
    const arch: any = { systems: [{ id: 'Sys' }] };
    serializeRelations(cy as any, arch);
    expect(arch.systems[0].relations).toEqual([
      { from: '.', to: 'API', label: 'uses' },
    ]);
  });

  it('falls back to top-level relations when no system found', () => {
    const nodes: Record<string, any> = {
      'A': makeNode('A', 'component'),
      'B': makeNode('B', 'component'),
    };
    const edges = [makeEdge('A', 'B', 'links')];
    const cy = makeCy(nodes, edges);
    const arch: any = {};
    serializeRelations(cy as any, arch);
    expect(arch.relations).toEqual([
      { from: 'A', to: 'B', label: 'links' },
    ]);
  });
});
