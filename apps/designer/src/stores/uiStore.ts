import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import type { ViewTab } from "../types";

export type PendingActionType =
  | "create-requirement"
  | "create-adr"
  | "create-flow"
  | "create-scenario"
  | null;

export type CodeSubTab = "dsl" | "markdown";

interface UIState {
  // Legacy support mapping
  activeTab: ViewTab;
  setActiveTab: (tab: ViewTab) => void;

  // New Split Controls
  activeEditor: "builder" | "code" | null;
  setActiveEditor: (editor: "builder" | "code" | null) => void;
  activeView: "diagram" | "docs" | "review";
  setActiveView: (view: "diagram" | "docs" | "review") => void;

  // Builder State
  builderStep: string;
  setBuilderStep: (step: string) => void;

  // Beginner mode - kept for onboarding tour compatibility
  beginnerMode: boolean;
  setBeginnerMode: (enabled: boolean) => void;

  // Code Panel state managed globally for deep linking
  codeTab: CodeSubTab;
  setCodeTab: (tab: CodeSubTab) => void;
  targetLine: number | null;
  setTargetLine: (line: number | null) => void;

  // Layout Visibility Controls
  isNavigationVisible: boolean;
  setIsNavigationVisible: (visible: boolean) => void;
  toggleNavigation: () => void;
  isInspectorVisible: boolean;
  setIsInspectorVisible: (visible: boolean) => void;
  toggleInspector: () => void;

  // Pending action to execute after tab switch or component mount
  pendingAction: PendingActionType;
  setPendingAction: (action: PendingActionType) => void;
  clearPendingAction: () => void;
}

export const useUIStore = create<UIState>()(
  persist(
    (set) => ({
      activeEditor: null,
      setActiveEditor: (editor) =>
        set((state) => {
          return {
            activeEditor: editor,
            // Sync activeTab for legacy
            activeTab: editor ? editor : (state.activeView as ViewTab),
          };
        }),

      activeView: "diagram",
      setActiveView: (view) =>
        set((state) => ({
          activeView: view,
          activeTab: state.activeEditor ? state.activeEditor : (view as ViewTab),
        })),

      activeTab: "diagram",
      setActiveTab: (tab) =>
        set((state) => {
          // Map legacy tabs to split view state
          let activeEditor = state.activeEditor;
          let activeView = state.activeView;

          if (tab === "builder" || tab === "code") {
            // Toggle logic
            if (activeEditor === tab) {
              activeEditor = null;
            } else {
              activeEditor = tab;
            }
          } else if (tab === "diagram" || tab === "docs" || tab === "review") {
            activeView = tab;
          }

          return {
            activeTab: tab,
            activeEditor,
            activeView,
          };
        }),

      builderStep: "goals",
      setBuilderStep: (step) => set({ builderStep: step }),

      // Beginner mode enabled by default for new users
      beginnerMode: true,
      setBeginnerMode: (enabled) => set({ beginnerMode: enabled }),

      codeTab: "dsl",
      setCodeTab: (tab) => set({ codeTab: tab }),
      targetLine: null,
      setTargetLine: (line) => set({ targetLine: line }),

      isNavigationVisible: true,
      setIsNavigationVisible: (visible) => set({ isNavigationVisible: visible }),
      toggleNavigation: () => set((state) => ({ isNavigationVisible: !state.isNavigationVisible })),

      isInspectorVisible: true,
      setIsInspectorVisible: (visible) => set({ isInspectorVisible: visible }),
      toggleInspector: () => set((state) => ({ isInspectorVisible: !state.isInspectorVisible })),

      pendingAction: null,
      setPendingAction: (action) => set({ pendingAction: action }),
      clearPendingAction: () => set({ pendingAction: null }),
    }),
    {
      name: "sruja-ui-state",
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        beginnerMode: state.beginnerMode,
        isNavigationVisible: state.isNavigationVisible,
        isInspectorVisible: state.isInspectorVisible,
        // Persist view/editor preferences so the app stays in the last-used layout.
        activeEditor: state.activeEditor,
        activeView: state.activeView,
      }),
    }
  )
);
