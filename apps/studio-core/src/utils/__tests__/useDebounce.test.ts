import { describe, it, expect, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useDebounce } from '../useDebounce';

describe('useDebounce', () => {
  it('debounces value changes', () => {
    vi.useFakeTimers();

    const { result, rerender } = renderHook(({ v, d }) => useDebounce(v, d), {
      initialProps: { v: 'a', d: 300 },
    });

    expect(result.current).toBe('a');

    // Update value
    rerender({ v: 'b', d: 300 });
    // Still old value before timer
    expect(result.current).toBe('a');

    // Advance time
    act(() => {
      vi.advanceTimersByTime(300);
    });

    // Now debounced value updates
    expect(result.current).toBe('b');

    vi.useRealTimers();
  });
});
