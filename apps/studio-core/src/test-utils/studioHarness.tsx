import React from 'react';
import { render } from '@testing-library/react';
import { StudioStateProvider } from '../context/StudioStateContext';
import { StudioEditingProvider } from '../context/StudioEditingContext';
import type { ViewerInstance } from '@sruja/viewer';
import type { WasmApi } from '@sruja/shared';

export function createMockViewer(): ViewerInstance {
  const cyContainer = document.createElement('div');
  cyContainer.getBoundingClientRect = () => ({ x: 0, y: 0, width: 800, height: 600, top: 0, left: 0, right: 800, bottom: 600, toJSON: () => {} } as any);
  const cy: any = {
    destroyed: () => false,
    container: () => cyContainer,
    elements: () => ({ length: 0 }),
    fit: () => {},
    resize: () => {},
    nodes: () => [],
    edges: () => [],
    zoom: () => 1,
    pan: () => ({ x: 0, y: 0 }),
    stop: () => {},
    on: () => {},
    off: () => {},
    ready: (cb: Function) => cb(),
    $: () => [],
    getElementById: () => ({ length: 0 }),
  };
  const viewer: ViewerInstance = {
    cy,
    load: async () => {},
    toJSON: () => ({ metadata: {}, architecture: {} } as any),
    getLayout: () => ({}) as any,
    exportPNG: () => '',
    exportSVG: () => '<svg></svg>',
    selectNode: () => {},
    toggleCollapse: () => {},
    addNode: () => {},
    addEdge: () => {},
    destroy: () => {},
    cyInstance: null,
  } as any;
  return viewer;
}

export function createMockWasm(): WasmApi {
  return {
    parseDslToJson: async (dsl: string) => JSON.stringify({ metadata: { name: 'Test' }, architecture: { systems: [{ id: 'Sys' }] } }),
    getDiagnostics: async () => JSON.stringify({ ok: true, data: JSON.stringify([{ severity: 'Warning', code: 'W1', message: 'Note', location: { file: 'dsl', line: 1, column: 1 } }]) }),
    printJsonToDsl: async (_json: string) => 'architecture "A" {}',
  } as any;
}

export function renderWithStudio(ui: React.ReactElement, opts?: { viewer?: ViewerInstance; wasm?: WasmApi }) {
  (globalThis as any).logger = (globalThis as any).logger || { error: () => {}, warn: () => {}, info: () => {} };
  const viewerRef = { current: opts?.viewer || createMockViewer() } as React.RefObject<ViewerInstance | null>;
  const wasmApiRef = { current: opts?.wasm || createMockWasm() } as React.RefObject<WasmApi | null>;
  const onToast = () => {};
  return render(
    <StudioStateProvider>
      <StudioEditingProvider viewerRef={viewerRef} wasmApiRef={wasmApiRef} onToast={onToast}>
        {ui}
      </StudioEditingProvider>
    </StudioStateProvider>
  );
}
