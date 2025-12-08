import { describe, it, expect, vi, beforeEach } from 'vitest';
import { downloadHtml, downloadMarkdown, downloadJson, downloadPdf } from './downloads';

const ARCH = { metadata: { name: 'My Arch' } } as any;

function mockUrl() {
  const create = vi.spyOn(URL, 'createObjectURL').mockReturnValue('blob:mock');
  const revoke = vi.spyOn(URL, 'revokeObjectURL').mockImplementation(() => {});
  return { create, revoke };
}

function withAnchorCapture(fn: () => void) {
  const origCreate = document.createElement.bind(document);
  const createSpy = vi.spyOn(document, 'createElement').mockImplementation((tag: any) => {
    const el = origCreate(tag) as any;
    if (tag === 'a') {
      el.click = vi.fn();
    }
    return el;
  });
  let captured: HTMLAnchorElement | null = null;
  const origAppend = document.body.appendChild.bind(document.body);
  const appendSpy = vi.spyOn(document.body, 'appendChild').mockImplementation((el: any) => {
    captured = el as HTMLAnchorElement;
    return origAppend(el);
  });
  try {
    fn();
  } finally {
    createSpy.mockRestore();
    appendSpy.mockRestore();
  }
  return captured!;
}

describe('downloads', () => {
  beforeEach(() => {
    document.body.innerHTML = '';
  });

  it('downloads HTML with sanitized filename', () => {
    const { create, revoke } = mockUrl();
    const a = withAnchorCapture(() => downloadHtml('<html></html>', { metadata: { name: 'A&B Arch' } } as any));
    expect(a.download).toBe('a_b_arch.html');
    expect(create).toHaveBeenCalled();
    expect(revoke).toHaveBeenCalledWith('blob:mock');
  });

  it('downloads Markdown with correct filename', () => {
    const { create, revoke } = mockUrl();
    const a = withAnchorCapture(() => downloadMarkdown('# md', ARCH));
    expect(a.download).toBe('my_arch.md');
    expect(create).toHaveBeenCalled();
    expect(revoke).toHaveBeenCalledWith('blob:mock');
  });

  it('downloads JSON with correct filename and content type', () => {
    const { create, revoke } = mockUrl();
    const a = withAnchorCapture(() => downloadJson({ metadata: { name: 'My Arch' } } as any));
    expect(a.download).toBe('my_arch.json');
    expect(create).toHaveBeenCalled();
    expect(revoke).toHaveBeenCalledWith('blob:mock');
  });

  it('downloads PDF using existing URL without createObjectURL', () => {
    const createSpy = vi.spyOn(URL, 'createObjectURL');
    createSpy.mockClear();
    const a = withAnchorCapture(() => downloadPdf('blob:something', ARCH));
    expect(createSpy).not.toHaveBeenCalled();
    expect(a.href.endsWith('blob:something')).toBe(true);
    expect(a.download).toBe('my_arch.pdf');
  });
});
