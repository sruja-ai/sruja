import { describe, it, expect, vi, beforeEach } from 'vitest';
import { updateUrlWithCode, copyShareUrl } from './urlState';
import { URL_UPDATE_DEBOUNCE_MS } from '../constants';

describe('urlState', () => {
  beforeEach(() => {
    // Reset location
    Object.defineProperty(window, 'location', {
      value: { origin: 'https://example.com', pathname: '/viewer', href: 'https://example.com/viewer' },
      writable: true,
    });
  });

  it('updates URL with compressed DSL after debounce and clears existing timeouts', () => {
    vi.useFakeTimers();
    const replaceSpy = vi.spyOn(window.history, 'replaceState');
    const existing = setTimeout(() => {}, 1000);
    const setTimeoutRef = vi.fn();

    updateUrlWithCode('architecture "A" {}', existing as any, setTimeoutRef);
    // Advance debounce
    vi.advanceTimersByTime(URL_UPDATE_DEBOUNCE_MS);
    expect(replaceSpy).toHaveBeenCalled();
    const url = (replaceSpy.mock.calls[0] as any)[2] as string;
    expect(url.startsWith('/viewer#code=')).toBe(true);
    vi.useRealTimers();
  });

  it('removes code from URL when DSL is empty', () => {
    vi.useFakeTimers();
    const replaceSpy = vi.spyOn(window.history, 'replaceState');
    updateUrlWithCode('   ', null, () => {});
    vi.advanceTimersByTime(URL_UPDATE_DEBOUNCE_MS);
    const calls = replaceSpy.mock.calls;
    const url = (calls[calls.length - 1] as any)[2] as string;
    expect(url).toBe('/viewer');
    vi.useRealTimers();
  });

  it('copies share URL with compressed code via clipboard', async () => {
    const writeSpy = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, { clipboard: { writeText: writeSpy } });
    const ok = await copyShareUrl('architecture "A" {}');
    expect(ok).toBe(true);
    expect(writeSpy).toHaveBeenCalled();
    const arg = writeSpy.mock.calls[0][0] as string;
    expect(arg.startsWith('https://example.com/viewer#code=')).toBe(true);
  });

  it('falls back to execCommand copy when clipboard write fails', async () => {
    const writeSpy = vi.fn().mockRejectedValue(new Error('no clipboard'));
    Object.assign(navigator, { clipboard: { writeText: writeSpy } });
    Object.defineProperty(document, 'execCommand', { value: vi.fn().mockReturnValue(true), configurable: true });
    const execSpy = vi.spyOn(document, 'execCommand');
    const ok = await copyShareUrl('architecture "A" {}');
    expect(writeSpy).toHaveBeenCalled();
    expect(execSpy).toHaveBeenCalledWith('copy');
    expect(ok).toBe(true);
  });
});
