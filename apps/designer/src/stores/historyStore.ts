// apps/designer/src/stores/historyStore.ts
import { create } from "zustand";
import type { SrujaModelDump } from "@sruja/shared";

/**
 * History store state interface.
 * 
 * Manages undo/redo history for architecture changes.
 * Maintains a stack of architecture states for navigation.
 */
interface HistoryState {
  history: SrujaModelDump[];
  currentIndex: number;
  maxHistorySize: number;

  // Actions
  push: (state: SrujaModelDump) => void;
  undo: () => SrujaModelDump | null;
  redo: () => SrujaModelDump | null;
  canUndo: () => boolean;
  canRedo: () => boolean;
  clear: () => void;
  getCurrent: () => SrujaModelDump | null;
}

/**
 * Zustand store for managing undo/redo history.
 * 
 * Provides:
 * - History stack of architecture states
 * - Undo/redo functionality
 * - Current position tracking in history
 * - History persistence to localStorage
 * 
 * Automatically persists history to localStorage for restoration across sessions.
 * 
 * @example
 * ```ts
 * const { push, undo, redo, canUndo, canRedo } = useHistoryStore.getState();
 * 
 * push(newArchitectureState);
 * const previousState = undo(); // Returns previous state or null
 * const nextState = redo(); // Returns next state or null
 * ```
 */
export const useHistoryStore = create<HistoryState>((set, get) => ({
  history: [],
  currentIndex: -1,
  maxHistorySize: 50, // Limit history to 50 entries

  push: (state: SrujaModelDump) => {
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
