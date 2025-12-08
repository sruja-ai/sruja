import { describe, it, expect, beforeEach } from 'vitest';
import { saveDslToStorage, loadDslFromStorage, savePaneToStorage, loadPaneFromStorage } from './storage';
import { STORAGE_KEYS } from '@/shared/constants/storage';

describe('viewer storage utils', () => {
  beforeEach(() => {
    localStorage.removeItem(STORAGE_KEYS.VIEWER_DSL);
    localStorage.removeItem(STORAGE_KEYS.VIEWER_PANE);
  });

  it('saves and loads DSL text', () => {
    saveDslToStorage('dsl');
    expect(loadDslFromStorage()).toBe('dsl');
  });

  it('saves only valid panes and maps legacy panes on load', () => {
    savePaneToStorage('diagram');
    expect(loadPaneFromStorage()).toBe('diagram');

    // invalid pane should be ignored
    savePaneToStorage('invalid' as any);
    expect(loadPaneFromStorage()).toBe('diagram');

    // legacy viewer → diagram
    localStorage.setItem(STORAGE_KEYS.VIEWER_PANE, 'viewer');
    expect(loadPaneFromStorage()).toBe('diagram');

    // legacy pdf → preview
    localStorage.setItem(STORAGE_KEYS.VIEWER_PANE, 'pdf');
    expect(loadPaneFromStorage()).toBe('preview');

    // default when missing
    localStorage.removeItem(STORAGE_KEYS.VIEWER_PANE);
    expect(loadPaneFromStorage()).toBe('split');
  });
});
