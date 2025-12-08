import { describe, it, expect, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useAutosave } from '../useAutosave';
import { persistence } from '../../utils/persistence';

describe('useAutosave', () => {
  it('debounces and saves to localStorage', async () => {
    vi.useFakeTimers();
    const spy = vi.spyOn(persistence, 'saveLocal').mockResolvedValue(undefined as any);
    const { rerender } = renderHook(({ d }) => useAutosave(d, 300), { initialProps: { d: 'a' } });
    rerender({ d: 'b' } as any);
    expect(spy).not.toHaveBeenCalled();
    await act(async () => { await vi.runAllTimersAsync(); });
    expect(spy).toHaveBeenCalledWith('b');
    vi.useRealTimers();
  });
});
