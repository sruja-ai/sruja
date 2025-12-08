import { describe, it, expect, vi } from 'vitest';
import { generateHtmlPreview } from './previews';

describe('injectPreviewDiagnostics placement', () => {
  it('inserts before </head> when head exists', async () => {
    const wasmApi = { dslToHtml: vi.fn().mockResolvedValue('<html><head><title>X</title></head><body>Y</body></html>') } as any;
    const setHtml = vi.fn();
    await generateHtmlPreview('dsl', wasmApi, setHtml, () => {});
    const html = setHtml.mock.calls[0][0] as string;
    const idxHead = html.indexOf('</head>');
    const idxSnippet = html.indexOf('sruja:html-preview');
    expect(idxSnippet).toBeGreaterThan(-1);
    expect(idxSnippet).toBeLessThan(idxHead);
  });

  it('inserts after <body> when only body exists', async () => {
    const wasmApi = { dslToHtml: vi.fn().mockResolvedValue('<html><body>Y</body></html>') } as any;
    const setHtml = vi.fn();
    await generateHtmlPreview('dsl', wasmApi, setHtml, () => {});
    const html = setHtml.mock.calls[0][0] as string;
    const idxBody = html.indexOf('<body>');
    const idxSnippet = html.indexOf('sruja:html-preview');
    expect(idxSnippet).toBeGreaterThan(idxBody);
  });

  it('prefixes snippet when neither head nor body exists', async () => {
    const wasmApi = { dslToHtml: vi.fn().mockResolvedValue('<div>plain</div>') } as any;
    const setHtml = vi.fn();
    await generateHtmlPreview('dsl', wasmApi, setHtml, () => {});
    const html = setHtml.mock.calls[0][0] as string;
    expect(html.startsWith('\n<script>')).toBe(true);
  });
});
