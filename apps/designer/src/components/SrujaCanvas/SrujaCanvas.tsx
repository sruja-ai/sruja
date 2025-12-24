import { useState, useEffect, useCallback, useMemo, useRef } from "react";
import {
  ReactFlow,
  Background,
  Controls,
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
import { Paper, Group, Stack, ActionIcon, Text, Loader, Button } from "@mantine/core";
import { useTheme } from "@sruja/ui";

import { useArchitectureStore, useSelectionStore } from "../../stores";
import { trackInteraction } from "@sruja/shared";
import { layoutWithRefinement } from "./layoutEngine";
import { SrujaNode } from "./SrujaNode";
import type { C4Node, C4Level } from "./types";
import { ArrowLeft } from "lucide-react";

import { convertDslToDot, type SrujaModelDump } from "@sruja/shared";
import { calculateNodeSize } from "./textMeasure";
import type { EdgeType } from "./types";
import SplineEdge from "./SplineEdge";

const nodeTypes: NodeTypes = {
  sruja: SrujaNode,
};

const edgeTypes: EdgeTypes = {
  spline: SplineEdge,
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
  const model = useArchitectureStore((s) => s.likec4Model) as unknown as SrujaModelDump | null;
  const dslSource = useArchitectureStore((s) => s.dslSource) as string | null;
  const currentExampleFile = useArchitectureStore((s) => s.currentExampleFile);
  // Use currentExampleFile or dslSource hash as model identifier for cache invalidation
  const modelId =
    currentExampleFile || (dslSource ? `${dslSource.length}-${dslSource.substring(0, 50)}` : null);

  // View State
  const [level, setLevel] = useState<C4Level>(1);
  const [focusNodeId, setFocusNodeId] = useState<string | undefined>(undefined);
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

  // Layout Cache
  const cacheRef = useRef<LayoutCache>({});

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
      const cacheKey = hashCacheKey(level, focusNodeId, collapsedNodeIds, modelId);
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
          console.warn("[SrujaCanvas] No DSL source available for DOT generation");
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

        const result = await convertDslToDot(dslSource, level, focusNodeId, nodeSizes);

        if (!result || !result.dot) {
          console.warn("[SrujaCanvas] DOT generation failed or returned empty");
          setNodes([]);
          setEdges([]);
          return;
        }

        console.log("[SrujaCanvas] DOT generated via Go WASM, length:", result.dot.length);
        console.log("[SrujaCanvas] Projected Relations count:", result.relations.length);

        // 2. Layout with iterative refinement
        const { layoutResult, quality } = await layoutWithRefinement(result.dot, result.relations);

        if (quality) {
          console.log(
            `[SrujaCanvas] Layout quality: score=${quality.score.toFixed(2)}, crossings=${quality.edgeCrossings}, overlaps=${quality.nodeOverlaps}`
          );
        }

        // 3. Build C4Nodes from layout result and model metadata
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
        const c4Edges = result.relations.map((rel: any, idx: number) => {
          return {
            id: `e-${rel.from}-${rel.to}-${idx}`,
            source: rel.from,
            target: rel.to,
            label: rel.label || "",
            technology: undefined,
          };
        });

        console.log(`[SrujaCanvas] Layout: ${c4Nodes.length} nodes, ${c4Edges.length} edges`);

        // Expose quality metrics to window for e2e tests and UI
        if (quality && typeof window !== "undefined") {
          (window as any).__DIAGRAM_QUALITY__ = {
            score: quality.score,
            edgeCrossings: quality.edgeCrossings,
            nodeOverlaps: quality.nodeOverlaps,
            labelOverlaps: quality.labelOverlaps,
            avgEdgeLength: quality.avgEdgeLength,
            edgeLengthVariance: quality.edgeLengthVariance,
            rankAlignment: quality.rankAlignment,
            clusterBalance: quality.clusterBalance,
            spacingConsistency: quality.spacingConsistency,
            timestamp: Date.now(),
            nodeCount: c4Nodes.length,
            edgeCount: c4Edges.length,
            level: level,
          };
        }

        // Early return if no nodes
        if (c4Nodes.length === 0) {
          setNodes([]);
          setEdges([]);
          return;
        }

        // 5. React Flow Mapping
        const nextNodes: RFNode[] = c4Nodes.map((node) => {
          const layout = layoutResult.nodes.find((n) => n.id === node.id);
          return {
            id: node.id,
            type: "sruja",
            position: {
              x: layout ? layout.x : 0,
              y: layout ? layout.y : 0,
            },
            data: { ...node, _theme: mode } as C4Node & Record<string, unknown>,
            width: node.width,
            height: node.height,
          };
        });

        // Create a map of nodes for quick lookup
        const nodeMap = new Map(nextNodes.map((n) => [n.id, n]));

        // Build map of Graphviz edge spline points for lookup
        const gvEdgeMap = new Map<string, Array<[number, number]> | undefined>();
        layoutResult.edges.forEach((gvEdge) => {
          // Key by source-target
          const key = `${gvEdge.source}-${gvEdge.target}`;
          gvEdgeMap.set(key, gvEdge.points);
        });
        console.log(`[SrujaCanvas] Graphviz edge map has ${gvEdgeMap.size} entries`);

        // Filter and create edges only for nodes that exist
        const rfValidNodeIds = new Set(nextNodes.map((n) => n.id));
        const nextEdges: RFEdge[] = c4Edges
          .filter((edge) => {
            const sourceExists = rfValidNodeIds.has(edge.source);
            const targetExists = rfValidNodeIds.has(edge.target);
            if (!sourceExists || !targetExists) {
              console.warn(
                `[SrujaCanvas] Edge ${edge.id} skipped: source=${edge.source} exists=${sourceExists}, target=${edge.target} exists=${targetExists}`
              );
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
        if (nextEdges.length > 0) {
          console.log(
            "[SrujaCanvas] Sample React Flow edges:",
            nextEdges.slice(0, 3).map((e) => ({
              id: e.id,
              source: e.source,
              target: e.target,
              label: e.label,
              labelShowBg: e.labelShowBg,
              labelStyle: e.labelStyle,
              labelBgStyle: e.labelBgStyle,
              sourceHandle: e.sourceHandle,
              targetHandle: e.targetHandle,
            }))
          );
          // Log raw C4 edges for comparison
          console.log(
            "[SrujaCanvas] Sample C4 edges (projected):",
            c4Edges.slice(0, 3).map((e) => ({
              id: e.id,
              source: e.source,
              target: e.target,
              label: e.label,
            }))
          );
        } else if (c4Edges.length > 0) {
          console.warn(
            "[SrujaCanvas] WARNING: Had projected edges but none made it to React Flow!"
          );
          console.warn("[SrujaCanvas] Projected edges:", c4Edges);
          console.warn("[SrujaCanvas] Valid node IDs:", Array.from(rfValidNodeIds));
        }

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
        console.error("Layout failed:", err);
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

  // Navigation Handlers
  const onNodeClick = useCallback(
    (event: React.MouseEvent, node: RFNode) => {
      const c4Data = node.data as unknown as C4Node;

      // Check if clicking on expand/collapse button (handled separately)
      const target = event.target as HTMLElement;
      if (target.closest(".expand-btn")) {
        return;
      }

      // Select the node to open details panel
      selectNode(node.id);

      // Logic: Drill down if possible (but still select the node)
      if (level === 1 && c4Data.kind === "system" && c4Data.navigateOnClick) {
        const targetId = c4Data.navigateOnClick;
        setFocusNodeId(targetId);
        setLevel(2);
        trackInteraction("drill-down", "node", {
          fromLevel: 1,
          toLevel: 2,
          nodeId: node.id,
          nodeKind: c4Data.kind,
        });
      } else if (level === 2 && c4Data.kind === "container" && c4Data.navigateOnClick) {
        const targetId = c4Data.navigateOnClick;
        setFocusNodeId(targetId);
        setLevel(3);
        trackInteraction("drill-down", "node", {
          fromLevel: 2,
          toLevel: 3,
          nodeId: node.id,
          nodeKind: c4Data.kind,
        });
      } else {
        // Just selecting a node without drilling down
        trackInteraction("select", "node", { nodeId: node.id, nodeKind: c4Data.kind, level });
      }
    },
    [level, selectNode, setFocusNodeId, setLevel]
  );

  const onGoUp = useCallback(() => {
    const previousLevel = level;
    if (level === 3 && model && focusNodeId) {
      // L3 -> L2: Find parent system
      const container = model.elements[focusNodeId];
      if (container?.parent) {
        const parentId = container.parent;
        setFocusNodeId(parentId);
        setLevel(2);
        trackInteraction("navigate", "level", { from: previousLevel, to: 2, nodeId: parentId });
        return;
      }
    }

    if (level === 2) {
      // L2 -> L1: Clear focus
      setLevel(1);
      setFocusNodeId(undefined);
      trackInteraction("navigate", "level", { from: previousLevel, to: 1 });
    }
  }, [level, focusNodeId, model]);

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

  const handleBreadcrumbClick = useCallback(
    (targetLevel: C4Level, targetId?: string) => {
      const previousLevel = level;
      if (targetLevel === 1) {
        setLevel(1 as C4Level);
        setFocusNodeId(undefined);
        trackInteraction("navigate", "breadcrumb", { from: previousLevel, to: 1 });
      } else if (targetLevel === 2 && targetId) {
        setLevel(2 as C4Level);
        setFocusNodeId(targetId);
        trackInteraction("navigate", "breadcrumb", {
          from: previousLevel,
          to: 2,
          nodeId: targetId,
        });
      } else if (targetLevel === 3 && targetId) {
        setLevel(3 as C4Level);
        setFocusNodeId(targetId);
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
        onInit={setReactFlowInstance}
        nodeTypes={nodeTypes}
        edgeTypes={edgeTypes}
        fitView
        minZoom={0.1}
        maxZoom={2}
      >
        <Background color={backgroundPatternColor} gap={16} />
        <Controls />
      </ReactFlow>
    </div>
  );
};
