import { describe, it, expect } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useDeepLinking } from '../useDeepLinking';

describe('useDeepLinking', () => {
  it('syncs state to URL and hydrates from URL', () => {
    const setStep = (v: any) => (window.__step = v);
    const setFocusPath = (v: string[]) => (window.__focus = v);
    const { rerender } = renderHook(({ step, focus }) => useDeepLinking({ activeStep: step, focusPath: focus, setStep, setFocusPath }), {
      initialProps: { step: null as any, focus: [] as any },
    });
    rerender({ step: 'define', focus: ['Sys', 'Web'] } as any);
    expect(window.location.search.includes('step=define')).toBe(true);
    expect(window.location.search.includes('focus=Sys.Web')).toBe(true);
    renderHook(() => useDeepLinking({ activeStep: null, focusPath: [], setStep, setFocusPath }));
    expect((window as any).__step).toBe('define');
    expect((window as any).__focus).toEqual(['Sys', 'Web']);
  });
});
