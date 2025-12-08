import { describe, it, expect, vi } from 'vitest';
import { createHandleAddNode, createHandleDelete } from '../nodeHandlers';

function makeViewerWithStore() {
  const nodes: any[] = [];
  const edges: any[] = [];
  let selectedId: string | null = null;
  const cyContainer = document.createElement('div');
  cyContainer.getBoundingClientRect = () => ({ x: 0, y: 0, width: 800, height: 600, top: 0, left: 0, right: 800, bottom: 600, toJSON: () => {} } as any);
  const cy: any = {
    destroyed: () => false,
    container: () => cyContainer,
    width: () => 800,
    height: () => 600,
    pan: () => ({ x: 0, y: 0 }),
    zoom: () => 1,
    stop: () => {},
    fit: () => {},
    resize: () => {},
    $: (sel: string) => {
      if (sel === 'node:selected') return selectedId ? [{ id: () => selectedId, data: (k: string) => nodes.find(n => n.data.id === selectedId)?.data[k] }] : [];
      return [];
    },
    nodes: (sel?: string) => {
      if (sel === ':selected') {
        if (!selectedId) return { length: 0, data: () => undefined, first: () => ({ id: () => '' }) };
        const selectedNode = nodes.find(n => n.data.id === selectedId)!;
        return {
          length: 1,
          data: (k: string) => selectedNode.data[k],
          first: () => ({ id: () => selectedNode.data.id }),
        };
      }
      return {
        unselect: () => { selectedId = null; },
        forEach: (cb: any) => nodes.forEach(n => cb({ id: () => n.data.id })),
      };
    },
    edges: () => edges.map(e => ({ data: (k: string) => e[k], source: () => ({ data: () => ({ id: e.source }) }), target: () => ({ data: () => ({ id: e.target }) }) })),
    getElementById: (id: string) => {
      const n = nodes.find(n => n.data.id === id);
      return n ? {
        length: 1,
        id: () => id,
        data: (k: string, v?: any) => { if (v !== undefined) n.data[k] = v; return n.data[k]; },
        position: (pos?: any) => { if (pos) n.position = pos; return n.position || { x: 0, y: 0 }; },
        select: () => { selectedId = id; },
        descendants: () => ({ style: () => {} }),
        remove: () => {
          const idx = nodes.findIndex(nn => nn.data.id === id);
          if (idx >= 0) nodes.splice(idx, 1);
        },
        style: () => {},
        removeStyle: () => {},
      } : { length: 0 };
    },
  };
  const viewer: any = {
    cy,
    addNode: (type: string, label: string, parentId?: string, extraData?: any) => {
      let id = (parentId ? `${parentId}.` : '') + label.replace(/\s+/g, '');
      let unique = id; let c = 1;
      while (cy.getElementById(unique).length > 0) { unique = `${id}${c++}`; }
      nodes.push({ group: 'nodes', data: { id: unique, label, type, parent: parentId, ...(extraData || {}) } });
      return unique;
    },
    addEdge: (s: string, t: string, label?: string) => { edges.push({ source: s, target: t, label }); },
    removeSelected: () => { if (selectedId) { const el = cy.getElementById(selectedId); el.length && (el as any).remove(); selectedId = null; } },
    toJSON: () => {
      const arch: any = { systems: [], persons: [] };
      const systems = nodes.filter(n => n.data.type === 'system');
      systems.forEach(sys => {
        const sysObj: any = { id: sys.data.id, containers: [], datastores: [], queues: [], relations: [] };
        nodes.forEach(n => {
          if (n.data.parent === sys.data.id && n.data.type === 'container') sysObj.containers.push({ id: n.data.id.split('.').slice(1).join('.'), label: n.data.label });
          if (n.data.parent === sys.data.id && n.data.type === 'datastore') sysObj.datastores.push({ id: n.data.id.split('.').slice(1).join('.'), label: n.data.label });
          if (n.data.parent === sys.data.id && n.data.type === 'queue') sysObj.queues.push({ id: n.data.id.split('.').slice(1).join('.'), label: n.data.label });
        });
        edges.forEach(e => {
          const from = e.source.startsWith(sys.data.id + '.') ? e.source.split('.').slice(1).join('.') : (e.source === sys.data.id ? '.' : null);
          const to = e.target.startsWith(sys.data.id + '.') ? e.target.split('.').slice(1).join('.') : (e.target === sys.data.id ? '.' : null);
          if (from && to) sysObj.relations.push({ from, to, label: e.label || '' });
        });
        arch.systems.push(sysObj);
      });
      return { metadata: { name: 'Test' }, architecture: arch };
    },
  };
  return { viewer, cy, nodes, edges, select: (id: string) => { const el: any = cy.getElementById(id); if (el.length) el.select(); } };
}

function makeWasmRoundtrip() {
  let lastJsonStr = '';
  return {
    printJsonToDsl: async (jsonStr: string) => { lastJsonStr = jsonStr; return `DSL:${jsonStr}`; },
    parseDslToJson: async (dsl: string) => {
      const payload = dsl.startsWith('DSL:') ? dsl.slice(4) : '{}';
      return payload;
    },
    getDiagnostics: async () => JSON.stringify({ ok: true, data: JSON.stringify([]) }),
  } as any;
}

describe('node operations DSL roundtrip', () => {
  it('adds container under selected system and roundtrips to DSL and back', async () => {
    vi.useFakeTimers();
    const { viewer, cy, select } = makeViewerWithStore();
    // Add a system first
    viewer.addNode('system', 'Sys');
    select('Sys');
    let dsl = '';
    const wasm = makeWasmRoundtrip();
    const sync = async () => { const json = viewer.toJSON(); dsl = await wasm.printJsonToDsl(JSON.stringify(json)); };
    const setSelectedNodeId = vi.fn();
    const setToast = vi.fn();
    const handleAdd = createHandleAddNode({ viewerRef: { current: viewer }, setSelectedNodeId, syncDiagramToDslState: sync, setToast, setAdrModalOpen: () => {} });
    await handleAdd('container');
    await vi.runAllTimersAsync();
    // Roundtrip
    const jsonStr = await wasm.parseDslToJson(dsl);
    const json = JSON.parse(jsonStr);
    const sys = json.architecture.systems[0];
    expect(sys.containers.length).toBe(1);
  });

  it('adds edge between containers and roundtrips relations', async () => {
    const { viewer } = makeViewerWithStore();
    viewer.addNode('system', 'Sys');
    const c1 = viewer.addNode('container', 'Web', 'Sys');
    const c2 = viewer.addNode('container', 'API', 'Sys');
    viewer.addEdge(c1, c2, 'calls');
    const wasm = makeWasmRoundtrip();
    const json = viewer.toJSON();
    const dsl = await wasm.printJsonToDsl(JSON.stringify(json));
    const parsed = JSON.parse(await wasm.parseDslToJson(dsl));
    const sys = parsed.architecture.systems[0];
    expect(sys.relations[0]).toEqual({ from: 'Web', to: 'API', label: 'calls' });
  });

  it('deletes selected node and roundtrips removal', async () => {
    const { viewer, select } = makeViewerWithStore();
    viewer.addNode('system', 'Sys');
    viewer.addNode('container', 'Web', 'Sys');
    select('Sys.Web');
    const wasm = makeWasmRoundtrip();
    let dsl = '';
    const sync = async () => { const json = viewer.toJSON(); dsl = await wasm.printJsonToDsl(JSON.stringify(json)); };
    const setToast = vi.fn();
    const handleDelete = createHandleDelete({ viewerRef: { current: viewer }, syncDiagramToDslState: sync, setToast });
    await handleDelete();
    const parsed = JSON.parse(await wasm.parseDslToJson(dsl));
    const sys = parsed.architecture.systems[0];
    expect(sys.containers.length).toBe(0);
  });
});
