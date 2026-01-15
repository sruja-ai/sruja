import React, { useState, useEffect, useCallback, useMemo, useRef } from "react";
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  type Node as RFNode,
  type Edge as RFEdge,
  type NodeTypes,
  type EdgeTypes,
  type ReactFlowInstance,
  type Node as FlowNode,
  MarkerType,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { Paper, Group, Stack, ActionIcon, Text, Loader, Button, Badge } from "@mantine/core";
import { useTheme } from "@sruja/ui";

import { useArchitectureStore, useSelectionStore, useVisualEditorStore } from "../../stores";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import { trackInteraction, logger } from "@sruja/shared";
import { runGraphviz, GraphvizLayoutError } from "./layoutEngine";
import { useToastStore } from "../../stores/toastStore";
import { handleError } from "../../utils/errorHandling";
import { measureQuality, measureQualityFromNodes } from "./qualityMetrics";
import { SrujaNode } from "./SrujaNode";
import { GroupNode } from "../Nodes/GroupNode";
import { buildCompoundNodeStructure } from "./compoundNodes";
import type { C4Node, C4Level } from "./types";
import type { C4NodeData } from "../../types";
import { ArrowLeft, Edit3, Zap } from "lucide-react";
import { type LayoutQuality, type ParentChildRelationships } from "./qualityMetrics";

import { convertDslToDot, type SrujaModelDump } from "@sruja/shared";
type ElementDump = NonNullable<SrujaModelDump["elements"]>[string];
import type { EdgeType } from "./types";
import SplineEdge from "./SplineEdge";
import TrafficEdge from "./TrafficEdge";
import { useViewStore } from "../../stores/viewStore"; // Ensure view store is imported
import { AnimationController } from "../../utils/animation/AnimationController";
import { VisualEffectsSystem } from "../../utils/animation/VisualEffectsSystem";
import { AnimationControls } from "../Canvas/AnimationControls";
import { StepDescriptionOverlay } from "../Canvas/StepDescriptionOverlay";
import type { Connection } from "@xyflow/react";
import { VisualEditorToolbar } from "../VisualEditor/VisualEditorToolbar";
import { NodePalette } from "../VisualEditor/NodePalette";
import { PositionPreservation } from "../../utils/positionPreservation";
import {
  applyChaosAndCapacity,
  buildLayoutSignature,
  getManualPositionsMap,
  hashCacheKey,
  selectOptimalHandles,
  getManualLayoutViewKey,
} from "./layoutUtils";
import { updateManualLayoutPosition } from "./manualLayout";
import {
  buildC4Edges,
  buildC4Nodes,
  buildNodeSizeMap,
  buildParentChildRelationships,
  exposeQualityMetrics,
} from "./layoutPipeline";

const nodeTypes: NodeTypes = {
  sruja: SrujaNode,
  group: GroupNode, // Parent container nodes
};

const edgeTypes: EdgeTypes = {
  spline: SplineEdge,
  traffic: TrafficEdge,
};


interface LayoutCache {
  [key: string]: {
    nodes: RFNode[];
    edges: RFEdge[];
    timestamp: number;
    layoutSignature: string;
  };
}

const CACHE_TTL = 5 * 60 * 1000; // 5 minutes

export const SrujaCanvas = () => {
  // Layout Cache Ref
  const cacheRef = useRef<LayoutCache>({});

  // Global Store
  const model = useArchitectureStore((s) => s.model) as unknown as SrujaModelDump | null;
  const dslSource = useArchitectureStore((s) => s.dslSource) as string | null;
  const currentExampleFile = useArchitectureStore((s) => s.currentExampleFile);
  // Use currentExampleFile or dslSource hash as model identifier for cache invalidation
  const modelId =
    currentExampleFile || (dslSource ? `${dslSource.length}-${dslSource.substring(0, 50)}` : null);

  // View State (from Store)
  const currentLevelInfo = useViewStore((s) => s.currentLevel);
  const focusedSystemId = useViewStore((s) => s.focusedSystemId);
  const focusedContainerId = useViewStore((s) => s.focusedContainerId);

  // Derive numeric level and focus node ID for compatibility with existing layout logic
  const level = useMemo(() => {
    switch (currentLevelInfo) {
      case "L1":
        return 1;
      case "L2":
        return 2;
      case "L3":
        return 3;
      default:
        return 1;
    }
  }, [currentLevelInfo]);

  const focusNodeId = useMemo(() => {
    if (level === 3) return focusedContainerId || undefined;
    if (level === 2) return focusedSystemId || undefined;
    return undefined;
  }, [level, focusedSystemId, focusedContainerId]);

  // Collapse/expand state - managed via left navigation (UI panel removed)

  const [collapsedNodeIds, _setCollapsedNodeIds] = useState<Set<string>>(new Set());
  const [isComputing, setIsComputing] = useState(false);
  const [showChaosPanel, setShowChaosPanel] = useState(false);
  const [reactFlowInstance, setReactFlowInstance] = useState<ReactFlowInstance | null>(null);

  // React Flow State
  const [nodes, setNodes, onNodesChange] = useNodesState<RFNode>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<RFEdge>([]);

  // Use shared UI theme hook
  const { theme: uiTheme, mode } = useTheme();

  // Determine if dark mode is active (handle 'system' mode)
  const isDark =
    mode === "dark" ||
    (mode === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);

  const activeViewId = useViewStore((s) => s.activeViewId);
  const fitContextKey = useMemo(
    () => `${level}:${focusNodeId || "root"}:${activeViewId || "default"}`,
    [level, focusNodeId, activeViewId]
  );
  const storedViewport = useViewStore((s) => s.viewportByContext[fitContextKey]);
  const setViewportForContext = useViewStore((s) => s.setViewportForContext);

  useEffect(() => {
    shouldAutoFitRef.current = true;
    userInteractedRef.current = false;
    lastFocusedSelectionRef.current = null;
    positionPreservationRef.current.clear();
    previousLayoutSignatureRef.current = null;
    appliedViewportContextRef.current = null;
  }, [fitContextKey]);

  // Animation State
  const activeAnimation = useSelectionStore((s) => s.activeAnimation);
  const isAnimationPlaying = useSelectionStore((s) => s.isAnimationPlaying);
  const animationStep = useSelectionStore((s) => s.animationStep);
  const setAnimationStep = useSelectionStore((s) => s.setAnimationStep);
  const selectedNodeId = useSelectionStore((s) => s.selectedNodeId);
  const selectionSource = useSelectionStore((s) => s.selectionSource);
  const { activeTool, selectedNodeType, setActiveTool, isManualMode } = useVisualEditorStore();

  // Animation Controller Ref
  const animationControllerRef = useRef<AnimationController | null>(null);
  const visualEffectsRef = useRef<VisualEffectsSystem | null>(null);
  const shouldAutoFitRef = useRef(true);
  const userInteractedRef = useRef(false);
  const lastFocusedSelectionRef = useRef<string | null>(null);
  const positionPreservationRef = useRef(new PositionPreservation());
  const previousLayoutSignatureRef = useRef<string | null>(null);
  const appliedViewportContextRef = useRef<string | null>(null);

  // Initialize Visual Effects System
  useEffect(() => {
    // We bind to document body or a specific container if possible,
    // but VisualEffectsSystem typically works on DOM elements with data-id attributes
    visualEffectsRef.current = new VisualEffectsSystem();
    return () => {
      visualEffectsRef.current?.reset();
    };
  }, []);

  // Manage Animation Controller Lifecycle
  useEffect(() => {
    if (activeAnimation) {
      if (!animationControllerRef.current) {
        animationControllerRef.current = new AnimationController({
          model: model,
          onStepChange: (step, _data) => {
            setAnimationStep(step);
          },
          onStateChange: (state) => {
            // Sync visual effects
            if (visualEffectsRef.current) {
              visualEffectsRef.current.updateStepVisuals(
                state.activeNodes,
                state.activeEdges,
                state.visitedNodes,
                state.visitedEdges
              );
            }
          },
        });
      }

      // Update source and model
      animationControllerRef.current.setModel(model);
      animationControllerRef.current.setSource(activeAnimation);

      // Sync initial state
      if (isAnimationPlaying) {
        animationControllerRef.current.play();
      } else {
        animationControllerRef.current.pause();
      }

      animationControllerRef.current.goToStep(animationStep);
    } else {
      // Cleanup if animation stopped
      if (animationControllerRef.current) {
        animationControllerRef.current.destroy();
        animationControllerRef.current = null;
      }
      if (visualEffectsRef.current) {
        visualEffectsRef.current.reset();
      }
    }

    return () => {
      // Cleanup on unmount or change
      // We don't destroy here to avoid flickering on re-renders,
      // relying on the activeAnimation check above for main lifecycle
    };
  }, [activeAnimation, model]); // Re-create/update when source or model changes

  // Sync Store Control Actions to Controller
  useEffect(() => {
    const controller = animationControllerRef.current;
    if (!controller || !activeAnimation) return;

    if (isAnimationPlaying && !controller.isPlaying()) {
      controller.play();
    } else if (!isAnimationPlaying && controller.isPlaying()) {
      controller.pause();
    }
  }, [isAnimationPlaying, activeAnimation]);

  // Sync Store Step Navigation to Controller
  useEffect(() => {
    const controller = animationControllerRef.current;
    if (!controller || !activeAnimation) return;

    // Only update if different to avoid loops
    if (controller.getCurrentStep() !== animationStep) {
      controller.goToStep(animationStep);
    }
  }, [animationStep, activeAnimation]);

  // Update node animation states when animation step changes
  // This uses React state instead of DOM manipulation for better performance
  useEffect(() => {
    if (!activeAnimation || !activeAnimation.steps) {
      // Clear animation states from all nodes and edges
      setNodes((currentNodes) =>
        currentNodes.map((node) => ({
          ...node,
          data: { ...node.data, _animationState: "" },
          className: (node.className || "")
            .replace(/animation-mode|animation-node-\w+/g, "")
            .trim(),
        }))
      );
      setEdges((currentEdges) =>
        currentEdges.map((edge) => ({
          ...edge,
          className: (edge.className || "")
            .replace(/animation-mode|animation-edge-\w+/g, "")
            .trim(),
        }))
      );
      return;
    }

    const steps = activeAnimation.steps;
    const currentStepData = steps[animationStep];

    // Build sets of active/visited nodes
    const activeNodes = new Set<string>();
    const visitedNodes = new Set<string>();

    // Current step nodes are active
    if (currentStepData) {
      if (currentStepData.from) activeNodes.add(currentStepData.from);
      if (currentStepData.to) activeNodes.add(currentStepData.to);
    }

    // Previous steps nodes are visited
    for (let i = 0; i < animationStep; i++) {
      const step = steps[i];
      if (step?.from) visitedNodes.add(step.from);
      if (step?.to) visitedNodes.add(step.to);
    }

    // Remove active nodes from visited (active takes precedence)
    activeNodes.forEach((id) => visitedNodes.delete(id));

    // Build sets of active/visited edges based on step connections
    const activeEdges = new Set<string>();
    const visitedEdges = new Set<string>();

    // Current step edge
    if (currentStepData?.from && currentStepData?.to) {
      activeEdges.add(`${currentStepData.from}->${currentStepData.to}`);
    }

    // Previous step edges
    for (let i = 0; i < animationStep; i++) {
      const step = steps[i];
      if (step?.from && step?.to) {
        visitedEdges.add(`${step.from}->${step.to}`);
      }
    }

    // Remove active edges from visited (active takes precedence)
    activeEdges.forEach((id) => visitedEdges.delete(id));

    // Update nodes with animation state
    setNodes((currentNodes) =>
      currentNodes.map((node) => {
        let animationClass = "";
        if (activeNodes.has(node.id)) {
          animationClass = "animation-node-highlighted";
        } else if (visitedNodes.has(node.id)) {
          animationClass = "animation-node-visited";
        }

        return {
          ...node,
          data: { ...node.data, _animationState: animationClass },
          className: `animation-mode ${animationClass}`.trim(),
        };
      })
    );

    // Update edges with animation state (ByteByteGo-style)
    setEdges((currentEdges) =>
      currentEdges.map((edge) => {
        // Match edge by source->target pattern
        const edgeKey = `${edge.source}->${edge.target}`;
        let animationClass = "";

        if (activeEdges.has(edgeKey)) {
          animationClass = "animation-edge-highlighted animation-edge-flow-forward";
        } else if (visitedEdges.has(edgeKey)) {
          animationClass = "animation-edge-visited";
        }

        return {
          ...edge,
          className: `animation-mode ${animationClass}`.trim(),
        };
      })
    );
  }, [activeAnimation, animationStep, setNodes, setEdges]);

  // Auto-zoom to active nodes (ByteByteGo-style focus effect)
  // Using a ref to track last zoomed step to prevent infinite loops
  const lastZoomedStepRef = useRef<number>(-1);
  const zoomTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (!activeAnimation || !activeAnimation.steps || !reactFlowInstance || !isAnimationPlaying) {
      lastZoomedStepRef.current = -1; // Reset on animation stop
      return;
    }

    // Only zoom if step actually changed
    if (lastZoomedStepRef.current === animationStep) return;
    lastZoomedStepRef.current = animationStep;

    // Clear any pending zoom
    if (zoomTimeoutRef.current) {
      clearTimeout(zoomTimeoutRef.current);
    }

    const steps = activeAnimation.steps;
    const currentStepData = steps[animationStep];
    if (!currentStepData || (!currentStepData.from && !currentStepData.to)) return;

    // Delay zoom to let nodes update and render first
    zoomTimeoutRef.current = setTimeout(() => {
      // Find active nodes in the DOM (avoids state dependency issues)
      const activeNodeIds: string[] = [];
      if (currentStepData.from) activeNodeIds.push(currentStepData.from);
      if (currentStepData.to && currentStepData.to !== currentStepData.from) {
        activeNodeIds.push(currentStepData.to);
      }

      const activeElements = activeNodeIds
        .map((id) => {
          const el = document.querySelector(`[data-id="${id}"]`);
          if (!el) return null;
          const rect = el.getBoundingClientRect();
          const flowPosition = reactFlowInstance.screenToFlowPosition({
            x: rect.left + rect.width / 2,
            y: rect.top + rect.height / 2,
          });
          return { x: flowPosition.x, y: flowPosition.y };
        })
        .filter((pos): pos is NonNullable<typeof pos> => pos !== null);

      if (activeElements.length > 0) {
        // Calculate center of active nodes
        const avgX = activeElements.reduce((sum, n) => sum + n.x, 0) / activeElements.length;
        const avgY = activeElements.reduce((sum, n) => sum + n.y, 0) / activeElements.length;

        // Smooth zoom and pan (ByteByteGo-style auto-focus)
        reactFlowInstance.setCenter(avgX, avgY, {
          duration: 800,
          zoom: Math.min(reactFlowInstance.getZoom() || 1, 1.2),
        });
      }
    }, 600); // Delay to let animations and DOM updates settle

    return () => {
      if (zoomTimeoutRef.current) {
        clearTimeout(zoomTimeoutRef.current);
        zoomTimeoutRef.current = null;
      }
    };
  }, [activeAnimation, animationStep, isAnimationPlaying, reactFlowInstance]);

  // Store actions
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);

  // Chaos Mode State
  const chaosState = useArchitectureStore((s) => s.chaosState);
  const setFailedNode = useArchitectureStore((s) => s.setFailedNode);
  const setChaosEnabled = useArchitectureStore((s) => s.setChaosEnabled);
  const capacityState = useArchitectureStore((s) => s.capacityState);
  // Role selection removed - now handled in Roles tab

  // Calculate Blast Radius (Impacted Nodes)
  const impactedNodeIds = useMemo(() => {
    if (!chaosState.enabled || !chaosState.failedNodeId || !model) return new Set<string>();

    // Use the ArchitectureModel helper we added
    const archModel = getArchitectureModel();
    // Ensure archModel is in sync (it should be subscribed, but we can call directly)
    if (archModel.getModel() !== model) {
      archModel.updateModel(model);
    }
    return archModel.getBlastRadius(chaosState.failedNodeId);
  }, [chaosState.enabled, chaosState.failedNodeId, model]);

  // Drag and Drop Handlers
  const onMoveStart = useCallback(() => {
    userInteractedRef.current = true;
  }, []);

  const onMoveEnd = useCallback(
    (event: MouseEvent | TouchEvent | null, viewport: { x: number; y: number; zoom: number }) => {
      if (!event || !viewport) return;
      userInteractedRef.current = true;
      setViewportForContext(fitContextKey, viewport);
    },
    [fitContextKey, setViewportForContext]
  );

  const onDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = "copy";
  }, []);

  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();

      if (!reactFlowInstance) return;

      const featureData = event.dataTransfer.getData("application/feature");
      if (!featureData) return;

      try {
        const feature = JSON.parse(
          featureData
        ) as import("../../data/featureTemplates").FeatureTemplate;
        const position = reactFlowInstance.screenToFlowPosition({
          x: event.clientX,
          y: event.clientY,
        });

        // Feature dropped - handled

        // Update architecture model with new components from feature
        updateArchitecture((model) => {
          const newModel = { ...model };
          if (!newModel.elements) newModel.elements = {};

          // Helper to generate unique ID
          const generateId = (base: string) => {
            const id = base.toLowerCase().replace(/[^a-z0-9]/g, "");
            if (!newModel.elements[id]) return id;
            let counter = 1;
            while (newModel.elements[`${id}${counter}`]) {
              counter++;
            }
            return `${id}${counter}`;
          };

          // Group new components
          // If we are drilled down (level > 1) and have a focusNodeId, we add as children
          const parentId = level > 1 && focusNodeId ? focusNodeId : undefined;

          feature.requiredComponents.forEach((comp, index) => {
            const id = generateId(comp.name);

            // Map template type to C4 kind
            let kind = comp.type;
            const tags: string[] = ["feature:" + feature.id];

            // Normalize kinds
            if (kind === "service") kind = "container";
            if (kind === "datastore") {
              kind = "container";
              tags.push("database");
            }
            if (kind === "queue") {
              kind = "container";
              tags.push("queue");
            }

            // Offset positions slightly for multiple components
            // Note: Position isn't stored in C4 model, but we set it in metdata for initial layout hint if needed
            // However, Current layout engine recalculates positions.
            // We just add to model.

            newModel.elements[id] = {
              id,
              title: comp.name,
              kind: kind as ElementDump["kind"],
              description: typeof comp.description === "string" ? comp.description : undefined,
              technology: comp.technology,
              tags,
              parent: parentId,
              // Add metadata for layout engine if we want to preserve dropped position?
              // The current DOT layout engine might overwrite this, but we can try.
              metadata: {
                // @ts-expect-error - position is not in ElementDump type but needed for layout hint
                position: { x: position.x + index * 40, y: position.y + index * 40 },
              },
            };
          });

          return newModel;
        });

        trackInteraction("drop", "feature", { featureId: feature.id });
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        logger.error("Failed to handle drop", {
          component: "SrujaCanvas",
          action: "handleDrop",
          error:
            err instanceof Error
              ? {
                  message: err.message,
                  name: err.name,
                  stack: err.stack,
                }
              : errorMessage,
        });
      }
    },
    [reactFlowInstance, updateArchitecture, level, focusNodeId]
  );

  // Get node title helper
  const getNodeTitle = useCallback(
    (nodeId: string | undefined): string => {
      if (!nodeId || !model) return "";
      const element = model.elements[nodeId];
      return element?.title || nodeId;
    },
    [model]
  );

  const queueAutoFit = useCallback(() => {
    if (!reactFlowInstance) return;
    if (!shouldAutoFitRef.current || userInteractedRef.current) return;
    shouldAutoFitRef.current = false;
    setTimeout(() => {
      reactFlowInstance.fitView({ padding: 0.2, duration: 300 });
    }, 100);
  }, [reactFlowInstance]);

  useEffect(() => {
    if (!reactFlowInstance || !storedViewport) return;
    if (appliedViewportContextRef.current === fitContextKey) return;
    if (userInteractedRef.current) return;
    appliedViewportContextRef.current = fitContextKey;
    shouldAutoFitRef.current = false;
    userInteractedRef.current = true;
    reactFlowInstance.setViewport(storedViewport, { duration: 0 });
  }, [fitContextKey, reactFlowInstance, storedViewport]);

  // Pipeline Execution with Caching
  // Include theme in dependencies to force re-render when theme changes
  useEffect(() => {
    if (!model) {
      setNodes([]);
      setEdges([]);
      positionPreservationRef.current.clear();
      previousLayoutSignatureRef.current = null;
      return;
    }

    const computeLayout = async () => {
      const cacheKey =
        hashCacheKey(level, focusNodeId, collapsedNodeIds, modelId) +
        (activeViewId ? `:${activeViewId}` : "") +
        (isManualMode ? ":manual" : ":auto");
      const cached = cacheRef.current[cacheKey];

      // Check cache validity
      if (cached && Date.now() - cached.timestamp < CACHE_TTL) {
        setNodes(cached.nodes);
        setEdges(cached.edges);
        positionPreservationRef.current.updateFromNodes(
          cached.nodes as unknown as FlowNode<C4NodeData>[]
        );
        previousLayoutSignatureRef.current = cached.layoutSignature;
        queueAutoFit();
        return;
      }

      setIsComputing(true);
      try {
        // 1. Generate DOT via Go WASM (with view projection)
        // Use dslSource if available, otherwise fallback cannot proceed
        if (!dslSource) {
          // No DSL source available - skipping layout
          setNodes([]);
          setEdges([]);
          return;
        }

        // Pre-calculate node sizes for accurate layout
        const nodeSizes = buildNodeSizeMap(model);

        const result = await convertDslToDot(
          dslSource,
          activeViewId ? 1 : level, // Default to L1 when using a view definition
          activeViewId ? undefined : focusNodeId, // Don't pass focusNodeId when using viewId
          nodeSizes,
          activeViewId || undefined, // Pass the view ID to load the view definition
          currentExampleFile || undefined // Pass filename
        );

        if (!result || !result.dot) {
          // DOT generation failed - skipping layout
          setNodes([]);
          setEdges([]);
          setIsComputing(false);
          return;
        }

        // Debug logging removed - use browser devtools if needed

        // 2. Layout with iterative refinement
        // Note: parent-child relationships will be extracted AFTER layout using only visible nodes
        let layoutResult;
        try {
          layoutResult = await runGraphviz(result.dot);
        } catch (layoutError) {
          // Handle Graphviz layout errors with user-friendly messages
          handleError(layoutError, "SrujaCanvas.computeLayout.runGraphviz");

          // Show error toast to user
          const showToast = useToastStore.getState().showToast;
          showToast(
            layoutError instanceof GraphvizLayoutError
              ? layoutError.message
              : "Failed to generate diagram layout. Please check your architecture model for errors.",
            "error",
            8000 // Longer duration for error messages
          );

          // Clear diagram and stop computation
          setNodes([]);
          setEdges([]);
          setIsComputing(false);
          return;
        }

        // 3. Extract parent-child relationships ONLY from visible nodes in current view
        // This ensures L2/L3 views only show relationships within the focused scope
        const visibleNodeIds = new Set(layoutResult.nodes.map((n) => n.id));
        const parentChildRelationships = buildParentChildRelationships(model, visibleNodeIds);

        // 4. Measure quality (dev-only) - will be recalculated after compound nodes are built
        // if using compound structure, to account for parent container bounding boxes
        const isDev = import.meta.env.DEV || import.meta.env.MODE === "development";
        let qualityResult: {
          quality: LayoutQuality;
          parentChildContainmentViolations: Array<{ childId: string; parentId: string }>;
        } | null = null;
        if (isDev) {
          // Initial quality measurement (will be recalculated for compound nodes)
          qualityResult = measureQuality(layoutResult, parentChildRelationships);
        }
        let quality = qualityResult?.quality ?? null;
        let parentChildContainmentViolations =
          qualityResult?.parentChildContainmentViolations ?? [];

        // 5. Build C4Nodes from layout result and model metadata

        // Quality metrics exposed via window.__DIAGRAM_QUALITY__ for e2e tests
        const c4Nodes = buildC4Nodes({
          layoutResult,
          model,
          nodeSizes,
          level: level as C4Level,
        });

        // 4. Build edges from projected relations returned by Go
        const c4Edges = buildC4Edges(result.relations);

        // Layout complete: nodes and edges computed
        const layoutSignature = buildLayoutSignature(
          fitContextKey,
          layoutResult.nodes,
          result.relations
        );

        // Expose quality metrics to window for e2e tests and UI (dev only)
        // Quality metrics are developer tools, not user-facing features
        if (quality) {
          exposeQualityMetrics({
            quality,
            parentChildContainmentViolations,
            c4Nodes,
            c4Edges,
            level,
          });
        }

        // Early return if no nodes
        if (c4Nodes.length === 0) {
          setNodes([]);
          setEdges([]);
          positionPreservationRef.current.clear();
          previousLayoutSignatureRef.current = null;
          return;
        }

        // 5. React Flow Mapping
        // Check for saved manual positions in view metadata
        // Priority: 1) manual-layout-* (from UI edits), 2) regular view IDs (from DSL), 3) level-based views (L1, L2, L3)
        const { manualPositionsMap, hasManualPositions } = getManualPositionsMap({
          model,
          level,
          focusNodeId,
          isManualMode,
        });

        // Build compound node structure if clusters are available
        // This creates parent nodes as visual containers with children inside
        let nextNodes: RFNode[];
        const hasClusters = layoutResult.clusters && Object.keys(layoutResult.clusters).length > 0;

        // Log structure decision in dev mode (isDev is defined later in this function)
        const logStructure =
          typeof import.meta !== "undefined" &&
          (import.meta.env?.DEV || import.meta.env?.MODE === "development");
        if (logStructure && typeof window !== "undefined") {
          if (hasClusters && layoutResult.clusters) {
            console.debug(
              `[SrujaCanvas] Using compound node structure with ${Object.keys(layoutResult.clusters).length} clusters`
            );
          } else {
            console.debug("[SrujaCanvas] No clusters found - using flat node structure");
          }
        }

        if (hasClusters) {
          // Use compound node structure (parent containers with children)
          nextNodes = buildCompoundNodeStructure(layoutResult, c4Nodes, manualPositionsMap);
        } else {
          // Fallback to flat structure (no clusters available)
          nextNodes = c4Nodes.map((node) => {
            const layout = layoutResult.nodes.find((n) => n.id === node.id);

            // Use saved manual position if available, otherwise use auto-layout
            // Handle both old format (direct object) and new format (ViewPositionDump)
            const savedPosition = manualPositionsMap[node.id];
            let position = { x: 0, y: 0 };
            if (savedPosition) {
              // Handle both formats: {x, y} or ViewPositionDump with X, Y
              const pos = savedPosition as { x?: number; y?: number; X?: number; Y?: number };
              position = {
                x: pos.x ?? pos.X ?? 0,
                y: pos.y ?? pos.Y ?? 0,
              };
            } else if (layout) {
              position = { x: layout.x, y: layout.y };
            }

            return {
              id: node.id,
              type: "sruja",
              position,
              data: {
                ...node,
              } as C4Node & Record<string, unknown>,
              width: node.width,
              height: node.height,
            };
          });
        }

        // Apply chaos styling and capacity metrics to all nodes (both flat and compound)
        nextNodes = applyChaosAndCapacity({
          nodes: nextNodes,
          chaosState,
          impactedNodeIds,
          capacityState,
          themeMode: mode,
        });

        const shouldPreservePositions =
          previousLayoutSignatureRef.current === layoutSignature &&
          !isManualMode &&
          !hasManualPositions;

        if (shouldPreservePositions) {
          const { stableNodeIds } = positionPreservationRef.current.detectChanges(
            nextNodes as unknown as FlowNode<C4NodeData>[],
            new Set<string>(),
            new Set<string>()
          );
          nextNodes = positionPreservationRef.current.applyPreservedPositions(
            nextNodes as unknown as FlowNode<C4NodeData>[],
            stableNodeIds
          ) as unknown as RFNode[];
        }

        // Recalculate quality metrics using actual rendered positions (for compound nodes)
        // This ensures parent-child containment is checked against actual parent container bounding boxes
        if (hasClusters && nextNodes.length > 0) {
          const updatedQualityResult = measureQualityFromNodes(
            nextNodes,
            layoutResult.edges,
            parentChildRelationships
          );
          quality = updatedQualityResult.quality;
          parentChildContainmentViolations = updatedQualityResult.parentChildContainmentViolations;
        }

        // Create a map of nodes for quick lookup
        const nodeMap = new Map(nextNodes.map((n) => [n.id, n]));

        // Build map of Graphviz edge spline points for lookup
        const gvEdgeMap = new Map<string, Array<[number, number]> | undefined>();
        layoutResult.edges.forEach((gvEdge) => {
          // Key by source-target
          const key = `${gvEdge.source}-${gvEdge.target}`;
          gvEdgeMap.set(key, gvEdge.points);
        });
        // Graphviz edge map computed

        // Filter and create edges only for nodes that exist
        const rfValidNodeIds = new Set(nextNodes.map((n) => n.id));
        const nextEdges: RFEdge[] = c4Edges
          .filter((edge) => {
            const sourceExists = rfValidNodeIds.has(edge.source);
            const targetExists = rfValidNodeIds.has(edge.target);
            if (!sourceExists || !targetExists) {
              // Edge skipped: invalid source or target
              return false;
            }
            return true;
          })
          .map((edge) => {
            const sourceNode = nodeMap.get(edge.source);
            const targetNode = nodeMap.get(edge.target);

            if (!sourceNode || !targetNode) {
              // Fallback to default handles if nodes not found
              // Theme-aware fallback edge colors using shared UI theme
              const edgeColor = isDark ? uiTheme.neutral[600] : uiTheme.neutral[600];

              return {
                id: edge.id,
                source: edge.source,
                target: edge.target,
                sourceHandle: "source-bottom",
                targetHandle: "target-top",
                type: "smoothstep" as EdgeType,
                animated: false,
                style: { stroke: edgeColor, strokeWidth: 2 },
                markerEnd: {
                  type: MarkerType.ArrowClosed,
                  color: edgeColor,
                  width: 20,
                  height: 20,
                },
              };
            }

            // Select optimal handles based on node positions
            const { sourceHandle, targetHandle } = selectOptimalHandles(sourceNode, targetNode);

            // Theme-aware edge colors using shared UI theme
            const edgeColor = isDark ? uiTheme.neutral[600] : uiTheme.neutral[600]; // slate-600 for both themes
            // Use high-contrast colors for edge labels to ensure visibility
            // White text on dark blue background for both themes
            const labelColor = "#FFFFFF"; // White text for maximum contrast
            const labelBgColor = isDark ? "#1e3a8a" : "#2563eb"; // Dark blue (800) for dark theme, blue (600) for light theme

            // Use SplineEdge if points are available for high fidelity
            // Graphviz spline coordinates are now reliable with node sizing
            // BUT: If in manual mode or using saved positions, splines won't match node positions
            const splinePoints = gvEdgeMap.get(`${edge.source}-${edge.target}`);
            // Disable splines if in manual mode or if we have manual positions loaded
            const useSpline =
              splinePoints && splinePoints.length > 0 && !isManualMode && !hasManualPositions;

            const finalEdgeType = useSpline ? "spline" : "smoothstep";

            const hasLabel = edge.label && edge.label.trim().length > 0;
            return {
              id: edge.id,
              source: edge.source,
              target: edge.target,
              // Always use handles so React Flow maintains connections when nodes move
              sourceHandle: sourceHandle,
              targetHandle: targetHandle,
              type: finalEdgeType,
              data: useSpline ? { points: splinePoints, label: edge.label } : undefined,
              ...(hasLabel && {
                label: edge.label,
                labelShowBg: true,
                labelStyle: {
                  color: labelColor,
                  fontWeight: 600,
                  fontSize: "12px",
                  textShadow: "0 1px 2px rgba(0, 0, 0, 0.2)", // Add subtle text shadow for better readability
                },
                labelBgStyle: {
                  backgroundColor: labelBgColor,
                  opacity: 0.95,
                  padding: "4px 8px",
                  borderRadius: "6px",
                  border: `1px solid ${isDark ? "#1e40af" : "#1d4ed8"}`, // Slightly darker border for definition
                  boxShadow: "0 2px 4px rgba(0, 0, 0, 0.15)", // Add shadow for depth
                },
              }),
              animated: false,
              style: { stroke: edgeColor, strokeWidth: 2 },
              markerEnd: {
                type: MarkerType.ArrowClosed,
                color: edgeColor,
                width: 20,
                height: 20,
              },
            };
          });

        // Debug logging
        console.debug(
          `[SrujaCanvas] Layout complete: ${nextNodes.length} nodes, ${nextEdges.length} edges (from ${c4Edges.length} projected edges)`
        );
        // Edges processed and validated

        // Cache the result
        cacheRef.current[cacheKey] = {
          nodes: nextNodes,
          edges: nextEdges,
          timestamp: Date.now(),
          layoutSignature,
        };

        setNodes(nextNodes);
        setEdges(nextEdges);

        positionPreservationRef.current.updateFromNodes(
          nextNodes as unknown as FlowNode<C4NodeData>[]
        );
        previousLayoutSignatureRef.current = layoutSignature;

        queueAutoFit();
      } catch (err) {
        // Fallback error handling for unexpected errors
        handleError(err, "SrujaCanvas.computeLayout");
        const showToast = useToastStore.getState().showToast;
        showToast(
          "An unexpected error occurred while generating the diagram. Please try again.",
          "error",
          8000
        );
        // Clear diagram on error
        setNodes([]);
        setEdges([]);
      } finally {
        setIsComputing(false);
      }
    };

    computeLayout();
  }, [
    model,
    modelId,
    level,
    focusNodeId,
    collapsedNodeIds,
    fitContextKey,
    setNodes,
    setEdges,
    reactFlowInstance,
    activeViewId,
    isManualMode,
    queueAutoFit,
    mode,
    isDark,
    uiTheme,
  ]); // Include modelId to invalidate cache when model changes

  useEffect(() => {
    setNodes((currentNodes) => {
      let didChange = false;
      const nextNodes = currentNodes.map((node) => {
        const shouldSelect = !!selectedNodeId && node.id === selectedNodeId;
        if (node.selected !== shouldSelect) {
          didChange = true;
          return { ...node, selected: shouldSelect };
        }
        return node;
      });
      return didChange ? nextNodes : currentNodes;
    });
  }, [selectedNodeId, setNodes]);

  useEffect(() => {
    if (!selectedNodeId || !reactFlowInstance) return;
    if (selectionSource !== "code" && selectionSource !== "navigation") return;
    if (lastFocusedSelectionRef.current === selectedNodeId) return;

    const targetNode = nodes.find((node) => node.id === selectedNodeId);
    if (!targetNode) return;

    const centerX = targetNode.position.x + (targetNode.width ?? 0) / 2;
    const centerY = targetNode.position.y + (targetNode.height ?? 0) / 2;
    lastFocusedSelectionRef.current = selectedNodeId;

    reactFlowInstance.setCenter(centerX, centerY, {
      duration: 300,
      zoom: Math.max(reactFlowInstance.getZoom(), 1),
    });
  }, [nodes, reactFlowInstance, selectedNodeId, selectionSource]);

  // Selection store for details panel
  const selectNode = useSelectionStore((s) => s.selectNode);

  // Visual editor store
  const addNode = useArchitectureStore((s) => s.addNode);
  const addRelation = useArchitectureStore((s) => s.addRelation);
  const deleteNodes = useArchitectureStore((s) => s.deleteNodes);
  const showToast = useToastStore((s) => s.showToast);

  // Manual editing: Track node positions for saving
  const [, setManualPositions] = useState<Map<string, { x: number; y: number }>>(new Map());
  const [hasManualEdits, setHasManualEdits] = useState(false);

  // Manual editing: Handle node drag end to save positions
  const onNodeDragStop = useCallback(
    (_event: React.MouseEvent, node: RFNode) => {
      // Save position to manual positions map
      setManualPositions((prev) => {
        const next = new Map(prev);
        next.set(node.id, { x: node.position.x, y: node.position.y });
        return next;
      });
      setHasManualEdits(true);

      // Save to architecture store (in view metadata)
      if (model) {
        updateArchitecture((currentModel) =>
          updateManualLayoutPosition({
            model: currentModel,
            nodeId: node.id,
            level,
            focusNodeId,
            position: { x: node.position.x, y: node.position.y },
          })
        );
      }

      trackInteraction("manual-edit", "node-position", {
        nodeId: node.id,
        position: node.position,
        level,
        focusNodeId,
      });
    },
    [model, updateArchitecture, level, focusNodeId]
  );

  // Visual editing: Handle pane click for node creation
  const handlePaneInteraction = useCallback(
    (event: React.MouseEvent | MouseEvent) => {
      // Only proceed if in create-node mode with selected type
      if (activeTool === "create-node" && selectedNodeType && reactFlowInstance) {
        // Prevent default to avoid conflicts
        event.preventDefault?.();
        event.stopPropagation?.();

        const point = reactFlowInstance.screenToFlowPosition({
          x: event.clientX,
          y: event.clientY,
        });

        // Generate a default name based on type
        const defaultName = `${selectedNodeType.charAt(0).toUpperCase() + selectedNodeType.slice(1)} ${Date.now().toString().slice(-4)}`;

        logger.debug("Creating node", {
          component: "SrujaCanvas",
          nodeType: selectedNodeType,
          name: defaultName,
          position: point,
        });

        // Create the node first
        addNode(selectedNodeType, defaultName, undefined, point)
          .then(async () => {
            // After node is created, save its position in view metadata
            // Use the same view key format as onNodeDragStop
            const viewKey = getManualLayoutViewKey(level, focusNodeId);

            // Get the node ID - need to wait for the node to be created first
            // The node ID will be generated by addNode, so we need to get it from the model
            // For now, we'll save the position after a short delay to ensure the node exists
            // A better approach would be to return the nodeId from addNode, but for now this works
            setTimeout(async () => {
              const currentModel = useArchitectureStore.getState().model;
              if (!currentModel) return;

              // Find the node we just created (it will have the name we used)
              const nodeId = Object.keys(currentModel.elements || {}).find(
                (id) => currentModel.elements?.[id]?.title === defaultName
              );

              if (!nodeId) {
                logger.warn("Could not find created node to save position", {
                  component: "SrujaCanvas",
                  nodeName: defaultName,
                });
                return;
              }

              // Save position in view metadata
              await updateArchitecture((currentModel) =>
                updateManualLayoutPosition({
                  model: currentModel,
                  nodeId,
                  level,
                  focusNodeId,
                  position: { x: point.x, y: point.y },
                })
              );
            }, 100); // Small delay to ensure node is created

            showToast(`Created ${selectedNodeType} "${defaultName}"`, "success");
            // Reset tool to select after creation
            setActiveTool("select");
          })
          .catch((error) => {
            logger.error("Failed to create node", {
              component: "SrujaCanvas",
              action: "handlePaneInteraction",
              error: error instanceof Error ? error.message : String(error),
            });
            showToast(
              `Failed to create node: ${error instanceof Error ? error.message : String(error)}`,
              "error"
            );
          });
      }
    },
    [
      activeTool,
      selectedNodeType,
      reactFlowInstance,
      addNode,
      updateArchitecture,
      setActiveTool,
      showToast,
      level,
      focusNodeId,
    ]
  );

  const onPaneClick = useCallback(
    (event: React.MouseEvent) => {
      console.log("[SrujaCanvas] onPaneClick FIRED", {
        activeTool,
        selectedNodeType,
        clientX: event.clientX,
        clientY: event.clientY,
        target: event.target,
      });
      handlePaneInteraction(event);
    },
    [handlePaneInteraction]
  );

  // Alternative approach: Add direct event listener to pane element as fallback
  // This handles cases where React Flow's onPaneClick doesn't fire
  useEffect(() => {
    if (!reactFlowInstance || activeTool !== "create-node" || !selectedNodeType) {
      return;
    }

    let cleanup: (() => void) | undefined;

    // Wait a bit for React Flow to render
    const timeoutId = setTimeout(() => {
      // Try multiple selectors to find the pane
      const reactFlowPane = document.querySelector(".react-flow__pane") as HTMLElement;
      const reactFlowWrapper = document.querySelector(".react-flow") as HTMLElement;

      if (!reactFlowPane && !reactFlowWrapper) {
        console.warn("[SrujaCanvas] Could not find React Flow pane element");
        return;
      }

      const targetElement = reactFlowPane || reactFlowWrapper;

      const handlePaneClick = (e: MouseEvent) => {
        const target = e.target as HTMLElement;

        // Ignore clicks on UI elements
        if (
          target.closest(".react-flow__node") ||
          target.closest(".react-flow__edge") ||
          target.closest(".react-flow__controls") ||
          target.closest(".react-flow__minimap") ||
          target.closest('[data-testid="visual-editor-toolbar"]') ||
          target.closest('[data-testid="node-palette"]') ||
          target.closest('[class*="mantine"]') // Ignore Mantine UI components
        ) {
          return;
        }

        // Only handle clicks on pane/background
        if (
          target.classList.contains("react-flow__pane") ||
          target.classList.contains("react-flow__background") ||
          target === targetElement ||
          (!target.closest(".react-flow__node") && !target.closest(".react-flow__edge"))
        ) {
          console.log("[SrujaCanvas] Direct pane click handler fired (fallback)", {
            activeTool,
            selectedNodeType,
            targetClass: target.className,
            targetTag: target.tagName,
            clientX: e.clientX,
            clientY: e.clientY,
          });
          e.stopPropagation();
          handlePaneInteraction(e);
        }
      };

      targetElement.addEventListener("click", handlePaneClick, true); // Use capture phase

      cleanup = () => {
        targetElement.removeEventListener("click", handlePaneClick, true);
      };
    }, 100); // Small delay to ensure DOM is ready

    return () => {
      clearTimeout(timeoutId);
      if (cleanup) {
        cleanup();
      }
    };
  }, [reactFlowInstance, activeTool, selectedNodeType, handlePaneInteraction]);

  // Visual editing: Handle connection creation
  const onConnect = useCallback(
    (params: Connection) => {
      const { source, target } = params;
      if (source && target && activeTool === "connect") {
        addRelation(source, target, "")
          .then(() => {
            showToast("Connection created", "success");
            // Reset tool to select after connection
            setActiveTool("select");
          })
          .catch((error) => {
            logger.error("Failed to create connection", {
              component: "SrujaCanvas",
              action: "onConnect",
              error: error instanceof Error ? error.message : String(error),
            });
            showToast(
              `Failed to create connection: ${error instanceof Error ? error.message : String(error)}`,
              "error"
            );
          });
      }
    },
    [activeTool, addRelation, setActiveTool, showToast]
  );

  // Visual editing: Handle node deletion
  const onNodesDelete = useCallback(
    (deleted: RFNode[]) => {
      if (deleted.length > 0) {
        const nodeIds = deleted.map((node) => node.id);
        deleteNodes(nodeIds)
          .then(() => {
            showToast(`Deleted ${nodeIds.length} node(s)`, "success");
          })
          .catch((error) => {
            logger.error("Failed to delete nodes", {
              component: "SrujaCanvas",
              action: "onNodesDelete",
              error: error instanceof Error ? error.message : String(error),
            });
            showToast(
              `Failed to delete nodes: ${error instanceof Error ? error.message : String(error)}`,
              "error"
            );
          });
      }
    },
    [deleteNodes, showToast]
  );

  // Check if current view has manual positions
  useEffect(() => {
    if (model) {
      const viewKey = getManualLayoutViewKey(level, focusNodeId);
      const savedPositions = model.views?.[viewKey];
      type ViewWithLayout = typeof savedPositions & {
        layout?: { positions?: Record<string, { x: number; y: number }> };
        Layout?: {
          Positions?: Record<string, { x: number; y: number }>;
          positions?: Record<string, { x: number; y: number }>;
        };
      };
      const viewWithLayout = savedPositions as ViewWithLayout | undefined;
      // Support both old format (layout.positions) and new format (Layout.Positions)
      const positions =
        viewWithLayout?.layout?.positions ||
        viewWithLayout?.Layout?.Positions ||
        viewWithLayout?.Layout?.positions ||
        {};
      const hasSaved = positions && Object.keys(positions).length > 0;
      setHasManualEdits(hasSaved || false);
    } else {
      setHasManualEdits(false);
    }
  }, [model, level, focusNodeId]);

  // Chaos Mode Interaction Handler
  const onNodeClickChaos = useCallback(
    (event: React.MouseEvent, node: RFNode) => {
      event.stopPropagation(); // prevent standard selection
      if (chaosState.enabled) {
        // Toggle failure state
        if (chaosState.failedNodeId === node.id) {
          setFailedNode(null);
        } else {
          // Only fail 'container' or 'component' nodes, not systems for now (optional)
          setFailedNode(node.id);
          trackInteraction("chaos-mode", "fail-node", { nodeId: node.id });
        }
      }
    },
    [chaosState.enabled, chaosState.failedNodeId, setFailedNode]
  );

  // Navigation Handlers
  const onNodeClick = useCallback(
    (event: React.MouseEvent, node: RFNode) => {
      // Intercept for Chaos Mode
      if (chaosState.enabled) {
        onNodeClickChaos(event, node);
        return;
      }
      const c4Data = node.data as unknown as C4Node;

      // Check if clicking on expand/collapse button (handled separately)
      const target = event.target as HTMLElement;
      if (target.closest(".expand-btn")) {
        return;
      }

      // Select the node to open details panel
      selectNode(node.id, "diagram");

      // Just selecting a node without drill-down (Single Click behavior)
      trackInteraction("select", "node", { nodeId: node.id, nodeKind: c4Data.kind, level });

      // Handle double click for drill down
      const clickTime = Date.now();
      const nodeWithClick = node as RFNode & { _lastClick?: number };
      const lastClick = nodeWithClick._lastClick || 0;
      nodeWithClick._lastClick = clickTime;

      if (clickTime - lastClick < 300) {
        // Double click detected
        const c4Node = node.data as unknown as C4Node;
        if (c4Node.kind === "system") {
          useViewStore.getState().drillDown(c4Node.id, "system");
        } else if (c4Node.kind === "container") {
          const parentId = focusNodeId; // Use current focus as parent for container
          useViewStore.getState().drillDown(c4Node.id, "container", parentId);
        }
      }
    },
    [level, selectNode, focusNodeId, chaosState.enabled, onNodeClickChaos]
  );

  const onGoUp = useCallback(() => {
    const previousLevel = level;
    if (level === 3) {
      useViewStore.getState().goUp();
      trackInteraction("navigate", "level", { from: previousLevel, to: 2, nodeId: focusNodeId }); // focusNodeId is the container being exited
    } else if (level === 2) {
      useViewStore.getState().goToRoot(); // or goUp(), same effect from L2
      trackInteraction("navigate", "level", { from: previousLevel, to: 1 });
    }
  }, [level, focusNodeId]);

  // Build breadcrumb path
  const breadcrumbPath = useMemo(() => {
    if (level === 1) {
      return [{ id: "", title: "System Context", level: 1 as C4Level }];
    }

    if (level === 2 && focusNodeId) {
      const systemTitle = getNodeTitle(focusNodeId);
      return [
        { id: "", title: "System Context", level: 1 as C4Level },
        { id: focusNodeId, title: systemTitle, level: 2 as C4Level },
      ];
    }

    if (level === 3 && focusNodeId && model) {
      const container = model.elements[focusNodeId];
      const systemId = container?.parent;
      const systemTitle = systemId ? getNodeTitle(systemId) : "";
      const containerTitle = getNodeTitle(focusNodeId);

      return [
        { id: "", title: "System Context", level: 1 as C4Level },
        { id: systemId || "", title: systemTitle, level: 2 as C4Level },
        { id: focusNodeId, title: containerTitle, level: 3 as C4Level },
      ];
    }

    return [];
  }, [level, focusNodeId, model, getNodeTitle]);

  // Keyboard Shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Escape to go up one level
      if (e.key === "Escape") {
        if (level > 1) {
          onGoUp();
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [level, onGoUp]);

  // Expose navigation methods on window for e2e testing
  useEffect(() => {
    if (typeof window !== "undefined") {
      // @ts-expect-error - Adding test helpers
      window.navigateCanvas = (targetLevel: number, targetId?: string) => {
        const store = useViewStore.getState();
        if (targetLevel === 1) {
          store.goToRoot();
        } else if (targetLevel === 2) {
          if (targetId) store.drillDown(targetId, "system");
        } else if (targetLevel === 3) {
          if (targetId) store.drillDown(targetId, "container");
        }
      };
    }
  }, []);

  const handleBreadcrumbClick = useCallback(
    (targetLevel: C4Level, targetId?: string) => {
      const previousLevel = level;
      const store = useViewStore.getState();

      // targetLevel is C4Level (number: 1, 2, 3)
      if (targetLevel === 1) {
        store.goToRoot();
        trackInteraction("navigate", "breadcrumb", { from: previousLevel, to: 1 });
      } else if (targetLevel === 2 && targetId) {
        store.drillDown(targetId, "system");
        trackInteraction("navigate", "breadcrumb", {
          from: previousLevel,
          to: 2,
          nodeId: targetId,
        });
      } else if (targetLevel === 3 && targetId) {
        store.drillDown(targetId, "container");
        trackInteraction("navigate", "breadcrumb", {
          from: previousLevel,
          to: 3,
          nodeId: targetId,
        });
      }
    },
    [level]
  );

  // Theme-aware colors using shared UI theme
  const bgColor = uiTheme.background;
  const paperBg = isDark
    ? `${uiTheme.surface}F2` // 95% opacity (F2 in hex)
    : `${uiTheme.background}F2`;
  const backgroundPatternColor = isDark ? uiTheme.surface : uiTheme.neutral[200]; // slate-200 for light

  return (
    <div className="w-full h-full relative" style={{ backgroundColor: bgColor }}>
      {/* Navigation Header (Left) */}
      <Paper
        shadow="sm"
        p="xs"
        withBorder
        style={{
          position: "absolute",
          top: 16,
          left: 16,
          zIndex: 10,
          backgroundColor: paperBg,
          backdropFilter: "blur(8px)",
          borderRadius: "8px", // Sleek rounding
        }}
      >
        <Group gap="xs" align="center">
          {level > 1 && (
            <ActionIcon variant="subtle" onClick={onGoUp} aria-label="Go up one level" size="sm">
              <ArrowLeft size={16} />
            </ActionIcon>
          )}

          <Group gap={4} align="center">
            {breadcrumbPath.map((item, idx) => (
              <Group key={item.id || "root"} gap={4} align="center">
                {idx > 0 && (
                  <Text size="sm" c="dimmed">
                    /
                  </Text>
                )}
                <Button
                  variant={idx === breadcrumbPath.length - 1 ? "light" : "subtle"}
                  size="xs"
                  onClick={() => handleBreadcrumbClick(item.level, item.id || undefined)}
                  style={{
                    fontWeight: idx === breadcrumbPath.length - 1 ? 600 : 400,
                  }}
                >
                  {item.title}
                </Button>
              </Group>
            ))}
          </Group>
        </Group>
      </Paper>

      {/* Tools Toolbar (Right) */}
      <Paper
        shadow="sm"
        p="xs"
        withBorder
        style={{
          position: "absolute",
          top: 16,
          right: 16,
          zIndex: 10,
          backgroundColor: paperBg,
          backdropFilter: "blur(8px)",
          borderRadius: "8px",
        }}
      >
        <Group gap="xs">
          {/* Manual editing indicator */}
          {hasManualEdits && (
            <Badge
              leftSection={<Edit3 size={12} />}
              variant="light"
              color="blue"
              size="sm"
              title="This view has manual position adjustments"
            >
              Manual Layout
            </Badge>
          )}

          {/* Chaos Mode Toggle */}
          <ActionIcon
            variant={showChaosPanel || chaosState.enabled ? "filled" : "subtle"}
            color={chaosState.enabled ? "red" : "gray"}
            onClick={() => setShowChaosPanel(!showChaosPanel)}
            title="Toggle Chaos Engineering Mode"
            size="sm"
          >
            <Zap size={16} />
          </ActionIcon>
        </Group>
      </Paper>

      {isComputing && (
        <div
          style={{
            position: "absolute",
            inset: 0,
            zIndex: 50,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            backgroundColor: isDark ? `${uiTheme.background}B3` : `${uiTheme.background}B3`, // 70% opacity (B3 in hex)
            backdropFilter: "blur(4px)",
          }}
        >
          <Paper shadow="lg" p="md" withBorder>
            <Stack gap="xs" align="center">
              <Loader size="sm" />
              <Text size="sm" fw={500} c="dimmed">
                Computing Layout...
              </Text>
              <Text size="xs" c="dimmed">
                This may take a moment
              </Text>
            </Stack>
          </Paper>
        </div>
      )}

      <div
        style={{
          width: "100%",
          height: "100%",
          position: "relative",
        }}
      >
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onNodeClick={onNodeClick}
          onNodeDragStop={onNodeDragStop}
          onInit={setReactFlowInstance}
          onDragOver={onDragOver}
          onDrop={onDrop}
          onPaneClick={onPaneClick}
          onConnect={onConnect}
          onNodesDelete={onNodesDelete}
          onMoveStart={onMoveStart}
          onMoveEnd={onMoveEnd}
          nodeTypes={nodeTypes}
          edgeTypes={edgeTypes}
          nodesDraggable={isManualMode}
          nodesConnectable={activeTool === "connect"}
          elementsSelectable={true}
          minZoom={0.1}
          maxZoom={2}
        >
          <Background color={backgroundPatternColor} gap={16} />
          <Controls
            className="sruja-controls"
            style={{
              display: "flex",
              flexDirection: "column",
              gap: "4px",
              padding: "4px",
              backgroundColor: isDark ? "#1a1a1a" : "#fff",
              border: `1px solid ${isDark ? "#333" : "#ddd"}`,
              borderRadius: "4px",
              boxShadow: "0 2px 5px rgba(0,0,0,0.1)",
            }}
          />
          <MiniMap
            nodeStrokeColor={(n) => {
              if (n.type === "input") return "#0041d0";
              if (n.type === "output") return "#ff0072";
              if (n.type === "default") return "#1a192b";
              return "#eee";
            }}
            nodeColor={(n) => {
              if (n.style?.background) return n.style.background as string;
              return "#fff";
            }}
            nodeBorderRadius={2}
            maskColor={isDark ? "rgba(0,0,0,0.3)" : "rgba(240,240,240,0.3)"}
            style={{
              backgroundColor: isDark ? "#1a1a1a" : "#fff",
              border: `1px solid ${isDark ? "#333" : "#ddd"}`,
            }}
          />
        </ReactFlow>
      </div>

      {/* Visual Editor Toolbar */}
      <Paper
        shadow="md"
        p="sm"
        radius="md"
        withBorder
        data-testid="visual-editor-toolbar"
        style={{
          position: "absolute",
          top: 16,
          left: 16,
          zIndex: 1000,
          backgroundColor: isDark ? "rgba(30, 30, 30, 0.95)" : "rgba(255, 255, 255, 0.95)",
          backdropFilter: "blur(8px)",
        }}
      >
        <VisualEditorToolbar />
      </Paper>

      {/* Node Palette - Show when create-node tool is active */}
      {activeTool === "create-node" && (
        <div
          data-testid="node-palette"
          style={{
            position: "absolute",
            top: 80,
            left: 16,
            zIndex: 1000,
          }}
        >
          <NodePalette />
        </div>
      )}

      {/* SRE Chaos Mode Controls - Available for all users */}
      {(showChaosPanel || chaosState.enabled) && (
        <Paper
          shadow="md"
          p="md"
          radius="md"
          withBorder
          style={{
            position: "absolute",
            top: 60, // Aligned below the top-right tools toolbar
            right: 16, // Aligned with the right edge
            zIndex: 1000,
            width: 300,
            backgroundColor: isDark ? "rgba(30, 30, 30, 0.95)" : "rgba(255, 255, 255, 0.95)",
            backdropFilter: "blur(8px)", // Consistent glass effect
          }}
        >
          <Stack>
            <Group justify="space-between">
              <Text fw={700} size="sm">
                Chaos Engineering Mode
              </Text>
              <Badge color={chaosState.enabled ? "error" : "neutral"}>
                {chaosState.enabled ? "ACTIVE" : "OFF"}
              </Badge>
            </Group>

            <Text size="xs" c="dimmed">
              Simulate service failures to verify system resilience and identify blast radius.
            </Text>

            <Button
              color={chaosState.enabled ? "red" : "blue"}
              variant={chaosState.enabled ? "outline" : "filled"}
              onClick={() => {
                setChaosEnabled(!chaosState.enabled);
                if (chaosState.enabled) setFailedNode(null); // Reset when disabling
              }}
            >
              {chaosState.enabled ? "Exit Simulation" : "Start Simulation"}
            </Button>

            {chaosState.enabled && (
              <Text size="xs" fw={500} c="orange">
                {chaosState.failedNodeId
                  ? `Simulating failure: ${getNodeTitle(chaosState.failedNodeId)}`
                  : "Click any node to simulate failure"}
              </Text>
            )}

            {chaosState.enabled && chaosState.failedNodeId && (
              <Group gap="xs">
                <Badge color="error">Failed: 1</Badge>
                <Badge color="warning">Impacted: {impactedNodeIds.size}</Badge>
              </Group>
            )}
          </Stack>
        </Paper>
      )}
      {/* Animation Controls & Overlay */}
      {activeAnimation && (
        <>
          <div
            style={{
              position: "absolute",
              bottom: "20px",
              left: "50%",
              transform: "translateX(-50%)",
              zIndex: 5,
              pointerEvents: "auto", // Ensure clicks work
            }}
          >
            <AnimationControls animation={activeAnimation} />
          </div>

          <StepDescriptionOverlay
            currentStep={animationStep}
            totalSteps={activeAnimation.steps?.length ?? 0}
            stepData={activeAnimation.steps?.[animationStep] ?? null}
          />
        </>
      )}
    </div>
  );
};
