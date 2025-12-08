// apps/website/src/features/viewer/utils/storage.ts
import { createStringStorage } from '@/shared/utils/storage';
import { STORAGE_KEYS } from '@/shared/constants/storage';
import type { PaneType } from '../types';

const dslStorage = createStringStorage(STORAGE_KEYS.VIEWER_DSL, '');
const paneStorage = createStringStorage(STORAGE_KEYS.VIEWER_PANE, 'split');

export const saveDslToStorage = (dslText: string): void => {
  dslStorage.set(dslText);
};

export const loadDslFromStorage = (): string => {
  return dslStorage.get();
};

export const savePaneToStorage = (pane: PaneType): void => {
  const validPanes: PaneType[] = ['split', 'editor', 'diagram', 'json', 'preview', 'markdown'];
  if (validPanes.includes(pane)) {
    paneStorage.set(pane);
  }
};

export const loadPaneFromStorage = (): PaneType => {
  const saved = paneStorage.get();
  const validPanes: PaneType[] = ['split', 'editor', 'diagram', 'json', 'preview', 'markdown'];
  if (saved) {
    if (saved === 'viewer') {
      return 'diagram';
    }
    if (saved === 'pdf') {
      return 'preview';
    }
    if (validPanes.includes(saved as PaneType)) {
      return saved as PaneType;
    }
  }
  return 'split';
};
