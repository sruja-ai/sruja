import { describe, it, expect, vi } from 'vitest';
import { handlePropertiesUpdate, syncDiagramToDsl } from '../viewerUtils';

function makeViewerWithCy(selectedId: string) {
  const nodeData: any = { id: selectedId, label: 'Web', description: '', technology: '' };
  const node: any = {
    length: 1,
    data: (k?: string, v?: any) => {
      if (k && v !== undefined) nodeData[k] = v;
      return k ? nodeData[k] : nodeData;
    },
    position: (pos?: any) => (pos ? undefined : { x: 0, y: 0 }),
    select: vi.fn(),
  };
  const cy: any = {
    zoom: vi.fn().mockReturnValue(2),
    pan: vi.fn().mockReturnValue({ x: 10, y: 20 }),
    stop: vi.fn(),
    nodes: () => ({ unselect: vi.fn() }),
    getElementById: (id: string) => (id === selectedId ? node : { length: 0 }),
    destroyed: () => false,
  };
  const viewer: any = {
    cy,
    load: vi.fn(async () => {}),
  };
  return { viewer, cy, node, nodeData };
}

describe('viewerUtils', () => {
  it('handlePropertiesUpdate updates node data directly and restores view', async () => {
    vi.useFakeTimers();
    const selectedId = 'Sys.Web';
    const { viewer, cy, node, nodeData } = makeViewerWithCy(selectedId);
    const wasmApi = { printJsonToDsl: vi.fn(async () => 'dsl') } as any;
    const setArchData = vi.fn();
    const setDsl = vi.fn();
    const setToast = vi.fn();
    const isUpdatingRef = { current: false } as any;
    const newData: any = { architecture: { systems: [{ id: 'Sys', containers: [{ id: 'Sys.Web', label: 'Web2', description: 'UI', technology: 'React' }] }] } };

    await handlePropertiesUpdate(newData, { current: wasmApi } as any, { current: viewer } as any, selectedId, setArchData, setDsl, setToast, isUpdatingRef);

    expect(setArchData).toHaveBeenCalledWith(newData);
    expect(setDsl).toHaveBeenCalledWith('dsl');
    expect(node.data('label')).toBe('Web2');
    expect(node.data('description')).toBe('UI');
    expect(node.data('technology')).toBe('React');
    expect(cy.nodes().unselect).toBeDefined();
    expect(node.select).toHaveBeenCalled();
    // After restore
    expect(cy.stop).toHaveBeenCalled();
    expect(cy.zoom).toHaveBeenCalledWith(2);
    expect(cy.pan).toHaveBeenCalledWith({ x: 10, y: 20 });

    await vi.runAllTimersAsync();
    expect(isUpdatingRef.current).toBe(false);
    vi.useRealTimers();
  });

  it('syncDiagramToDsl roundtrips viewer JSON to DSL and sets archData', async () => {
    const viewer = {
      toJSON: () => ({ metadata: {}, architecture: { systems: [] } }),
    } as any;
    const wasm = {
      printJsonToDsl: vi.fn(async () => 'architecture "A" {}'),
    } as any;
    const setDsl = vi.fn();
    const setToast = vi.fn();
    const setArchData = vi.fn();
    await syncDiagramToDsl({ current: viewer } as any, { current: wasm } as any, setDsl, setToast, setArchData);
    expect(setDsl).toHaveBeenCalledWith('architecture "A" {}');
    expect(setArchData).toHaveBeenCalledWith({ metadata: {}, architecture: { systems: [] } });
  });

  it('syncDiagramToDsl handles print error and shows toast', async () => {
    (globalThis as any).logger = (globalThis as any).logger || { error: () => {}, warn: () => {}, info: () => {} };
    const viewer = { toJSON: () => ({ metadata: {}, architecture: { systems: [] } }) } as any;
    const wasm = { printJsonToDsl: vi.fn(async () => { throw new Error('fail'); }) } as any;
    const setDsl = vi.fn();
    const setToast = vi.fn();
    const setArchData = vi.fn();
    await syncDiagramToDsl({ current: viewer } as any, { current: wasm } as any, setDsl, setToast, setArchData);
    expect(setToast).toHaveBeenCalledWith({ message: 'Failed to sync diagram to DSL', type: 'error' });
  });
});
