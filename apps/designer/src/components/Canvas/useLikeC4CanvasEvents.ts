import { useEffect } from "react";
import type { LikeC4Model } from "@likec4/core/model";
import type { RefObject } from "react";
import type { ViewTab } from "../../types";

/**
 * Configuration for node click event handling.
 */
export interface NodeClickConfig {
  containerRef: RefObject<HTMLDivElement | null>;
  likec4Model: LikeC4Model<any> | null;
  currentLevel: string | null;
  onNodeClick?: (nodeId: string) => void;
  onDrillDown: (nodeId: string, type: "system" | "container", systemId?: string) => void;
  onSelectNode: (nodeId: string) => void;
  onSetActiveTab: (tab: ViewTab) => void;
}

/**
 * Hook for handling node click events in LikeC4Canvas.
 *
 * @param config - Configuration for node click handling
 *
 * @remarks
 * Returns click event handlers to:
 * - Handle navigation (drill down) based on current level
 * - Select nodes and show details panel
 */
export function useLikeC4Handlers(config: NodeClickConfig) {
  const { onNodeClick, onSelectNode } =
    config;

  const handleNodeClick = (node: { id: string } | string, event?: React.MouseEvent) => {
    // Extract node ID (it might be an object with id property or just a string)
    const nodeId = typeof node === "string" ? node : node.id;

    if (!nodeId) return;

    event?.stopPropagation();

    // Always select the node to show details panel
    onSelectNode(nodeId);
    // onSetActiveTab("details"); // Disabled: Details are now shown in side panel
    // console.log("[LikeC4Canvas] Node selected:", nodeId, "- showing related items in details panel");

    if (onNodeClick) {
      onNodeClick(nodeId);
    }
    // Default navigation disabled.
    // Navigation is handled via Side Panel / Left Nav.
  };

  return {
    handleNodeClick,
  };
}

/**
 * Configuration for keyboard event handling.
 */
export interface KeyboardConfig {
  activeFlow: any;
  isFlowPlaying: boolean;
  flowStep: number;
  onPlayFlow: () => void;
  onPauseFlow: () => void;
  onNextStep: () => void;
  onPrevStep: () => void;
  onSetFlowStep: (step: number) => void;
}

/**
 * Hook for handling keyboard shortcuts for animation controls.
 *
 * @param config - Configuration for keyboard handling
 *
 * @remarks
 * Handles keyboard shortcuts:
 * - Space: Play/pause
 * - ArrowLeft: Previous step
 * - ArrowRight: Next step
 * - Home: First step
 * - End: Last step
 */
export function useKeyboardShortcuts(config: KeyboardConfig): void {
  const {
    activeFlow,
    isFlowPlaying,
    flowStep,
    onPlayFlow,
    onPauseFlow,
    onNextStep,
    onPrevStep,
    onSetFlowStep,
  } = config;

  useEffect(() => {
    if (!activeFlow) {
      return;
    }

    const handleKeyDown = (event: KeyboardEvent) => {
      // Only handle if not typing in an input/textarea
      if (
        event.target instanceof HTMLInputElement ||
        event.target instanceof HTMLTextAreaElement ||
        (event.target as HTMLElement).isContentEditable
      ) {
        return;
      }

      switch (event.key) {
        case " ": // Space - play/pause
          event.preventDefault();
          if (isFlowPlaying) {
            onPauseFlow();
          } else {
            onPlayFlow();
          }
          break;
        case "ArrowLeft": // Previous step
          event.preventDefault();
          if (flowStep > 0) {
            onPrevStep();
          }
          break;
        case "ArrowRight": // Next step
          event.preventDefault();
          if (flowStep < (activeFlow.steps?.length ?? 0) - 1) {
            onNextStep();
          }
          break;
        case "Home": // Go to first step
          event.preventDefault();
          onSetFlowStep(0);
          break;
        case "End": {
          // Go to last step
          event.preventDefault();
          const totalSteps = activeFlow.steps?.length ?? 0;
          if (totalSteps > 0) {
            onSetFlowStep(totalSteps - 1);
          }
          break;
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [
    activeFlow,
    isFlowPlaying,
    flowStep,
    onPlayFlow,
    onPauseFlow,
    onNextStep,
    onPrevStep,
    onSetFlowStep,
  ]);
}
