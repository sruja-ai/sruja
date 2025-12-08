import { describe, it, expect, vi } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useClickToConnect } from '../useClickToConnect';

function makeViewer() {
  let handler: any;
  const styled: Record<string, any> = {};
  const nodes: Record<string, any> = {
    A: { id: () => 'A', isNode: () => true, style: (k: string, v: any) => { styled[k] = v; }, removeStyle: () => { Object.keys(styled).forEach(k => delete styled[k]); } },
    B: { id: () => 'B', isNode: () => true, style: () => {}, removeStyle: () => {} },
  };
  const cy = {
    on: (_evt: string, cb: any) => { handler = cb; },
    removeListener: () => {},
    getElementById: (id: string) => nodes[id] || { removeStyle: () => {} },
  } as any;
  const viewer = { cy } as any;
  return { viewer, getHandler: () => handler, styled };
}

describe('useClickToConnect', () => {
  it('sets source node and opens relation modal on second node', () => {
    const { viewer, getHandler } = makeViewer();
    let sourceNode: string | null = null;
    let isAddingRelation = true;
    let modalConfig: any = null;
    const setSourceNode = (v: string | null) => { sourceNode = v; };
    const setIsAddingRelation = (v: boolean) => { isAddingRelation = v; };
    const setModalConfig = (c: any) => { modalConfig = c; };
    const { rerender } = renderHook((props: any) => useClickToConnect(props), { initialProps: { viewerRef: { current: viewer }, isAddingRelation, sourceNode, setSourceNode, setIsAddingRelation, setModalConfig } });
    const tap = getHandler();
    // stub getComputedStyle
    (globalThis as any).getComputedStyle = () => ({ getPropertyValue: () => '#ef4444' });
    tap({ target: viewer.cy.getElementById('A') });
    expect(sourceNode).toBe('A');
    rerender({ viewerRef: { current: viewer }, isAddingRelation, sourceNode, setSourceNode, setIsAddingRelation, setModalConfig });
    const tap2 = getHandler();
    tap2({ target: viewer.cy.getElementById('B') });
    expect(modalConfig?.isOpen).toBe(true);
    expect(modalConfig?.data).toEqual({ source: 'A', target: 'B' });
  });

  it('cancels on background and clears style', () => {
    const { viewer, getHandler, styled } = makeViewer();
    let sourceNode: string | null = 'A';
    let isAddingRelation = true;
    const setSourceNode = (v: string | null) => { sourceNode = v; };
    const setIsAddingRelation = (v: boolean) => { isAddingRelation = v; };
    const setModalConfig = vi.fn();
    renderHook(() => useClickToConnect({ viewerRef: { current: viewer }, isAddingRelation, sourceNode, setSourceNode, setIsAddingRelation, setModalConfig }));
    const tap = getHandler();
    tap({ target: viewer.cy });
    expect(sourceNode).toBeNull();
    expect(isAddingRelation).toBe(false);
    expect(Object.keys(styled).length).toBe(0);
  });
});
