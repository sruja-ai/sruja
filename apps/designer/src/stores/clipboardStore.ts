// apps/designer/src/stores/clipboardStore.ts
import { create } from "zustand";
import type { ElementDump } from "@sruja/shared";

/**
 * Clipboard data for copied nodes
 * Stores a flat list of elements representing the copied subtree.
 * The first element is the root of the copied subtree.
 */
interface ClipboardData {
  rootId: string;
  elements: ElementDump[]; // All elements in the copied subtree
}

interface ClipboardState {
  clipboard: ClipboardData | null;
  copyNode: (rootId: string, elements: ElementDump[]) => void;
  clearClipboard: () => void;
  hasClipboard: () => boolean;
}

export const useClipboardStore = create<ClipboardState>((set, get) => ({
  clipboard: null,

  copyNode: (rootId, elements) => {
    // Deep clone the data
    const clonedElements = JSON.parse(JSON.stringify(elements));
    set({
      clipboard: {
        rootId,
        elements: clonedElements,
      },
    });
  },

  clearClipboard: () => {
    set({ clipboard: null });
  },

  hasClipboard: () => {
    return get().clipboard !== null;
  },
}));
