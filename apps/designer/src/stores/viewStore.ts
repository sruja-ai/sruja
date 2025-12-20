import { create } from "zustand";
import type { C4Level, ViewTab, ViewMode } from "../types";
import type { FlowDump } from "@sruja/shared";

interface ViewState {
  currentLevel: C4Level;
  focusedSystemId: string | null;
  focusedContainerId: string | null;
  expandedNodes: Set<string>;
  breadcrumb: string[];

  // Navigation actions
  setLevel: (level: C4Level) => void;
  drillDown: (nodeId: string, nodeType: "system" | "container", parentId?: string) => void;
  goUp: () => void;
  goToRoot: () => void;
  toggleExpand: (nodeId: string) => void;
}

export const useViewStore = create<ViewState>((set, get) => ({
  currentLevel: "L1",
  focusedSystemId: null,
  focusedContainerId: null,
  expandedNodes: new Set<string>(),
  breadcrumb: ["Architecture"],

  setLevel: (level) => {
    set({ currentLevel: level });
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
  activeFlow: FlowDump | null;
  flowStep: number;
  isFlowPlaying: boolean;

  activeRequirement: string | null;

  // Actions
  selectNode: (id: string | null) => void;
  /**
   * Set the active view (Diagram, Code, etc.)
   */
  setActiveTab: (tab: ViewTab) => void;
  /**
   * Set the view mode (Designer, Present)
   */
  setViewMode: (mode: ViewMode) => void;
  /**
   * Set the active flow for animation
   */
  setActiveFlow: (flow: FlowDump | null) => void;
  setActiveRequirement: (reqId: string | null) => void;
  setFlowStep: (step: number) => void;
  playFlow: () => void;
  pauseFlow: () => void;
  nextStep: () => void;
  prevStep: () => void;
}

export const useSelectionStore = create<SelectionState>((set, get) => ({
  selectedNodeId: null,
  activeFlow: null,
  activeRequirement: null,
  flowStep: 0,
  isFlowPlaying: false,
  activeTab: "Diagram", // Added default
  viewMode: "Designer", // Added default

  selectNode: (id) => {
    set({ selectedNodeId: id });
  },

  setActiveTab: (tab) => set((state) => ({ ...state, activeTab: tab })),
  setViewMode: (mode) => set((state) => ({ ...state, viewMode: mode })),

  setActiveFlow: (flow) => {
    set({ activeFlow: flow, isFlowPlaying: !!flow, flowStep: 0 });
  },

  setActiveRequirement: (reqId) => {
    set({ activeRequirement: reqId, selectedNodeId: null, activeFlow: null }); // Clear others
  },

  setFlowStep: (step) => {
    set({ flowStep: step });
  },

  playFlow: () => {
    set({ isFlowPlaying: true });
  },

  pauseFlow: () => {
    set({ isFlowPlaying: false });
  },

  nextStep: () => {
    const state = get();
    const maxStep = (state.activeFlow?.steps?.length ?? 1) - 1;
    set({ flowStep: Math.min(state.flowStep + 1, maxStep) });
  },

  prevStep: () => {
    const state = get();
    set({ flowStep: Math.max(state.flowStep - 1, 0) });
  },
}));
