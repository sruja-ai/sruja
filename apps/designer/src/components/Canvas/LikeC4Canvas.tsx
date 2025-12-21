import { useRef, useCallback, useEffect } from "react";
import { forwardRef } from "react";
import { LikeC4ModelProvider, LikeC4View } from "@likec4/diagram/bundle";
import { MantineProvider } from "@sruja/ui";
import { useArchitectureStore } from "../../stores";
import { useViewStore } from "../../stores/viewStore";
import { useSelectionStore } from "../../stores/viewStore";
import { useUIStore } from "../../stores";
import { AnimatePresence, motion } from "framer-motion";
import { applyNodeColors } from "./likeC4NodeStyler";
import {
  useLikeC4Model,
  useAvailableViews,
  useActiveViewId,
  useNodeColorApplication,
} from "./useLikeC4Canvas";
import { useLikeC4CanvasExports } from "./useLikeC4CanvasExports";
import { useLikeC4Handlers, useKeyboardShortcuts } from "./useLikeC4CanvasEvents";
import { useLikeC4Animation } from "./useLikeC4Animation";
import { AnimationControls } from "./AnimationControls";
import { CanvasEmptyMessage } from "./components/CanvasEmptyMessage";
import { CanvasLoadingState } from "./components/CanvasLoadingState";
import { CanvasErrorState } from "./components/CanvasErrorState";
import type { ArchitectureCanvasRef, LikeC4CanvasProps } from "./types";
import "./LikeC4Canvas.css";
import "./LikeC4Canvas.animation.css";
import "./LikeC4Canvas.node-colors.css";

/**
 * LikeC4-based diagram canvas component
 *
 * Replaces ArchitectureCanvas with LikeC4 rendering. Uses @likec4/diagram
 * for diagram rendering instead of ReactFlow.
 *
 * @remarks
 * This component orchestrates:
 * - Model conversion and computation
 * - View management and selection
 * - Node styling and color application
 * - Animation controls
 * - Export functionality
 * - Event handling (clicks, keyboard)
 */
export const LikeC4Canvas = forwardRef<ArchitectureCanvasRef, LikeC4CanvasProps>(
  (
    {
      model: propModel,
      dragEnabled: _dragEnabled = true,
      viewId,
      onNodeClick,
      onEdgeClick: _onEdgeClick,
      onCanvasClick: _onCanvasClick,
    },
    ref
  ) => {
    const containerRef = useRef<HTMLDivElement>(null);
    const storeLikec4Model = useArchitectureStore((s) => s.likec4Model);
    const currentLevel = useViewStore((s) => s.currentLevel);
    const focusedSystemId = useViewStore((s) => s.focusedSystemId);
    const focusedContainerId = useViewStore((s) => s.focusedContainerId);
    const drillDown = useViewStore((s) => s.drillDown);
    const selectNode = useSelectionStore((s) => s.selectNode);
    const selectedNodeId = useSelectionStore((s) => s.selectedNodeId);
    const setActiveTab = useUIStore((s) => s.setActiveTab);

    // Animation engine state
    const activeFlow = useSelectionStore((s) => s.activeFlow);
    const flowStep = useSelectionStore((s) => s.flowStep);
    const isFlowPlaying = useSelectionStore((s) => s.isFlowPlaying);
    const playFlow = useSelectionStore((s) => s.playFlow);
    const pauseFlow = useSelectionStore((s) => s.pauseFlow);
    const nextStep = useSelectionStore((s) => s.nextStep);
    const prevStep = useSelectionStore((s) => s.prevStep);
    const setFlowStep = useSelectionStore((s) => s.setFlowStep);

    // Use model from prop (priority) or store
    const model = propModel || storeLikec4Model;

    // Model conversion and computation
    const {
      model: likec4Model,
      error: conversionError,
      isComputing,
    } = useLikeC4Model(model, focusedSystemId, focusedContainerId);

    // View management
    const availableViews = useAvailableViews(likec4Model);
    const { activeViewId } = useActiveViewId(availableViews, viewId, currentLevel);

    // Node color application
    const applyColors = useCallback(() => {
      applyNodeColors({
        containerRef,
        likec4Model,
        selectedNodeId,
      });
    }, [likec4Model, selectedNodeId]);

    useNodeColorApplication(containerRef, likec4Model, activeViewId, selectedNodeId, applyColors);

    // Export functionality
    useLikeC4CanvasExports({ containerRef, model }, ref);

    // Event handlers
    const { handleNodeClick } = useLikeC4Handlers({
      containerRef,
      likec4Model,
      currentLevel,
      onNodeClick,
      onDrillDown: drillDown,
      onSelectNode: selectNode,
      onSetActiveTab: setActiveTab,
    });

    useKeyboardShortcuts({
      activeFlow,
      isFlowPlaying,
      flowStep,
      onPlayFlow: playFlow,
      onPauseFlow: pauseFlow,
      onNextStep: nextStep,
      onPrevStep: prevStep,
      onSetFlowStep: setFlowStep,
    });

    // Initialize animation engine
    useLikeC4Animation({
      containerRef,
      activeFlow,
      isFlowPlaying,
      flowStep,
    });

    // Handle node clicks via event delegation since LikeC4View doesn't support onNodeClick prop
    useEffect(() => {
      if (!containerRef.current || !handleNodeClick) {
        return;
      }

      const container = containerRef.current;

      const handleClick = (event: MouseEvent) => {
        // Find the clicked element that represents a node
        const target = event.target as HTMLElement;

        // Look for data attributes that LikeC4 uses to identify nodes
        // LikeC4 typically uses data-element-id or similar attributes
        let nodeElement: HTMLElement | null = target;
        let nodeId: string | null = null;

        // Walk up the DOM tree to find the node element
        while (nodeElement && nodeElement !== container) {
          // Check for various data attributes that might contain the node ID
          nodeId =
            nodeElement.getAttribute("data-element-id") ||
            nodeElement.getAttribute("data-id") ||
            nodeElement.getAttribute("id") ||
            nodeElement.closest("[data-element-id]")?.getAttribute("data-element-id") ||
            nodeElement.closest("[data-id]")?.getAttribute("data-id") ||
            null;

          if (nodeId) {
            break;
          }

          nodeElement = nodeElement.parentElement;
        }

        if (nodeId && likec4Model) {
          // Verify the node exists in the model
          try {
            const element = likec4Model.element(nodeId);
            if (element) {
              handleNodeClick(nodeId, event as any);
            }
          } catch {
            // Node not found, ignore click
          }
        }
      };

      container.addEventListener("click", handleClick);
      return () => {
        container.removeEventListener("click", handleClick);
      };
    }, [containerRef, handleNodeClick, likec4Model]);

    /* Debug: Check if SVG is rendered - Disabled for production
    useEffect(() => {
      if (activeViewId && containerRef.current) {
        const checkSVG = () => {
          const svg = containerRef.current?.querySelector('svg');
          if (svg) {
            // SVG found
          }
        };
        const timeout1 = setTimeout(checkSVG, 1000);
        const timeout2 = setTimeout(checkSVG, 2000);
        return () => {
          clearTimeout(timeout1);
          clearTimeout(timeout2);
        };
      }
    }, [activeViewId]);
    */

    // Render empty states
    if (!model) {
      return (
        <CanvasEmptyMessage title="No diagram data available">
          The DSL may contain syntax errors or the model failed to load.
        </CanvasEmptyMessage>
      );
    }

    const hasViewsInModel = model?.views && Object.keys(model.views).length > 0;
    const hasViewsInLikeC4Model = likec4Model ? [...likec4Model.views()].length > 0 : false;

    if (isComputing) {
      return (
        <CanvasEmptyMessage title="Computing and laying out views...">
          This may take a moment for complex models
        </CanvasEmptyMessage>
      );
    }

    if (availableViews.length === 0 && !conversionError) {
      // Only warn if model has views but LikeC4Model filtered them all out
      /*
      if (hasViewsInModel && !hasViewsInLikeC4Model) {
        console.warn("[LikeC4Canvas] Model has views but LikeC4Model filtered them out. This might indicate invalid view rules or missing elements.");
      }
      */
      return (
        <CanvasEmptyMessage title="No diagrams found">
          The model is valid but contains no views. Add a <code>views</code> block to your DSL.
          {hasViewsInModel &&
            !hasViewsInLikeC4Model &&
            " Views exist in the model but were filtered out by LikeC4Model - check view rules and element IDs."}
        </CanvasEmptyMessage>
      );
    }

    /*
    if (activeViewId && likec4Model) {
      console.log('[LikeC4Canvas] Rendering view:', {
        viewId: activeViewId,
        hasModel: !!likec4Model,
        availableViews: [...likec4Model.views()].map(v => v.id)
      });
    }
    */

    return (
      <div ref={containerRef} className="likec4-canvas" data-testid="likec4-canvas">
        {/* LikeC4 diagram with AnimatePresence for transitions */}
        <div
          className="likec4-diagram-container"
          style={{
            width: "100%",
            height: "100%",
            position: "relative",
            display: "flex",
            flexDirection: "column",
          }}
        >
          {activeViewId && likec4Model ? (
            <MantineProvider>
              <LikeC4ModelProvider likec4model={likec4Model}>
                <div
                  key={activeViewId}
                  style={{ width: "100%", height: "100%", flex: 1, minHeight: 0 }}
                >
                  <LikeC4View key={activeViewId} viewId={activeViewId} />
                </div>
              </LikeC4ModelProvider>
            </MantineProvider>
          ) : conversionError ? (
            <CanvasErrorState error={conversionError} />
          ) : (
            <CanvasLoadingState />
          )}
          {/* Animation Controls overlay */}
          <AnimatePresence>
            {activeFlow && (
              <motion.div
                className="animation-controls-wrapper"
                initial={{ y: 50, opacity: 0, x: "-50%" }}
                animate={{ y: 0, opacity: 1, x: "-50%" }}
                exit={{ y: 50, opacity: 0, x: "-50%" }}
                transition={{ type: "spring", damping: 20, stiffness: 300 }}
                style={{
                  position: "absolute",
                  bottom: 16,
                  left: "50%",
                  zIndex: 100,
                  pointerEvents: "none",
                }}
              >
                <div style={{ pointerEvents: "auto" }}>
                  <AnimationControls flow={activeFlow} />
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </div>
    );
  }
);

LikeC4Canvas.displayName = "LikeC4Canvas";

// Re-export types for convenience
export type { ArchitectureCanvasRef, LikeC4CanvasProps } from "./types";
