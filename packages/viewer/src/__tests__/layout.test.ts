import { describe, it, expect, vi } from 'vitest';
import { computeLayoutOptions, fitPresetWithRetry, waitForContainerSize, createLayoutStopHandler } from '../layout';

function makeContainer(rect: { width: number; height: number }) {
  return {
    getBoundingClientRect: () => rect,
  } as any;
}

describe('layout', () => {
  it('computeLayoutOptions returns preset or engine options', () => {
    const preset = computeLayoutOptions(true, 'dagre');
    expect(preset.name).toBe('preset');
    const dagre = computeLayoutOptions(false, 'dagre');
    expect(dagre.name).toBe('dagre');
    const defaulted = computeLayoutOptions(false, 'unknown');
    expect(defaulted.name).toBe('dagre');
  });

  it('fitPresetWithRetry retries until container has size', () => {
    vi.useFakeTimers();
    const rect = { width: 0, height: 0 };
    const container = makeContainer(rect);
    const fit = vi.fn();
    const cy = { fit, destroyed: () => false } as any;
    fitPresetWithRetry(cy, container as any);
    // Still zero
    expect(fit).not.toHaveBeenCalled();
    // Make non-zero and advance retries
    rect.width = 100; rect.height = 100;
    vi.advanceTimersByTime(400);
    expect(fit).toHaveBeenCalled();
    vi.useRealTimers();
  });

  it('waitForContainerSize resolves when ResizeObserver detects size', async () => {
    const callbacks: Function[] = [];
    (globalThis as any).ResizeObserver = class {
      cb: Function;
      constructor(cb: Function) { this.cb = cb; callbacks.push(cb); }
      observe() {}
      disconnect() {}
    } as any;
    const rect = { width: 0, height: 0 };
    const el = makeContainer(rect) as any as HTMLElement;
    const promise = waitForContainerSize(el);
    rect.width = 50; rect.height = 50;
    callbacks.forEach(cb => cb());
    await promise;
    expect(true).toBe(true);
  });

  it('createLayoutStopHandler fits on non-zero and detaches listener', () => {
    const rect = { width: 100, height: 100 };
    const container = makeContainer(rect);
    const fit = vi.fn();
    const off = vi.fn();
    const cy = { fit, off, destroyed: () => false } as any;
    const onDone = vi.fn();
    const handler = createLayoutStopHandler(cy, container as any, onDone);
    handler();
    expect(fit).toHaveBeenCalled();
    expect(off).toHaveBeenCalled();
  });
});
