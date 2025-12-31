// apps/designer/src/hooks/useTagNavigation.ts
import { useCallback } from "react";
import { useUIStore } from "../stores/uiStore";
import { useSelectionStore } from "../stores/viewStore";
// import type { ArchitectureCanvasRef } from "../components/Canvas/types";

// Global ref to canvas - set by App component
let globalCanvasRef: React.RefObject<any> | null = null;

export function setGlobalCanvasRef(ref: React.RefObject<any>) {
  globalCanvasRef = ref;
}

/**
 * Hook for navigating to tagged elements in the diagram
 * Provides functionality to navigate from Details tab to Diagram tab
 * and focus on a specific element
 */
export function useTagNavigation() {
  const setActiveTab = useUIStore((s) => s.setActiveTab);
  const selectNode = useSelectionStore((s) => s.selectNode);

  const navigateToTaggedElement = useCallback(
    (tagId: string) => {
      // Switch to diagram tab
      setActiveTab("diagram");

      // Select the node
      selectNode(tagId);

      // Zoom to the node after a short delay to ensure it's rendered
      setTimeout(() => {
        if (globalCanvasRef?.current) {
          globalCanvasRef.current.focusNode(tagId);
        }
      }, 150);
    },
    [setActiveTab, selectNode]
  );

  return { navigateToTaggedElement };
}
