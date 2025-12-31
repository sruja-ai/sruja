import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import type { ViewTab } from "../types";
import type { Persona } from "../components/PersonaSwitcher";

export type PendingActionType =
  | "create-requirement"
  | "create-adr"
  | "create-flow"
  | "create-scenario"
  | null;

export type CodeSubTab = "dsl" | "json" | "markdown";

interface UIState {
  activeTab: ViewTab;
  setActiveTab: (tab: ViewTab) => void;

  // Persona view state
  selectedPersona: Persona;
  setSelectedPersona: (persona: Persona) => void;

  // Code Panel state managed globally for deep linking
  codeTab: CodeSubTab;
  setCodeTab: (tab: CodeSubTab) => void;
  targetLine: number | null;
  setTargetLine: (line: number | null) => void;

  // Pending action to execute after tab switch or component mount
  pendingAction: PendingActionType;
  setPendingAction: (action: PendingActionType) => void;
  clearPendingAction: () => void;
}

export const useUIStore = create<UIState>()(
  persist(
    (set) => ({
      activeTab: "overview", // Default will be set by App.tsx from URL
      setActiveTab: (tab) => set({ activeTab: tab }),

      selectedPersona: "architect", // Default to architect view
      setSelectedPersona: (persona) => set({ selectedPersona: persona }),

      codeTab: "dsl",
      setCodeTab: (tab) => set({ codeTab: tab }),
      targetLine: null,
      setTargetLine: (line) => set({ targetLine: line }),

      pendingAction: null,
      setPendingAction: (action) => set({ pendingAction: action }),
      clearPendingAction: () => set({ pendingAction: null }),
    }),
    {
      name: "sruja-ui-state",
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        selectedPersona: state.selectedPersona,
      }),
    }
  )
);
