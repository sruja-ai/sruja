import { useState, useEffect, useCallback, useMemo, useRef } from "react";
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
  MarkerType,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { Paper, Group, Stack, ActionIcon, Text, Loader, Button, Badge } from "@mantine/core";
import { useTheme } from "@sruja/ui";

import { useArchitectureStore, useSelectionStore, useUIStore } from "../../stores";
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
import { ArrowLeft, Edit3 } from "lucide-react";
import { type LayoutQuality, type ParentChildRelationships } from "./qualityMetrics";

import { convertDslToDot, type SrujaModelDump } from "@sruja/shared";
type ElementDump = NonNullable<SrujaModelDump["elements"]>[string];
import { calculateNodeSize } from "./textMeasure";
import type { EdgeType } from "./types";
import SplineEdge from "./SplineEdge";
import TrafficEdge from "./TrafficEdge";
import { useViewStore } from "../../stores/viewStore"; // Ensure view store is imported

const nodeTypes: NodeTypes = {
  sruja: SrujaNode,
  group: GroupNode, // Parent container nodes
};

const edgeTypes: EdgeTypes = {
  spline: SplineEdge,
  traffic: TrafficEdge,
};

/**
 * Select optimal handle positions based on node positions.
 * Chooses the closest sides between source and target nodes.
 */
function selectOptimalHandles(
  sourceNode: RFNode,
  targetNode: RFNode
): { sourceHandle: string; targetHandle: string } {
  const sourceX = sourceNode.position.x + (sourceNode.width || 0) / 2;
  const sourceY = sourceNode.position.y + (sourceNode.height || 0) / 2;
  const targetX = targetNode.position.x + (targetNode.width || 0) / 2;
  const targetY = targetNode.position.y + (targetNode.height || 0) / 2;

  const dx = targetX - sourceX;
  const dy = targetY - sourceY;
  const absDx = Math.abs(dx);
  const absDy = Math.abs(dy);

  // Determine primary direction
  let sourceHandle: string;
  let targetHandle: string;

  if (absDx > absDy) {
    // Horizontal layout - prefer left/right
    if (dx > 0) {
      // Target is to the right
      sourceHandle = "source-right";
      targetHandle = "target-left";
    } else {
      // Target is to the left
      sourceHandle = "source-left";
      targetHandle = "target-right";
    }
  } else {
    // Vertical layout - prefer top/bottom
    if (dy > 0) {
      // Target is below
      sourceHandle = "source-bottom";
      targetHandle = "target-top";
    } else {
      // Target is above
      sourceHandle = "source-top";
      targetHandle = "target-bottom";
    }
  }

  return { sourceHandle, targetHandle };
}

/**
 * Simple hash function for cache keys
 * Includes model identifier to invalidate cache when model changes
 */
function hashCacheKey(
  level: C4Level,
  focusNodeId: string | undefined,
  collapsedNodeIds: Set<string>,
  modelId?: string | null
): string {
  const collapsedArray = Array.from(collapsedNodeIds).sort().join(",");
  // Include modelId in cache key to ensure cache is invalidated when model changes
  const modelHash = modelId ? modelId.substring(0, 16) : "no-model";
  return `${level}:${focusNodeId || ""}:${collapsedArray}:${modelHash}`;
}

interface LayoutCache {
  [key: string]: {
    nodes: RFNode[];
    edges: RFEdge[];
    timestamp: number;
  };
}

const CACHE_TTL = 5 * 60 * 1000; // 5 minutes

export const SrujaCanvas = () => {
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
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [collapsedNodeIds, _setCollapsedNodeIds] = useState<Set<string>>(new Set());
  const [isComputing, setIsComputing] = useState(false);
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

  // Layout Cache
  const cacheRef = useRef<LayoutCache>({});

  // Store actions
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);

  // Chaos Mode State
  const chaosState = useArchitectureStore((s) => s.chaosState);
  const setFailedNode = useArchitectureStore((s) => s.setFailedNode);
  const setChaosEnabled = useArchitectureStore((s) => s.setChaosEnabled);
  const selectedPersona = useUIStore((s) => s.selectedPersona);

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
            let id = base.toLowerCase().replace(/[^a-z0-9]/g, "");
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
              description: comp.description as any,
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

  // Pipeline Execution with Caching
  // Include theme in dependencies to force re-render when theme changes
  useEffect(() => {
    if (!model) {
      setNodes([]);
      setEdges([]);
      return;
    }

    const computeLayout = async () => {
      const cacheKey =
        hashCacheKey(level, focusNodeId, collapsedNodeIds, modelId) +
        (activeViewId ? `:${activeViewId}` : "");
      const cached = cacheRef.current[cacheKey];

      // Check cache validity
      if (cached && Date.now() - cached.timestamp < CACHE_TTL) {
        setNodes(cached.nodes);
        setEdges(cached.edges);
        // Fit view after a short delay to ensure nodes are rendered
        setTimeout(() => {
          reactFlowInstance?.fitView({ padding: 0.2 });
        }, 100);
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
        const nodeSizes: Record<string, { width: number; height: number }> = {};
        if (model) {
          Object.values(model.elements).forEach((elem) => {
            const size = calculateNodeSize(
              elem.title,
              elem.technology ?? undefined,
              typeof elem.description === "string" ? elem.description : undefined,
              elem.kind
            );
            nodeSizes[elem.id] = size;
          });
        }

        const result = await convertDslToDot(
          dslSource,
          activeViewId ? 0 : level, // If activeViewId set, pass 0 level? Need to confirm WASM behavior
          activeViewId || focusNodeId,
          nodeSizes
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
        const parentChildRelationships: ParentChildRelationships = {
          childToParent: new Map<string, string>(),
        };
        if (model && model.elements) {
          for (const element of Object.values(model.elements)) {
            // Only include relationships where BOTH child and parent are visible in current view
            if (
              element.parent &&
              typeof element.parent === "string" &&
              visibleNodeIds.has(element.id) &&
              visibleNodeIds.has(element.parent)
            ) {
              parentChildRelationships.childToParent.set(element.id, element.parent);
            }
          }
        }

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
        const c4Nodes: C4Node[] = layoutResult.nodes.map((layoutNode) => {
          // Get element metadata from model
          const element = model.elements[layoutNode.id];
          const kind = element?.kind?.toLowerCase() || "container";

          const measuredSize = nodeSizes[layoutNode.id];

          return {
            id: layoutNode.id,
            kind: kind as C4Node["kind"],
            title: element?.title || layoutNode.id,
            technology: element?.technology ?? undefined,
            description: typeof element?.description === "string" ? element.description : undefined,
            level: level as C4Level,
            width: measuredSize?.width || layoutNode.width || 200,
            height: measuredSize?.height || layoutNode.height || 120,
          };
        });

        // 4. Build edges from projected relations returned by Go
        const c4Edges = result.relations.map(
          (rel: { from: string; to: string; title?: string }, idx: number) => {
            return {
              id: `e-${rel.from}-${rel.to}-${idx}`,
              source: rel.from,
              target: rel.to,
              label: (rel as any).label || "",
              technology: undefined,
            };
          }
        );

        // Layout complete: nodes and edges computed

        // Expose quality metrics to window for e2e tests and UI (dev only)
        // Quality metrics are developer tools, not user-facing features
        if (quality && isDev && typeof window !== "undefined") {
          const qualityMetrics = {
            score: quality.score,
            edgeCrossings: quality.edgeCrossings,
            nodeOverlaps: quality.nodeOverlaps,
            labelOverlaps: quality.labelOverlaps,
            parentChildContainment: quality.parentChildContainment,
            avgEdgeLength: quality.avgEdgeLength,
            edgeLengthVariance: quality.edgeLengthVariance,
            rankAlignment: quality.rankAlignment,
            clusterBalance: quality.clusterBalance,
            spacingConsistency: quality.spacingConsistency,
            timestamp: Date.now(),
            nodeCount: c4Nodes.length,
            edgeCount: c4Edges.length,
            level: level > 0 ? `L${level}` : "L1",
          } as LayoutQuality & {
            level: string;
            nodeCount: number;
            edgeCount: number;
            timestamp: number;
          };

          (window as Window & { __DIAGRAM_QUALITY__?: typeof qualityMetrics }).__DIAGRAM_QUALITY__ =
            qualityMetrics;

          // Also expose to __LAYOUT_METRICS__ for e2e tests (matches test expectations)
          (
            window as Window & {
              __LAYOUT_METRICS__?: Record<string, unknown>;
            }
          ).__LAYOUT_METRICS__ = {
            ...qualityMetrics,
            // Export detailed parent-child containment violations as array (expected by tests)
            parentChildContainment: parentChildContainmentViolations,
          };

          logger.debug("Diagram quality metrics", {
            component: "SrujaCanvas",
            action: "calculateLayout",
            metrics: (window as { __DIAGRAM_QUALITY__?: unknown }).__DIAGRAM_QUALITY__,
          });
        }

        // Early return if no nodes
        if (c4Nodes.length === 0) {
          setNodes([]);
          setEdges([]);
          return;
        }

        // 5. React Flow Mapping
        // Check for saved manual positions in view metadata
        // Try both formats: manual-layout-* (from UI edits) and regular view IDs (from DSL)
        const viewKey = `manual-layout-${level}-${focusNodeId || "root"}`;
        const savedPositions = model.views?.[viewKey];
        // Support both old format (layout.positions) and new format (Layout.Positions)
        // Type assertion needed because layout positions are not in ParsedView type
        type ViewWithLayout = typeof savedPositions & {
          layout?: { positions?: Record<string, { x: number; y: number }> };
          Layout?: {
            Positions?: Record<string, { x: number; y: number }>;
            positions?: Record<string, { x: number; y: number }>;
          };
        };
        const viewWithLayout = savedPositions as ViewWithLayout | undefined;
        const manualPositionsMap =
          viewWithLayout?.layout?.positions ||
          viewWithLayout?.Layout?.Positions ||
          viewWithLayout?.Layout?.positions ||
          {};

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
            console.log(
              `[SrujaCanvas] Using compound node structure with ${Object.keys(layoutResult.clusters).length} clusters`
            );
          } else {
            console.log("[SrujaCanvas] No clusters found - using flat node structure");
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

            // Chaos Mode Styling
            const isFailed = chaosState.enabled && chaosState.failedNodeId === node.id;
            const isImpacted = chaosState.enabled && impactedNodeIds.has(node.id);
            const isDimmed =
              chaosState.enabled && !!chaosState.failedNodeId && !isFailed && !isImpacted;

            // Capacity Planning Calculation
            const capacityState = useArchitectureStore.getState().capacityState;
            const loadMultiplier = capacityState.userLoad / 100;
            // Simple heuristic: 3 replicas base * load multiplier
            // Only for containers (services)
            const replicas = node.kind === "container" ? Math.ceil(3 * loadMultiplier) : undefined;

            return {
              id: node.id,
              type: "sruja",
              position,
              data: {
                ...node,
                _theme: mode,
                // Pass chaos flags
                _chaos: {
                  isFailed,
                  isImpacted,
                  isDimmed,
                },
                // Pass capacity metrics
                _capacity: replicas
                  ? {
                      replicas,
                      load: capacityState.userLoad,
                    }
                  : undefined,
              } as C4Node & Record<string, unknown>,
              width: node.width,
              height: node.height,
              style: {
                // Apply z-index boost for important nodes
                zIndex: isFailed ? 999 : isImpacted ? 998 : 1,
                opacity: isDimmed ? 0.3 : 1,
                filter: isDimmed ? "grayscale(0.8)" : "none",
              },
            };
          });
        }

        // Apply chaos styling and capacity metrics to all nodes (both flat and compound)
        nextNodes = nextNodes.map((node) => {
          const isFailed = chaosState.enabled && chaosState.failedNodeId === node.id;
          const isImpacted = chaosState.enabled && impactedNodeIds.has(node.id);
          const isDimmed =
            chaosState.enabled && !!chaosState.failedNodeId && !isFailed && !isImpacted;

          const capacityState = useArchitectureStore.getState().capacityState;
          const loadMultiplier = capacityState.userLoad / 100;
          const replicas =
            node.data?.kind === "container" ? Math.ceil(3 * loadMultiplier) : undefined;

          return {
            ...node,
            data: {
              ...node.data,
              _theme: mode,
              _chaos: {
                isFailed,
                isImpacted,
                isDimmed,
              },
              _capacity: replicas
                ? {
                    replicas,
                    load: capacityState.userLoad,
                  }
                : undefined,
            },
            style: {
              ...node.style,
              zIndex: isFailed ? 999 : isImpacted ? 998 : (node.style?.zIndex ?? 1),
              opacity: isDimmed ? 0.3 : (node.style?.opacity ?? 1),
              filter: isDimmed ? "grayscale(0.8)" : (node.style?.filter ?? "none"),
            },
          };
        });

        // Recalculate quality metrics using actual rendered positions (for compound nodes)
        // This ensures parent-child containment is checked against actual parent container bounding boxes
        if (isDev && hasClusters && nextNodes.length > 0) {
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
            const splinePoints = gvEdgeMap.get(`${edge.source}-${edge.target}`);
            const useSpline = splinePoints && splinePoints.length > 0;

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
        console.log(
          `[SrujaCanvas] Layout complete: ${nextNodes.length} nodes, ${nextEdges.length} edges (from ${c4Edges.length} projected edges)`
        );
        // Edges processed and validated

        // Cache the result
        cacheRef.current[cacheKey] = {
          nodes: nextNodes,
          edges: nextEdges,
          timestamp: Date.now(),
        };

        setNodes(nextNodes);
        setEdges(nextEdges);

        // Fit view after layout
        setTimeout(() => {
          reactFlowInstance?.fitView({ padding: 0.2 });
        }, 100);
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
    setNodes,
    setEdges,
    reactFlowInstance,
    mode,
    isDark,
    uiTheme,
  ]); // Include modelId to invalidate cache when model changes

  // Selection store for details panel
  const selectNode = useSelectionStore((s) => s.selectNode);

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
        updateArchitecture((model) => {
          const updatedModel = { ...model };

          // Store positions in view metadata
          // Use current view context (level + focusNodeId) as view key
          const viewKey = `manual-layout-${level}-${focusNodeId || "root"}`;

          if (!updatedModel.views) {
            updatedModel.views = {};
          } else {
            // Create mutable copy of views
            updatedModel.views = { ...updatedModel.views };
          }

          // Use type assertion to avoid readonly error (since we cloned it, it's safe to mutate our copy)
          const views = updatedModel.views as Record<string, any>;

          if (!views[viewKey]) {
            views[viewKey] = {
              id: viewKey,
              title: `Manual Layout ${level > 1 ? `L${level}` : "L1"}`,
              rules: [],
              nodes: [],
              edges: [],
            };
          }

          type ViewWithLayoutMutable = (typeof views)[string] & {
            layout?: { positions?: Record<string, { x: number; y: number }> };
          };
          const view = views[viewKey] as ViewWithLayoutMutable;
          if (!view.layout) {
            view.layout = {};
          }
          if (!view.layout.positions) {
            view.layout.positions = {};
          }

          view.layout.positions[node.id] = {
            x: node.position.x,
            y: node.position.y,
          };

          return updatedModel;
        });
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

  // Check if current view has manual positions
  useEffect(() => {
    if (model) {
      const viewKey = `manual-layout-${level}-${focusNodeId || "root"}`;
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
      selectNode(node.id);

      // Select the node to open details panel
      selectNode(node.id);

      // Just selecting a node without drill-down (Single Click behavior)
      trackInteraction("select", "node", { nodeId: node.id, nodeKind: c4Data.kind, level });

      // Handle double click for drill down
      const clickTime = Date.now();
      const lastClick = (node as any)._lastClick || 0;
      (node as any)._lastClick = clickTime;

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
      {/* Breadcrumb / Navigation Header */}
      <Paper
        shadow="md"
        p="xs"
        withBorder
        style={{
          position: "absolute",
          top: 16,
          left: 16,
          zIndex: 10,
          backgroundColor: paperBg,
          backdropFilter: "blur(8px)",
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
        nodeTypes={nodeTypes}
        edgeTypes={edgeTypes}
        nodesDraggable={true}
        nodesConnectable={false}
        elementsSelectable={true}
        fitView
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

      {/* SRE Chaos Mode Controls */}
      {selectedPersona === "sre" && (
        <Paper
          shadow="md"
          p="md"
          radius="md"
          withBorder
          style={{
            position: "absolute",
            top: 80, // Moved down to avoid breadcrumbs
            right: 20,
            zIndex: 1000,
            width: 300,
            backgroundColor: isDark ? "rgba(30, 30, 30, 0.95)" : "rgba(255, 255, 255, 0.95)",
          }}
        >
          <Stack>
            <Group justify="space-between">
              <Text fw={700} size="sm">
                Chaos Engineering Mode
              </Text>
              <Badge color={chaosState.enabled ? "red" : "gray"}>
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
                <Badge variant="dot" color="red">
                  Failed: 1
                </Badge>
                <Badge variant="dot" color="orange">
                  Impacted: {impactedNodeIds.size}
                </Badge>
              </Group>
            )}
          </Stack>
        </Paper>
      )}
    </div>
  );
};
