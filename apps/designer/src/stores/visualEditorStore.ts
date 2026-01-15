// apps/designer/src/stores/visualEditorStore.ts
import { create } from "zustand";

/**
 * Visual editing tool types
 */
export type VisualEditorTool = "select" | "create-node" | "connect" | null;

/**
 * Sruja semantic node types that can be created visually
 */
export type SrujaNodeType = "person" | "system" | "container" | "component" | "datastore" | "queue";

/**
 * Visual editor store state
 */
interface VisualEditorState {
  // Active tool
  activeTool: VisualEditorTool;
  // Selected node type for creation (when activeTool is "create-node")
  selectedNodeType: SrujaNodeType | null;

  // Manual mode (move nodes manually vs auto-layout)
  isManualMode: boolean;

  // Actions
  setActiveTool: (tool: VisualEditorTool) => void;
  setSelectedNodeType: (type: SrujaNodeType | null) => void;
  setManualMode: (enabled: boolean) => void;
  reset: () => void;
}

/**
 * Zustand store for visual editor state
 */
export const useVisualEditorStore = create<VisualEditorState>((set) => ({
  activeTool: "select",
  selectedNodeType: null,
  isManualMode: false,

  setActiveTool: (tool) => {
    set({ activeTool: tool });
    // Reset selected node type when switching away from create-node
    if (tool !== "create-node") {
      set({ selectedNodeType: null });
    }
  },

  setSelectedNodeType: (type) => {
    set({ selectedNodeType: type, activeTool: "create-node" });
  },

  setManualMode: (enabled) => {
    set({ isManualMode: enabled });
  },

  reset: () => {
    set({ activeTool: "select", selectedNodeType: null, isManualMode: false });
  },
}));
