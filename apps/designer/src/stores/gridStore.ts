// apps/designer/src/stores/gridStore.ts
import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";

export type GridSize = 8 | 16 | 32;

interface GridState {
  enabled: boolean;
  size: GridSize;
  snapToGrid: boolean;

  // Actions
  setEnabled: (enabled: boolean) => void;
  setSize: (size: GridSize) => void;
  setSnapToGrid: (snap: boolean) => void;
  toggleEnabled: () => void;
  toggleSnapToGrid: () => void;
}

const STORAGE_KEY = "sruja-grid-settings";

export const useGridStore = create<GridState>()(
  persist(
    (set) => ({
      enabled: false,
      size: 16,
      snapToGrid: false,

      setEnabled: (enabled) => set({ enabled }),
      setSize: (size) => set({ size }),
      setSnapToGrid: (snapToGrid) => set({ snapToGrid }),
      toggleEnabled: () => set((state) => ({ enabled: !state.enabled })),
      toggleSnapToGrid: () => set((state) => ({ snapToGrid: !state.snapToGrid })),
    }),
    {
      name: STORAGE_KEY,
      storage: createJSONStorage(() => localStorage),
    }
  )
);
