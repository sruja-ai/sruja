// apps/designer/src/stores/uiSettingsStore.ts
import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";

interface UISettingsState {
  showMinimap: boolean;

  // Actions
  setShowMinimap: (show: boolean) => void;
  toggleMinimap: () => void;
}

const STORAGE_KEY = "sruja-ui-settings";

export const useUISettingsStore = create<UISettingsState>()(
  persist(
    (set) => ({
      showMinimap: true,

      setShowMinimap: (show) => set({ showMinimap: show }),
      toggleMinimap: () => set((state) => ({ showMinimap: !state.showMinimap })),
    }),
    {
      name: STORAGE_KEY,
      storage: createJSONStorage(() => localStorage),
    }
  )
);
