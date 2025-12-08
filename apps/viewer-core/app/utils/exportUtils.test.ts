import { describe, it, expect, vi } from 'vitest';
import { exportDiagram } from './exportUtils';

function makeViewer(overrides: Partial<any> = {}) {
  return {
    exportPNG: undefined,
    exportSVG: undefined,
    cy: undefined,
    ...overrides,
  };
}

async function withAnchorCaptureAsync(fn: () => Promise<void>) {
  const origCreate = document.createElement.bind(document);
  let captured: HTMLAnchorElement | null = null;
  const createSpy = vi.spyOn(document, 'createElement').mockImplementation((tag: any) => {
    const el = origCreate(tag) as any;
    if (tag === 'a') {
      el.click = vi.fn();
      captured = el as HTMLAnchorElement;
    }
    return el;
  });
  try {
    await fn();
  } finally {
    createSpy.mockRestore();
  }
  return captured!;
}

describe('exportUtils.exportDiagram', () => {
  it('reports error when PNG export not available', async () => {
    const viewer = makeViewer();
    const setToast = vi.fn();
    await exportDiagram(viewer as any, null, '', null, { format: 'png', filename: 'arch', scale: 1, includeTimestamp: false, includeMetadata: false }, setToast);
    expect(setToast).toHaveBeenCalledWith({ message: 'PNG export not available', type: 'error' });
  });

  it('reports error when SVG export not available', async () => {
    const viewer = makeViewer();
    const setToast = vi.fn();
    await exportDiagram(viewer as any, null, '', null, { format: 'svg', filename: 'arch', scale: 1, includeTimestamp: false, includeMetadata: false }, setToast);
    expect(setToast).toHaveBeenCalledWith({ message: 'Failed to export SVG', type: 'error' });
  });

  it('reports error when markdown export lacks wasmApi', async () => {
    const viewer = makeViewer();
    const setToast = vi.fn();
    await exportDiagram(viewer as any, null, 'dsl', null, { format: 'markdown', filename: 'arch', scale: 1, includeTimestamp: false, includeMetadata: false }, setToast);
    expect(setToast).toHaveBeenCalledWith({ message: 'WASM API not available for markdown export', type: 'error' });
  });

  it('exports SVG via wasmApi when available, includes metadata if requested', async () => {
    const viewer = makeViewer({ exportSVG: vi.fn() });
    const setToast = vi.fn();
    const wasmApi = { dslToSvg: vi.fn().mockResolvedValue('<svg></svg>') } as any;
    const arch = { metadata: { name: 'Arch', version: '1.2.3' } } as any;
    const a = await withAnchorCaptureAsync(() => exportDiagram(viewer as any, wasmApi, 'dsl', arch, { format: 'svg', filename: 'arch', scale: 1, includeTimestamp: false, includeMetadata: true }, setToast));
    // Blob URL
    expect(a.download).toBe('arch.svg');
  });

  it('falls back to viewer.exportSVG when wasm fails', async () => {
    const viewer = makeViewer({ exportSVG: vi.fn().mockReturnValue('<svg></svg>') });
    const setToast = vi.fn();
    const wasmApi = { dslToSvg: vi.fn().mockRejectedValue(new Error('fail')) } as any;
    const a = await withAnchorCaptureAsync(() => exportDiagram(viewer as any, wasmApi, 'dsl', null, { format: 'svg', filename: 'arch', scale: 1, includeTimestamp: false, includeMetadata: false }, setToast));
    expect(a.download).toBe('arch.svg');
  });

  it('exports JSON when archData provided', async () => {
    const viewer = makeViewer();
    const setToast = vi.fn();
    const arch = { metadata: { name: 'Arch' } } as any;
    const a = await withAnchorCaptureAsync(() => exportDiagram(viewer as any, null, '', arch, { format: 'json', filename: 'arch', scale: 1, includeTimestamp: false, includeMetadata: false }, setToast));
    expect(a.download).toBe('arch.json');
    expect(setToast).toHaveBeenCalledWith({ message: 'Exported arch.json', type: 'success' });
  });

  it('exports markdown via wasm and appends branding', async () => {
    const viewer = makeViewer();
    const setToast = vi.fn();
    const wasmApi = { dslToMarkdown: vi.fn().mockResolvedValue('# Title') } as any;
    const a = await withAnchorCaptureAsync(() => exportDiagram(viewer as any, wasmApi, 'dsl', null, { format: 'markdown', filename: 'arch', scale: 1, includeTimestamp: false, includeMetadata: false }, setToast));
    expect(a.download).toBe('arch.markdown');
    expect(setToast).toHaveBeenCalledWith({ message: 'Exported arch.markdown', type: 'success' });
  });

  it('exports HTML embedding escaped JSON and branding', async () => {
    const viewer = makeViewer();
    const setToast = vi.fn();
    const arch = { metadata: { name: 'Arch' } } as any;
    const a = await withAnchorCaptureAsync(() => exportDiagram(viewer as any, null, '', arch, { format: 'html', filename: 'arch', scale: 1, includeTimestamp: false, includeMetadata: false }, setToast));
    expect(a.download).toBe('arch.html');
    expect(setToast).toHaveBeenCalledWith({ message: 'Exported arch.html', type: 'success' });
  });

  it('exports PNG via viewer exportPNG fallback', async () => {
    const viewer = makeViewer({ exportPNG: vi.fn().mockReturnValue('data:image/png;base64,AAA') });
    const setToast = vi.fn();
    const a = await withAnchorCaptureAsync(() => exportDiagram(viewer as any, null, '', null, { format: 'png', filename: 'arch', scale: 2, includeTimestamp: false, includeMetadata: false } as any, setToast));
    expect(a.download).toBe('arch.png');
    expect(setToast).toHaveBeenCalledWith({ message: 'Exported arch.png', type: 'success' });
  });
});
