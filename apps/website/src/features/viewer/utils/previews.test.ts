import { describe, it, expect, vi } from 'vitest';
import { generateHtmlPreview, generateMarkdownPreview, generatePdfPreview } from './previews';
import { jsPDF } from 'jspdf';

describe('viewer previews', () => {
  it('injects diagnostics into HTML preview when wasm html export available', async () => {
    const wasmApi = { dslToHtml: vi.fn().mockResolvedValue('<html><head></head><body>Hello</body></html>') } as any;
    const setHtml = vi.fn();
    const setLoading = vi.fn();
    await generateHtmlPreview('dsl', wasmApi, setHtml, setLoading);
    const html = setHtml.mock.calls[0][0] as string;
    expect(html.includes('sruja:html-preview')).toBe(true);
    expect(setLoading).toHaveBeenCalledWith(true);
    expect(setLoading).toHaveBeenCalledWith(false);
  });

  it('falls back to error HTML when wasm html export missing', async () => {
    const wasmApi = {} as any;
    const setHtml = vi.fn();
    const setLoading = vi.fn();
    await generateHtmlPreview('dsl', wasmApi, setHtml, setLoading);
    const html = setHtml.mock.calls[0][0] as string;
    expect(html.includes('HTML Export Error')).toBe(true);
    expect(setLoading).toHaveBeenCalledWith(false);
  });

  it('generates markdown preview via wasm and handles error fallback', async () => {
    const setMarkdown = vi.fn();
    const setLoading = vi.fn();
    const wasmOk = { dslToMarkdown: vi.fn().mockResolvedValue('# Title') } as any;
    await generateMarkdownPreview('dsl', wasmOk, setMarkdown, setLoading);
    expect(setMarkdown).toHaveBeenCalledWith('# Title');

    const wasmBad = { dslToMarkdown: vi.fn().mockRejectedValue(new Error('fail')) } as any;
    setMarkdown.mockClear(); setLoading.mockClear();
    await generateMarkdownPreview('dsl', wasmBad, setMarkdown, setLoading);
    const md = setMarkdown.mock.calls[0][0] as string;
    expect(md.startsWith('# Error')).toBe(true);
    expect(setLoading).toHaveBeenCalledWith(false);
  });

  it('sets empty URL and toggles loading on PDF error path', async () => {
    const wasmBad = { dslToMarkdown: vi.fn().mockRejectedValue(new Error('fail')) } as any;
    const setUrl = vi.fn();
    const setLoading = vi.fn();
    await generatePdfPreview('dsl', null, wasmBad, setUrl, setLoading);
    expect(setUrl).toHaveBeenCalledWith('');
    expect(setLoading).toHaveBeenCalledWith(false);
  });
});
