import { describe, it, expect } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useModalState } from '../useModalState';

describe('useModalState', () => {
  it('initializes and toggles modal states', () => {
    const { result } = renderHook(() => useModalState());
    expect(result.current.modalConfig.isOpen).toBe(false);
    act(() => result.current.setAdrModalOpen(true));
    expect(result.current.adrModalOpen).toBe(true);
    act(() => result.current.setModalConfig({ isOpen: true, title: 'Add', type: 'node' } as any));
    expect(result.current.modalConfig.isOpen).toBe(true);
    expect(result.current.modalConfig.title).toBe('Add');
  });
});
