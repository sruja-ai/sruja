// apps/website/src/shared/hooks/useLocalStorage.test.ts
import { describe, it, expect, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useLocalStorage } from './useLocalStorage';

describe('useLocalStorage', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('returns default value when localStorage is empty', () => {
    const { result } = renderHook(() => useLocalStorage('test-key', 'default'));
    expect(result.current[0]).toBe('default');
  });

  it('stores and retrieves values', () => {
    const { result } = renderHook(() => useLocalStorage('test-key', 'default'));

    act(() => {
      result.current[1]('new-value');
    });

    expect(result.current[0]).toBe('new-value');
    expect(localStorage.getItem('test-key')).toBe('"new-value"');
  });

  it('supports functional updates', () => {
    const { result } = renderHook(() => useLocalStorage('counter', 0));

    act(() => {
      result.current[1]((prev) => prev + 1);
    });

    expect(result.current[0]).toBe(1);
  });

  it('removes value correctly', () => {
    const { result } = renderHook(() => useLocalStorage('test-key', 'default'));

    act(() => {
      result.current[1]('value');
    });

    expect(result.current[0]).toBe('value');

    act(() => {
      result.current[2](); // removeValue
    });

    expect(result.current[0]).toBe('default');
    expect(localStorage.getItem('test-key')).toBeNull();
  });
});
