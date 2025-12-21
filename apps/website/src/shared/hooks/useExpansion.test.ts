// apps/website/src/shared/hooks/useExpansion.test.ts
import { describe, it, expect } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useExpansion } from './useExpansion';

describe('useExpansion', () => {
  it('initializes with empty set by default', () => {
    const { result } = renderHook(() => useExpansion());
    expect(result.current.expanded.size).toBe(0);
  });

  it('initializes with provided items', () => {
    const { result } = renderHook(() => useExpansion(['item1', 'item2']));
    expect(result.current.isExpanded('item1')).toBe(true);
    expect(result.current.isExpanded('item2')).toBe(true);
  });

  it('toggles item expansion', () => {
    const { result } = renderHook(() => useExpansion());

    act(() => {
      result.current.toggle('item1');
    });

    expect(result.current.isExpanded('item1')).toBe(true);

    act(() => {
      result.current.toggle('item1');
    });

    expect(result.current.isExpanded('item1')).toBe(false);
  });

  it('expands specific item', () => {
    const { result } = renderHook(() => useExpansion());

    act(() => {
      result.current.expand('item1');
    });

    expect(result.current.isExpanded('item1')).toBe(true);
  });

  it('collapses specific item', () => {
    const { result } = renderHook(() => useExpansion(['item1']));

    act(() => {
      result.current.collapse('item1');
    });

    expect(result.current.isExpanded('item1')).toBe(false);
  });

  it('expands all items', () => {
    const { result } = renderHook(() => useExpansion());

    act(() => {
      result.current.expandAll(['item1', 'item2', 'item3']);
    });

    expect(result.current.isExpanded('item1')).toBe(true);
    expect(result.current.isExpanded('item2')).toBe(true);
    expect(result.current.isExpanded('item3')).toBe(true);
  });

  it('collapses all items', () => {
    const { result } = renderHook(() => useExpansion(['item1', 'item2']));

    act(() => {
      result.current.collapseAll();
    });

    expect(result.current.isExpanded('item1')).toBe(false);
    expect(result.current.isExpanded('item2')).toBe(false);
  });
});
