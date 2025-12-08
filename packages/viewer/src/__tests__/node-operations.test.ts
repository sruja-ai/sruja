import { describe, it, expect } from 'vitest';
import { collapseNode, expandNode, collapseNodesByType, expandNodesByType, isNodeInFocus, applyFocusDimming, getNode } from '../utils/node-operations';

function makeNode(id: string, type = 'component') {
  const styles: Record<string, any> = {};
  const data: Record<string, any> = { id, type };
  let classes: Set<string> = new Set();
  const descendants = { style: (k: string, v: any) => { styles[`desc:${k}`] = v; } } as any;
  return {
    id: () => id,
    length: 1,
    data: (k: string, v?: any) => { if (v !== undefined) data[k] = v; return data[k]; },
    addClass: (c: string) => { classes.add(c); },
    removeClass: (c: string) => { classes.delete(c); },
    hasClass: (c: string) => classes.has(c),
    descendants: () => descendants,
    style: (k: string, v: any) => { styles[k] = v; },
    styles,
  } as any;
}

function makeCy(nodes: any[], selectedIds: string[] = [], edges: Array<[string, string]> = []) {
  const map = new Map(nodes.map(n => [n.id(), n]));
  return {
    getElementById: (id: string) => map.get(id) || { length: 0 },
    $: (sel: string) => {
      if (sel === 'node:selected') {
        return selectedIds.map(id => map.get(id));
      }
      // node[type="..."] selector
      const m = sel.match(/node\[type=\"(.+?)\"\]/);
      if (m) {
        const t = m[1];
        return nodes.filter(n => n.data('type') === t);
      }
      return [];
    },
    nodes: () => nodes,
    edges: () => edges.map(([s, t]) => ({ source: () => map.get(s), target: () => map.get(t), style: (_k: string, _v: any) => { /* noop, assert via node opacities */ } } as any)),
    elements: () => ({ style: (k: string, v: any) => { nodes.forEach(n => n.style(k, v)); } } as any),
  } as any;
}

describe('node-operations', () => {
  it('collapseNode sets collapsed and hides descendants; expand shows', () => {
    const n = makeNode('Sys.API');
    const cy = makeCy([n]);
    collapseNode(cy, 'Sys.API');
    expect(n.data('collapsed')).toBe(true);
    expect(n.hasClass('collapsed')).toBe(true);
    expect(n.styles['desc:display']).toBe('none');
    expandNode(cy, 'Sys.API');
    expect(n.data('collapsed')).toBe(false);
    expect(n.hasClass('collapsed')).toBe(false);
    expect(n.styles['desc:display']).toBe('element');
  });

  it('collapse/expand by type iterates matching nodes', () => {
    const a = makeNode('Sys.Web', 'container');
    const b = makeNode('Sys.API', 'container');
    const c = makeNode('Sys.DB', 'datastore');
    const cy = makeCy([a, b, c]);
    collapseNodesByType(cy, 'container');
    expect(a.data('collapsed')).toBe(true);
    expect(b.data('collapsed')).toBe(true);
    expandNodesByType(cy, 'container');
    expect(a.data('collapsed')).toBe(false);
    expect(b.data('collapsed')).toBe(false);
    expect(c.data('collapsed')).toBeUndefined();
  });

  it('isNodeInFocus checks container and system scopes', () => {
    expect(isNodeInFocus('Sys.Web.Component', { containerId: 'Sys.Web' })).toBe(true);
    expect(isNodeInFocus('Sys.Web', { containerId: 'Sys.Web' })).toBe(true);
    expect(isNodeInFocus('Sys.Api', { containerId: 'Sys.Web' })).toBe(false);
    expect(isNodeInFocus('Sys.Web.Component', { systemId: 'Sys' })).toBe(true);
    expect(isNodeInFocus('Other', { systemId: 'Sys' })).toBe(false);
  });

  it('applyFocusDimming sets node/edge opacities based on focus', () => {
    const a = makeNode('Sys.Web');
    const b = makeNode('Sys.API');
    const c = makeNode('Other');
    const cy = makeCy([a, b, c], [], [['Sys.Web', 'Sys.API'], ['Other', 'Sys.API']]);
    applyFocusDimming(cy, { systemId: 'Sys' });
    expect(a.styles['opacity']).toBe(1);
    expect(b.styles['opacity']).toBe(1);
    expect(c.styles['opacity']).toBe(0.25);
  });

  it('getNode returns by id or selected single node', () => {
    const a = makeNode('A');
    const b = makeNode('B');
    const cy = makeCy([a, b], ['B']);
    expect(getNode(cy, 'A')).toBe(a);
    expect(getNode(cy)).toBe(b);
  });
});
