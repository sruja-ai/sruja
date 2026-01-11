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

export type CodeSubTab = "dsl" | "markdown";

// Mode-First Navigation: Create, Explore, Govern
export type NavigationMode = "create" | "explore" | "govern";

export type LayoutMode = "split" | "single";
export type LeftPaneContent = "builder" | "code" | "none";
export type RightPaneContent = "diagram" | "docs" | "none";

interface UIState {
  // Legacy support mapping
  activeTab: ViewTab;
  setActiveTab: (tab: ViewTab) => void;

  // New Split Controls
  activeEditor: "builder" | "code" | null;
  setActiveEditor: (editor: "builder" | "code" | null) => void;
  activeView: "diagram" | "docs" | "overview" | "details" | "roles" | null; // Null means nothing? Or always have a view? Usually Diagram.
  setActiveView: (view: "diagram" | "docs" | "overview" | "details" | "roles") => void;

  // Split View State
  layoutMode: LayoutMode;
  setLayoutMode: (mode: LayoutMode) => void;
  leftPaneContent: LeftPaneContent;
  setLeftPaneContent: (content: LeftPaneContent) => void;
  rightPaneContent: RightPaneContent;
  setRightPaneContent: (content: RightPaneContent) => void;

  // Builder State
  builderStep: string;
  setBuilderStep: (step: string) => void;

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
          // Toggle logic handled by caller or here? Let's make it direct setter.
          // Caller handles toggle.
          // Update leftPaneContent based on editor
          const left = editor === null ? "none" : editor;

          return {
            activeEditor: editor,
            leftPaneContent: left,
            // Sync activeTab for legacy
            activeTab: editor ? editor : (state.activeView as ViewTab),
          };
        }),

      activeView: "diagram",
      setActiveView: (view) =>
        set((state) => {
          const right = view === "diagram" ? "diagram" : view === "docs" ? "docs" : "diagram";

          return {
            activeView: view,
            rightPaneContent: right as RightPaneContent,
            // Sync activeTab for legacy
            activeTab: state.activeEditor ? state.activeEditor : (view as ViewTab),
          };
        }),

      activeTab: "diagram", // Default to diagram for new layout
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
          } else if (
            tab === "diagram" ||
            tab === "docs" ||
            tab === "overview" ||
            tab === "details" ||
            tab === "roles"
          ) {
            activeView = tab;
          }

          const left = activeEditor ? activeEditor : "none";
          // right layout logic
          let right: RightPaneContent = "diagram";
          if (activeView === "docs") right = "docs";

          return {
            activeTab: tab,
            activeEditor,
            activeView,
            leftPaneContent: left,
            rightPaneContent: right,
          };
        }),

      layoutMode: "split",
      setLayoutMode: (mode) => set({ layoutMode: mode }),

      leftPaneContent: "none",
      setLeftPaneContent: (content) => set({ leftPaneContent: content }),

      rightPaneContent: "diagram",
      setRightPaneContent: (content) => set({ rightPaneContent: content }),

      builderStep: "goals",
      setBuilderStep: (step) => set({ builderStep: step }),

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
        selectedRole: state.selectedRole,
        beginnerMode: state.beginnerMode,
        navigationMode: state.navigationMode,
        isNavigationVisible: state.isNavigationVisible,
        isInspectorVisible: state.isInspectorVisible,
        layoutMode: state.layoutMode,
        leftPaneContent: state.leftPaneContent,
        rightPaneContent: state.rightPaneContent,
      }),
    }
  )
);
