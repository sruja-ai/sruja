import { describe, it, expect } from 'vitest';
import { updateDocumentationForNode } from '../documentationUtils';

const ARCH: any = { architecture: { systems: [{ id: 'Sys' }] } };

describe('documentationUtils', () => {
  it('clears state when nodeId or archData missing', () => {
    let state: any = {};
    const setDoc = (s: any) => { state = s; };
    updateDocumentationForNode(null, ARCH, setDoc as any);
    expect(state.selectedNodeType).toBeNull();
    expect(state.selectedNodeId).toBeUndefined();
  });

  it('sets documentation when node found', () => {
    let state: any = {};
    const setDoc = (s: any) => { state = s; };
    updateDocumentationForNode('Sys', ARCH, setDoc as any);
    expect(state.selectedNodeType).toBe('system');
    expect(state.selectedNodeId).toBe('Sys');
    expect(state.selectedNodeLabel).toBe('Sys');
  });
});
