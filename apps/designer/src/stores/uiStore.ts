import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import type { ViewTab } from "../types";
import type { Role } from "../components/RoleSwitcher";

export type PendingActionType =
  | "create-requirement"
  | "create-adr"
  | "create-flow"
  | "create-scenario"
  | null;

export type CodeSubTab = "dsl" | "json" | "markdown";

// Mode-First Navigation: Create, Explore, Govern
export type NavigationMode = "create" | "explore" | "govern";

interface UIState {
  activeTab: ViewTab;
  setActiveTab: (tab: ViewTab) => void;

  // Mode-first navigation (replaces tab-switching paradigm)
  navigationMode: NavigationMode;
  setNavigationMode: (mode: NavigationMode) => void;

  // Beginner mode - kept for onboarding tour compatibility
  beginnerMode: boolean;
  setBeginnerMode: (enabled: boolean) => void;

  // Role view state
  selectedRole: Role;
  setSelectedRole: (role: Role) => void;

  // Code Panel state managed globally for deep linking
  codeTab: CodeSubTab;
  setCodeTab: (tab: CodeSubTab) => void;
  targetLine: number | null;
  setTargetLine: (line: number | null) => void;

  // Layout Visibility Controls
  isNavigationVisible: boolean;
  setIsNavigationVisible: (visible: boolean) => void;
  isInspectorVisible: boolean;
  setIsInspectorVisible: (visible: boolean) => void;

  // Pending action to execute after tab switch or component mount
  pendingAction: PendingActionType;
  setPendingAction: (action: PendingActionType) => void;
  clearPendingAction: () => void;
}

export const useUIStore = create<UIState>()(
  persist(
    (set) => ({
      activeTab: "diagram", // Default to diagram for new layout
      setActiveTab: (tab) => set({ activeTab: tab }),

      // Default to "create" mode for beginners
      navigationMode: "create",
      setNavigationMode: (mode) => set({ navigationMode: mode }),

      // Beginner mode enabled by default for new users
      beginnerMode: true,
      setBeginnerMode: (enabled) => set({ beginnerMode: enabled }),

      selectedRole: "architect", // Default to architect view
      setSelectedRole: (role) => set({ selectedRole: role }),

      codeTab: "dsl",
      setCodeTab: (tab) => set({ codeTab: tab }),
      targetLine: null,
      setTargetLine: (line) => set({ targetLine: line }),

      isNavigationVisible: true,
      setIsNavigationVisible: (visible) => set({ isNavigationVisible: visible }),
      isInspectorVisible: true,
      setIsInspectorVisible: (visible) => set({ isInspectorVisible: visible }),

      pendingAction: null,
      setPendingAction: (action) => set({ pendingAction: action }),
      clearPendingAction: () => set({ pendingAction: null }),
    }),
    {
      name: "sruja-ui-state",
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        selectedRole: state.selectedRole,
        beginnerMode: state.beginnerMode,
        navigationMode: state.navigationMode,
        isNavigationVisible: state.isNavigationVisible,
        isInspectorVisible: state.isInspectorVisible,
      }),
    }
  )
);
