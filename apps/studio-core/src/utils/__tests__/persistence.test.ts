import { describe, it, expect, vi, beforeEach } from 'vitest';
import { persistence } from '../persistence';

const KEY = 'sruja_studio_dsl_autosave';

describe('persistence', () => {
  beforeEach(() => {
    // Reset localStorage state
    localStorage.clear();
  });

  it('saves DSL to localStorage', async () => {
    const setSpy = vi.spyOn(window.localStorage.__proto__, 'setItem');
    await persistence.saveLocal('dsl');
    expect(setSpy).toHaveBeenCalledWith(KEY, 'dsl');
    expect(localStorage.getItem(KEY)).toBe('dsl');
  });

  it('loads DSL from localStorage', async () => {
    localStorage.setItem(KEY, 'value');
    const loaded = await persistence.loadLocal();
    expect(loaded).toBe('value');
  });

  it('clears DSL from localStorage', async () => {
    localStorage.setItem(KEY, 'value');
    const removeSpy = vi.spyOn(window.localStorage.__proto__, 'removeItem');
    await persistence.clearLocal();
    expect(removeSpy).toHaveBeenCalledWith(KEY);
    expect(localStorage.getItem(KEY)).toBeNull();
  });
});
