import { describe, it, expect } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useUIState } from '../useUIState';

describe('useUIState', () => {
  it('initializes with localStorage-based welcome flag and toggles', () => {
    localStorage.setItem('studio-hide-welcome', 'true');
    const { result } = renderHook(() => useUIState());
    expect(result.current.showWelcome).toBe(false);
    act(() => result.current.setShowWelcome(true));
    expect(result.current.showWelcome).toBe(true);
    act(() => result.current.setActiveView('viewer'));
    expect(result.current.activeView).toBe('viewer');
  });
});
