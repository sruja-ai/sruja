// apps/playground/src/stores/historyStore.ts
import { create } from "zustand";
import type { ArchitectureJSON } from "../types";

interface HistoryState {
  history: ArchitectureJSON[];
  currentIndex: number;
  maxHistorySize: number;

  // Actions
  push: (state: ArchitectureJSON) => void;
  undo: () => ArchitectureJSON | null;
  redo: () => ArchitectureJSON | null;
  canUndo: () => boolean;
  canRedo: () => boolean;
  clear: () => void;
  getCurrent: () => ArchitectureJSON | null;
}

export const useHistoryStore = create<HistoryState>((set, get) => ({
  history: [],
  currentIndex: -1,
  maxHistorySize: 50, // Limit history to 50 entries

  push: (state: ArchitectureJSON) => {
    const { history, currentIndex, maxHistorySize } = get();

    // If we're not at the end of history, remove everything after currentIndex
    const newHistory = history.slice(0, currentIndex + 1);

    // Add new state
    newHistory.push(JSON.parse(JSON.stringify(state))); // Deep clone

    // Limit history size
    if (newHistory.length > maxHistorySize) {
      newHistory.shift(); // Remove oldest entry
      // Keep currentIndex at maxHistorySize - 1
      set({ history: newHistory, currentIndex: maxHistorySize - 1 });
    } else {
      set({ history: newHistory, currentIndex: newHistory.length - 1 });
    }
  },

  undo: () => {
    const { history, currentIndex } = get();
    if (currentIndex > 0) {
      const newIndex = currentIndex - 1;
      set({ currentIndex: newIndex });
      return JSON.parse(JSON.stringify(history[newIndex])); // Deep clone
    }
    return null;
  },

  redo: () => {
    const { history, currentIndex } = get();
    if (currentIndex < history.length - 1) {
      const newIndex = currentIndex + 1;
      set({ currentIndex: newIndex });
      return JSON.parse(JSON.stringify(history[newIndex])); // Deep clone
    }
    return null;
  },

  canUndo: () => {
    return get().currentIndex > 0;
  },

  canRedo: () => {
    const { history, currentIndex } = get();
    return currentIndex < history.length - 1;
  },

  clear: () => {
    set({ history: [], currentIndex: -1 });
  },

  getCurrent: () => {
    const { history, currentIndex } = get();
    if (currentIndex >= 0 && currentIndex < history.length) {
      return JSON.parse(JSON.stringify(history[currentIndex])); // Deep clone
    }
    return null;
  },
}));
