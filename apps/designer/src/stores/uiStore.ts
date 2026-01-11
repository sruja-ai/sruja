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
      activeTab: "diagram", // Default to diagram for new layout
      setActiveTab: (tab) =>
        set((state) => {
          // Map legacy tabs to split view state
          let left: LeftPaneContent = state.leftPaneContent;
          let right: RightPaneContent = "diagram";

          if (tab === "builder") {
            // Toggle logic: if already on builder tab, toggle the pane
            if (state.activeTab === "builder") {
              left = state.leftPaneContent === "builder" ? "none" : "builder";
            } else {
              left = "builder";
            }
          } else if (tab === "code") {
            // Toggle logic for code tab
            if (state.activeTab === "code") {
              left = state.leftPaneContent === "code" ? "none" : "code";
            } else {
              left = "code";
            }
          } else if (tab === "docs") {
            right = "docs";
            left = "none"; // Fullscreen docs usually
          } else if (tab === "diagram") {
            // Toggle logic for diagram tab (maybe user wants to toggle right pane?
            // but diagram is usually the main view.
            // For now, let's keep it simple: reset left pane.
            // Or wait, if we are in builder mode and click diagram, we probably just want to switch right pane to diagram?
            // The request was about "builder, properties, diagram".
            // If I am in builder, and I click builder, it collapses.

            // If I am in Builder tab (activeTab=builder), left=builder, right=diagram.
            // If I click Diagram tab, activeTab becomes diagram. left=none, right=diagram.

            left = "none";
            right = "diagram";
          } else if (tab === "overview" || tab === "details" || tab === "roles") {
            // These might need specific handling or be treated as "overlays" or just "diagram" area content
            // For now, let's treat them as full right-pane content or legacy full screen
            left = "none";
            // We might need to extend RightPaneContent if we want them in split view
          }

          return {
            activeTab: tab,
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
