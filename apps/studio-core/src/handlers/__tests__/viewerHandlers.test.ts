import { describe, it, expect, vi } from 'vitest';
import { createHandleZoomIn, createHandleZoomOut, createHandleFitToScreen, createHandleSetLevel, createHandleShare } from '../viewerHandlers';

function makeViewer() {
  const cy = { zoom: vi.fn().mockReturnValue(1), fit: vi.fn(), pan: vi.fn() } as any;
  return { cy, setLevel: vi.fn() } as any;
}

describe('viewerHandlers', () => {
  it('zoom in/out and fit call cy methods', () => {
    const v = makeViewer();
    const setZoomLevel = vi.fn();
    createHandleZoomIn({ viewerRef: { current: v }, setZoomLevel })();
    expect(v.cy.zoom).toHaveBeenCalledWith(1.2);
    createHandleZoomOut({ viewerRef: { current: v }, setZoomLevel })();
    expect(v.cy.zoom).toHaveBeenCalledWith(1 / 1.2);
    createHandleFitToScreen({ viewerRef: { current: v }, setZoomLevel })();
    expect(v.cy.fit).toHaveBeenCalled();
  });

  it('set level forwards to viewer and updates state', () => {
    const v = makeViewer();
    const setCurrentLevel = vi.fn();
    createHandleSetLevel({ viewerRef: { current: v }, setCurrentLevel })(2);
    expect(v.setLevel).toHaveBeenCalledWith(2);
    expect(setCurrentLevel).toHaveBeenCalledWith(2);
  });

  it('share copies URL with params', () => {
    const write = vi.fn();
    (navigator as any).clipboard = { writeText: write };
    const setToast = vi.fn();
    createHandleShare({ dsl: 'x', currentLevel: 2, focusSystemId: 'Sys', focusContainerId: undefined, setToast })();
    expect(write).toHaveBeenCalled();
    const url = write.mock.calls[0][0] as string;
    expect(url.includes('#code=')).toBe(true);
    expect(url.includes('level=2')).toBe(true);
    expect(url.includes('focus=Sys')).toBe(true);
    expect(setToast).toHaveBeenCalledWith({ message: 'Share link copied', type: 'success' });
  });

  it('share handles clipboard failure', async () => {
    const write = vi.fn(() => { throw new Error('no clipboard'); });
    (navigator as any).clipboard = { writeText: write };
    const setToast = vi.fn();
    createHandleShare({ dsl: 'x', currentLevel: undefined, focusSystemId: undefined, focusContainerId: undefined, setToast })();
    expect(setToast).toHaveBeenCalledWith({ message: 'Failed to copy share link', type: 'error' });
  });
});
