import { describe, it, expect, vi } from 'vitest';
import { createHandlePaste, createHandleCopy } from '../nodeHandlers';

function makeViewer() {
  const nodes: any[] = [];
  let selected: string | null = null;
  const cy: any = {
    pan: () => ({ x: 0, y: 0 }),
    zoom: () => 1,
    width: () => 800,
    height: () => 600,
    getElementById: (id: string) => {
      const n = nodes.find(n => n.data.id === id);
      return n ? { length: 1, data: () => n.data, position: (pos?: any) => { if (pos) n.position = pos; return n.position || { x: 0, y: 0 }; }, select: () => { selected = id; } } : { length: 0 };
    },
    nodes: (sel?: string) => sel === ':selected' ? { length: selected ? 1 : 0, data: (k: string) => ({ type: 'system' }[k]), first: () => ({ id: () => selected }) } : [],
  };
  const viewer: any = {
    cy,
    addNode: (type: string, label: string, parentId?: string, extraData?: any) => {
      const base = (parentId ? `${parentId}.` : '') + label.replace(/\s+/g, '');
      let id = base; let c = 1; while (cy.getElementById(id).length) id = `${base}${c++}`;
      nodes.push({ group: 'nodes', data: { id, label, type, parent: parentId, ...(extraData || {}) } });
      return id;
    },
  };
  return { viewer, cy, nodes };
}

describe('nodeHandlers paste', () => {
  it('copies and pastes node, positions and syncs', async () => {
    vi.useFakeTimers();
    const { viewer } = makeViewer();
    viewer.addNode('system', 'Sys');
    viewer.addNode('container', 'Web', 'Sys');
    const setToast = vi.fn();
    const setCopiedNode = vi.fn();
    const copy = createHandleCopy({ viewerRef: { current: viewer }, selectedNodeId: 'Sys.Web', archData: { architecture: { systems: [{ id: 'Sys', containers: [{ id: 'Web', label: 'Web' }] }] } } as any, setCopiedNode, setToast });
    copy();
    const copiedNode = { id: 'Sys.Web', data: { id: 'Sys.Web', label: 'Web' }, type: 'container' } as any;
    const setSelectedNodeId = vi.fn();
    const sync = vi.fn();
    const paste = createHandlePaste({ viewerRef: { current: viewer }, copiedNode, setSelectedNodeId, syncDiagramToDslState: sync, setToast });
    await paste();
    await vi.runAllTimersAsync();
    expect(sync).toHaveBeenCalled();
    expect(setSelectedNodeId).toHaveBeenCalled();
    vi.useRealTimers();
  });
});
