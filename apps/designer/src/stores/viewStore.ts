import { create } from "zustand";
import type { C4Level, ViewTab, ViewMode } from "../types";
import type { FlowDump, ScenarioDump } from "@sruja/shared";

interface ViewState {
  currentLevel: C4Level;
  focusedSystemId: string | null;
  focusedContainerId: string | null;
  expandedNodes: Set<string>;
  breadcrumb: string[];
  viewportByContext: Record<string, { x: number; y: number; zoom: number }>;

  // Navigation actions
  setLevel: (level: C4Level) => void;
  drillDown: (nodeId: string, nodeType: "system" | "container", parentId?: string) => void;
  goUp: () => void;
  goToRoot: () => void;
  toggleExpand: (nodeId: string) => void;
  setViewportForContext: (
    contextKey: string,
    viewport: { x: number; y: number; zoom: number }
  ) => void;

  // View ID Navigation (for DSL Views)
  activeViewId: string | null;
  setActiveView: (viewId: string | null) => void;
}

export const useViewStore = create<ViewState>((set, get) => ({
  currentLevel: "L1",
  focusedSystemId: null,
  focusedContainerId: null,
  expandedNodes: new Set<string>(),
  breadcrumb: ["Architecture"],
  activeViewId: null,
  viewportByContext: {},

  setActiveView: (viewId) => {
    set({ activeViewId: viewId });
    // Reset breadcrumbs if going to a specific view?
    // Maybe keep them but mark as "View Mode"
  },

  setViewportForContext: (contextKey, viewport) => {
    set((state) => ({
      viewportByContext: {
        ...state.viewportByContext,
        [contextKey]: viewport,
      },
    }));
  },

  setLevel: (level) => {
    // Setting level clears active explicit view
    set({ activeViewId: null });

    // When setting level manually, ensure navigation state is consistent
    // L2 requires focusedSystemId, L3 requires both focusedSystemId and focusedContainerId
    const state = get();
    if (level === "L1") {
      // Going to L1 clears focused IDs
      set({
        currentLevel: level,
        focusedSystemId: null,
        focusedContainerId: null,
        breadcrumb: ["Architecture"],
      });
    } else if (level === "L2" && state.focusedSystemId) {
      // L2 requires focusedSystemId - keep it, clear focusedContainerId
      set({
        currentLevel: level,
        focusedContainerId: null,
        breadcrumb: ["Architecture", state.focusedSystemId],
      });
    } else if (level === "L3" && state.focusedSystemId && state.focusedContainerId) {
      // L3 requires both - keep them
      set({
        currentLevel: level,
        breadcrumb: ["Architecture", state.focusedSystemId, state.focusedContainerId],
      });
    } else {
      // Just update level if requirements not met (buttons should be disabled anyway)
      set({ currentLevel: level });
    }
  },

  drillDown: (nodeId, nodeType, parentId) => {
    const state = get();
    if (nodeType === "system") {
      const breadcrumb = ["Architecture", nodeId];
      // Idempotent update; avoid duplicate breadcrumb entries for same system
      set({
        currentLevel: "L2",
        focusedSystemId: nodeId,
        focusedContainerId: null,
        breadcrumb,
      });
    } else if (nodeType === "container") {
      const systemId = parentId ?? state.focusedSystemId ?? undefined;
      const breadcrumb = systemId ? ["Architecture", systemId, nodeId] : ["Architecture", nodeId];
      set({
        currentLevel: "L3",
        focusedContainerId: nodeId,
        breadcrumb,
      });
    }
  },

  goUp: () => {
    const state = get();
    const newBreadcrumb = [...state.breadcrumb];
    newBreadcrumb.pop();

    if (state.currentLevel === "L3") {
      set({
        currentLevel: "L2",
        focusedContainerId: null,
        breadcrumb: newBreadcrumb,
      });
    } else if (state.currentLevel === "L2") {
      set({
        currentLevel: "L1",
        focusedSystemId: null,
        breadcrumb: newBreadcrumb,
      });
    }
    // L1 is now root, no further up
  },

  goToRoot: () => {
    set({
      currentLevel: "L1",
      focusedSystemId: null,
      focusedContainerId: null,
      breadcrumb: ["Architecture"],
    });
  },

  toggleExpand: (nodeId) => {
    const state = get();
    const newExpanded = new Set(state.expandedNodes);
    if (newExpanded.has(nodeId)) {
      newExpanded.delete(nodeId);
    } else {
      newExpanded.add(nodeId);
    }
    set({ expandedNodes: newExpanded });
  },
}));

// Selection store for selected nodes and active flows
interface SelectionState {
  selectedNodeId: string | null;
  selectionSource: SelectionSource;
  activeAnimation: FlowDump | ScenarioDump | null;
  animationStep: number;
  isAnimationPlaying: boolean;

  activeRequirement: string | null;
  activeTab: ViewTab;
  viewMode: ViewMode;

  // Actions
  selectNode: (id: string | null, source?: SelectionSource) => void;
  /**
   * Set the active view (Diagram, Code, etc.)
   */
  setActiveTab: (tab: ViewTab) => void;
  /**
   * Set the view mode (Designer, Present)
   */
  setViewMode: (mode: ViewMode) => void;
  /**
   * Set the active animation (Flow or Scenario)
   */
  setActiveAnimation: (animation: FlowDump | ScenarioDump | null) => void;
  setActiveRequirement: (reqId: string | null) => void;
  setAnimationStep: (step: number) => void;
  playAnimation: () => void;
  pauseAnimation: () => void;
  nextStep: () => void;
  prevStep: () => void;
}

export type SelectionSource = "diagram" | "code" | "navigation" | "unknown";

export const useSelectionStore = create<SelectionState>((set, get) => ({
  selectedNodeId: null,
  selectionSource: "unknown",
  activeAnimation: null,
  activeRequirement: null,
  animationStep: 0,
  isAnimationPlaying: false,
  activeTab: "diagram", // Added default
  viewMode: "designer", // Added default

  selectNode: (id, source = "unknown") => {
    set({ selectedNodeId: id, selectionSource: source });
  },

  setActiveTab: (tab) => set((state) => ({ ...state, activeTab: tab })),
  setViewMode: (mode) => set((state) => ({ ...state, viewMode: mode })),

  setActiveAnimation: (animation) => {
    set({ activeAnimation: animation, isAnimationPlaying: !!animation, animationStep: 0 });
  },

  setActiveRequirement: (reqId) => {
    set({
      activeRequirement: reqId,
      selectedNodeId: null,
      selectionSource: "unknown",
      activeAnimation: null,
    }); // Clear others
  },

  setAnimationStep: (step) => {
    set({ animationStep: step });
  },

  playAnimation: () => {
    set({ isAnimationPlaying: true });
  },

  pauseAnimation: () => {
    set({ isAnimationPlaying: false });
  },

  nextStep: () => {
    const state = get();
    const maxStep = (state.activeAnimation?.steps?.length ?? 1) - 1;
    set({ animationStep: Math.min(state.animationStep + 1, maxStep) });
  },

  prevStep: () => {
    const state = get();
    set({ animationStep: Math.max(state.animationStep - 1, 0) });
  },
}));
