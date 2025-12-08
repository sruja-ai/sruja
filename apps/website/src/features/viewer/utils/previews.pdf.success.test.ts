import { describe, it, expect, vi } from 'vitest';

vi.mock('jspdf', () => {
  return {
    jsPDF: class {
      html(_html: string, opts: any) { opts.callback(this); }
      output(_type: string) { return new Blob(['pdf']); }
    },
  };
});

describe('viewer previews (pdf success)', () => {
  it('generates PDF and sets blob URL on success', async () => {
    const { generatePdfPreview } = await import('./previews');
    const wasmOk = { dslToMarkdown: vi.fn().mockResolvedValue('# Title\n\nBody') } as any;
    const setUrl = vi.fn();
    const setLoading = vi.fn();
    await generatePdfPreview('dsl', null, wasmOk, setUrl, setLoading);
    const url = setUrl.mock.calls[0][0] as string;
    expect(typeof url).toBe('string');
    expect(url.startsWith('blob:')).toBe(true);
    expect(setLoading).toHaveBeenCalledWith(false);
  });
});
