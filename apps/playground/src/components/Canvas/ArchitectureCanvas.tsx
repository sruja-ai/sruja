import { useCallback, useMemo, useEffect, useState, useRef } from "react";
import {
  ReactFlow,
  Controls,
  MiniMap,
  Background,
  useNodesState,
  useEdgesState,
  type Node,
  type Edge,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";

import { nodeTypes } from "../Nodes";
import { FlowController } from "../Flow";
import { Legend } from "../Legend";
import { jsonToReactFlow } from "../../utils/jsonToReactFlow";
import { applySrujaLayout } from "../../utils/layoutEngine";
import { applyC4LevelLayout } from "../../utils/c4LevelLayout";
import { applyIncrementalLayout } from "../../utils/incrementalLayout";
import { PositionPreservation } from "../../utils/positionPreservation";
import { layoutWorkerClient } from "../../utils/layoutWorkerClient";
import { useArchitectureStore, useViewStore, useSelectionStore } from "../../stores";
import { useFeatureFlagsStore } from "../../stores/featureFlagsStore";
import { useGovernanceCounts, enrichNodesWithGovernance } from "../../hooks/useGovernanceCounts";
import type { C4NodeData } from "../../types";
import { BoundaryNode } from "../Nodes/BoundaryNode";
import PolylineEdge from "../Edges/PolylineEdge";
import { RelationEdge } from "../Edges/RelationEdge";
import { HeatmapCanvasOverlay } from "./HeatmapCanvasOverlay";
import { calculateDiagramQuality, type DiagramQualityMetrics } from "../../utils/diagramQuality";
import "./ArchitectureCanvas.css";

interface ArchitectureCanvasProps {
  dragEnabled?: boolean;
}

export function ArchitectureCanvas({ dragEnabled = true }: ArchitectureCanvasProps) {
  const data = useArchitectureStore((s) => s.data);

  const currentLevel = useViewStore((s) => s.currentLevel);
  const focusedSystemId = useViewStore((s) => s.focusedSystemId);
  const focusedContainerId = useViewStore((s) => s.focusedContainerId);
  const expandedNodes = useViewStore((s) => s.expandedNodes);
  const selectNode = useSelectionStore((s) => s.selectNode);

  const [layoutReady, setLayoutReady] = useState(false);
  const [highlightedEdge, setHighlightedEdge] = useState<{ from: string; to: string } | null>(null);
  const [c4Mode, setC4Mode] = useState<"rich" | "pure">("pure"); // Default to pure C4 style
  const [showHeatmap, setShowHeatmap] = useState(false);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const isEditMode = useFeatureFlagsStore((s) => s.isEditMode);
  const [fabOpen, setFabOpen] = useState(false);
  const [createType, setCreateType] = useState<
    "system" | "person" | "container" | "component" | null
  >(null);
  const [createName, setCreateName] = useState("");
  const [createTech, setCreateTech] = useState("");
  const [createExternal, setCreateExternal] = useState(false);

  // Position preservation for stable layouts
  const positionPreservationRef = useRef<PositionPreservation>(new PositionPreservation());
  const previousExpandedNodesRef = useRef<Set<string>>(new Set());
  const previousLevelRef = useRef<string>(currentLevel);
  const previousFocusedSystemRef = useRef<string | null>(focusedSystemId);
  const previousFocusedContainerRef = useRef<string | null>(focusedContainerId);
  const hasLaidOutNodesRef = useRef<boolean>(false);

  // Governance counts for badge display
  const governanceCounts = useGovernanceCounts();

  // Transform JSON to React Flow nodes/edges
  const { rawNodes, rawEdges } = useMemo(() => {
    if (!data) {
      return { rawNodes: [], rawEdges: [] };
    }
    console.log("Recalculating rawNodes with expandedNodes:", Array.from(expandedNodes));
    const result = jsonToReactFlow(data, {
      level: currentLevel,
      focusedSystemId: focusedSystemId ?? undefined,
      focusedContainerId: focusedContainerId ?? undefined,
      expandedNodes,
    });

    // Enrich nodes with governance counts (requirements/ADRs linked via tags)
    const enrichedNodes = enrichNodesWithGovernance(result.nodes, governanceCounts);

    console.log(
      "Generated nodes:",
      enrichedNodes.length,
      "edges:",
      result.edges.length,
      "IDs:",
      enrichedNodes.map((n) => n.id)
    );
    return { rawNodes: enrichedNodes, rawEdges: result.edges };
  }, [data, currentLevel, focusedSystemId, focusedContainerId, expandedNodes, governanceCounts]);

  const [nodes, setNodes, onNodesChange] = useNodesState<Node<C4NodeData>>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);

  // Calculate layout quality metrics
  const metrics = useMemo<DiagramQualityMetrics | null>(() => {
    if (nodes.length === 0) return null;
    // Check availability of window for dimension
    const viewport = {
      width: window.innerWidth,
      height: window.innerHeight,
    };
    return calculateDiagramQuality(nodes, edges, viewport);
  }, [nodes, edges]);

  // Define nodeTypes properly
  const finalNodeTypes = useMemo(() => ({ ...nodeTypes, boundary: BoundaryNode }), []);
  const edgeTypes = useMemo(() => ({ polyline: PolylineEdge, relation: RelationEdge }), []);

  const [layoutEngine, setLayoutEngine] = useState<"sruja" | "c4level">("c4level");
  const previousEffectiveEngineRef = useRef<"sruja" | "c4level">("c4level");
  const lastLayoutWasIncrementalRef = useRef<boolean>(false);

  // ...

  // Apply layout when data changes
  useEffect(() => {
    let cancelled = false;

    async function runLayout() {
      if (rawNodes.length === 0) {
        setNodes([]);
        setEdges([]);
        setLayoutReady(false);
        positionPreservationRef.current.clear();
        hasLaidOutNodesRef.current = false;
        return;
      }

      setLayoutReady(false);

      // Auto-group Internal nodes under "Enterprise Boundary" if external nodes exist
      // Only for System Context level (L1) or maybe L0?
      let layoutNodes = [...rawNodes];
      const hasExternal = rawNodes.some((n) => n.data.isExternal);
      const hasInternal = rawNodes.some((n) => !n.data.isExternal);

      if (hasExternal && hasInternal && layoutEngine === "sruja" && currentLevel === "L1") {
        const boundaryId = "enterprise-boundary";
        const boundaryNode: Node<C4NodeData> = {
          id: boundaryId,
          type: "boundary",
          position: { x: 0, y: 0 },
          data: {
            label: "Enterprise Scope",
            type: "system", // Mock type to satisfy C4NodeData if strict
          } as any,
          style: { width: 100, height: 100 }, // Sruja layout will resize this
        };

        layoutNodes = layoutNodes.map((n) => {
          if (!n.data.isExternal && !n.parentId) {
            return { ...n, parentId: boundaryId, expandParent: true };
          }
          return n;
        });
        layoutNodes.push(boundaryNode);
      }

      // Check if this is a major structural change (level change, focus change)
      const isMajorChange =
        previousLevelRef.current !== currentLevel ||
        previousFocusedSystemRef.current !== focusedSystemId ||
        previousFocusedContainerRef.current !== focusedContainerId;

      // Check if we should use incremental layout (position preservation)
      // Use incremental layout if:
      // 1. Not a major structural change
      // 2. We have previously laid out nodes
      // 3. It's just an expansion/collapse change (preserve positions)
      // More aggressive: prefer incremental for any non-major change
      const isJustExpansionChange =
        !isMajorChange &&
        expandedNodes.size !== previousExpandedNodesRef.current.size &&
        previousLevelRef.current === currentLevel &&
        previousFocusedSystemRef.current === focusedSystemId &&
        previousFocusedContainerRef.current === focusedContainerId;

      const shouldUseIncremental =
        !isMajorChange &&
        hasLaidOutNodesRef.current &&
        (isJustExpansionChange || // Always use incremental for expansion changes
          expandedNodes.size === previousExpandedNodesRef.current.size); // Or if nothing changed

      console.log("[ArchitectureCanvas] Layout Decision:", {
        isMajorChange,
        hasLaidOutNodes: hasLaidOutNodesRef.current,
        shouldUseIncremental,
        layoutEngine,
        expandedNodesCount: expandedNodes.size,
      });

      // Clear position preservation on major changes
      if (isMajorChange) {
        positionPreservationRef.current.clear();
        hasLaidOutNodesRef.current = false;
      }

      // Use rules-based layout selection (unless manual override)
      let effectiveEngine = layoutEngine;
      let effectiveDirection: "DOWN" | "RIGHT" | "UP" | "LEFT" = "DOWN";
      let layoutOptions: any = {};

      // If using auto mode (c4level), select based on rules
      if (layoutEngine === "c4level") {
        const { selectLayoutConfig } = await import("../../utils/layoutRules");
        const config = selectLayoutConfig(
          layoutNodes,
          rawEdges,
          currentLevel,
          focusedSystemId ?? undefined,
          focusedContainerId ?? undefined,
          expandedNodes
        );
        effectiveEngine = config.engine;
        effectiveDirection = config.direction;
        layoutOptions = config.options || {};
        // Keep engine stable during incremental updates to reduce reflow
        if (shouldUseIncremental && previousEffectiveEngineRef.current) {
          effectiveEngine = previousEffectiveEngineRef.current;
        }
      }

      let layoutPromise;

      try {
        performance.clearMarks("layout-start");
        performance.clearMarks("layout-end");
        performance.clearMeasures("layout");
      } catch {}
      performance.mark("layout-start");

      if (shouldUseIncremental) {
        // Use incremental layout with position preservation
        layoutPromise = applyIncrementalLayout(
          layoutNodes,
          rawEdges,
          {
            layoutEngine: effectiveEngine,
            direction: effectiveDirection,
            level: currentLevel,
            focusedSystemId: focusedSystemId ?? undefined,
            focusedContainerId: focusedContainerId ?? undefined,
            expandedNodes,
            previousExpandedNodes: previousExpandedNodesRef.current,
          },
          positionPreservationRef.current
        );
      } else {
        // Full re-layout (first load or major change)
        const useWorker = layoutNodes.length > 80 || rawEdges.length > 120;
        if (useWorker) {
          const options =
            effectiveEngine === "c4level"
              ? {
                  level: currentLevel,
                  focusedSystemId: focusedSystemId ?? undefined,
                  focusedContainerId: focusedContainerId ?? undefined,
                  expandedNodes,
                }
              : {
                  direction: effectiveDirection,
                  level: currentLevel,
                  expandedNodes,
                  ...layoutOptions,
                };
          layoutPromise = layoutWorkerClient.run(
            effectiveEngine,
            effectiveEngine === "c4level" ? rawNodes : layoutNodes,
            rawEdges,
            options
          );
        } else {
          if (effectiveEngine === "c4level") {
            layoutPromise = applyC4LevelLayout(rawNodes, rawEdges, {
              level: currentLevel,
              focusedSystemId: focusedSystemId ?? undefined,
              focusedContainerId: focusedContainerId ?? undefined,
              expandedNodes,
            });
          } else {
            layoutPromise = applySrujaLayout(layoutNodes, rawEdges, {
              direction: effectiveDirection,
              level: currentLevel,
              expandedNodes,
              ...layoutOptions,
            });
          }
        }
      }

      layoutPromise
        .then((result) => {
          performance.mark("layout-end");
          try {
            performance.measure("layout", "layout-start", "layout-end");
          } catch {}
          if (cancelled) return;
          setNodes(result.nodes);
          setEdges(result.edges);
          lastLayoutWasIncrementalRef.current = shouldUseIncremental;
          // Update position preservation with new node positions
          positionPreservationRef.current.updateFromNodes(result.nodes);
          // Update previous state
          previousExpandedNodesRef.current = new Set(expandedNodes);
          previousLevelRef.current = currentLevel;
          previousFocusedSystemRef.current = focusedSystemId;
          previousFocusedContainerRef.current = focusedContainerId;
          hasLaidOutNodesRef.current = true;
          previousEffectiveEngineRef.current = effectiveEngine;
          setLayoutReady(true);
        })
        .catch((err) => {
          console.error("Layout error:", err);
          const nodeIdSet = new Set(rawNodes.map((n) => n.id));
          const filteredEdges = rawEdges.filter(
            (e) => nodeIdSet.has(e.source) && nodeIdSet.has(e.target)
          );
          performance.mark("layout-end");
          try {
            performance.measure("layout", "layout-start", "layout-end");
          } catch {}
          if (cancelled) return;
          setNodes(rawNodes);
          setEdges(filteredEdges);
          // Still update position preservation even on error
          positionPreservationRef.current.updateFromNodes(rawNodes);
          previousExpandedNodesRef.current = new Set(expandedNodes);
          previousLevelRef.current = currentLevel;
          previousFocusedSystemRef.current = focusedSystemId;
          previousFocusedContainerRef.current = focusedContainerId;
          hasLaidOutNodesRef.current = true;
          previousEffectiveEngineRef.current = effectiveEngine;
          setLayoutReady(true);
        });
    }

    runLayout();
    return () => {
      cancelled = true;
    };
  }, [
    rawNodes,
    rawEdges,
    setNodes,
    setEdges,
    layoutEngine,
    currentLevel,
    focusedSystemId,
    focusedContainerId,
    expandedNodes,
  ]);

  // Expose graph state and metrics for testing/metrics
  useEffect(() => {
    (window as any).__CYBER_GRAPH__ = { nodes, edges };
    (window as any).__LAYOUT_METRICS__ = metrics;
    (window as any).__LAYOUT_HEATMAP_VISIBLE__ = showHeatmap;
    (window as any).__LAYOUT_TOGGLE_HEATMAP__ = setShowHeatmap;
  }, [nodes, edges, metrics, showHeatmap]);

  // Apply edge highlighting for flow animation
  useEffect(() => {
    if (!highlightedEdge) {
      // Reset edges to normal
      setEdges((eds) =>
        eds.map((e) => ({
          ...e,
          animated: false,
          zIndex: 0,
          data: { ...e.data, isActive: false, isDimmed: false },
        }))
      );
      // Reset nodes
      setNodes((nds) =>
        nds.map((n) => ({
          ...n,
          className: (n.className || "").replace(/ dimmed| highlighted/g, ""),
        }))
      );
    } else {
      // Highlight matching edge and dim others
      const activeNodeIds = new Set<string>();
      activeNodeIds.add(highlightedEdge.from);
      activeNodeIds.add(highlightedEdge.to);

      setEdges((eds) =>
        eds.map((e) => {
          // Match edge strictly for now, or use loose match if needed
          const isHighlighted =
            e.source === highlightedEdge.from && e.target === highlightedEdge.to;

          return {
            ...e,
            animated: false, // We use custom animation in RelationEdge
            zIndex: isHighlighted ? 10 : 0,
            data: {
              ...e.data,
              isActive: isHighlighted,
              isDimmed: !isHighlighted,
            },
          };
        })
      );

      setNodes((nds) =>
        nds.map((n) => {
          // Check if this node is active (exact match or parent of active node)
          // e.g. if active is System.Container, and this node is System, it should active?
          // Actually usually the other way: if active is System.Container, System logic depends on view.
          // Assuming resolved IDs match visible nodes.
          const isActive =
            activeNodeIds.has(n.id) ||
            Array.from(activeNodeIds).some((id) => id.startsWith(n.id + "."));

          let cls = (n.className || "").replace(/ dimmed| highlighted/g, "");
          if (isActive) cls += " highlighted";
          else cls += " dimmed";

          return { ...n, className: cls };
        })
      );
    }
  }, [highlightedEdge, setEdges, setNodes]);

  const handleHighlightEdge = useCallback((from: string, to: string) => {
    setHighlightedEdge({ from, to });
  }, []);

  const handleClearHighlight = useCallback(() => {
    setHighlightedEdge(null);
  }, []);

  // Handle node click - select
  const onNodeClick = useCallback(
    (_event: React.MouseEvent, node: Node<C4NodeData>) => {
      selectNode(node.id);
    },
    [selectNode]
  );

  const toggleExpand = useViewStore((s) => s.toggleExpand);

  const onNodeDoubleClick = useCallback(
    (_event: React.MouseEvent, node: Node<C4NodeData>) => {
      const nodeType = node.data.type;
      const hasChildren = (node.data.childCount ?? 0) > 0;
      console.log(
        "Double-clicked node:",
        node.id,
        "type:",
        nodeType,
        "hasChildren:",
        hasChildren,
        "currentLevel:",
        currentLevel
      );

      // Only expand nodes that have children
      if (!hasChildren) {
        console.log("Node has no children, skipping expansion");
        return;
      }

      // Mindmap-style expansion
      if (currentLevel === "L1") {
        if (nodeType === "system" || nodeType === "container") {
          toggleExpand(node.id);
        }
      } else if (nodeType === "container" && currentLevel === "L2") {
        toggleExpand(node.id);
      }
    },
    [currentLevel, toggleExpand, expandedNodes]
  );

  if (!data) {
    return (
      <div className="empty-canvas">
        <p>No architecture loaded</p>
        <p className="hint">Load a JSON file or paste JSON content to visualize</p>
      </div>
    );
  }

  return (
    <div className="architecture-canvas" data-c4-mode={c4Mode}>
      {/* SVG Markers Definition */}
      <svg
        style={{
          position: "absolute",
          top: 0,
          left: 0,
          width: 0,
          height: 0,
          pointerEvents: "none",
        }}
      >
        <defs>
          {/* Arrow end markers (pointing forward) */}
          <marker
            id="arrow"
            viewBox="0 0 10 10"
            refX="8"
            refY="5"
            markerWidth="6"
            markerHeight="6"
            orient="auto-start-reverse"
          >
            <path d="M 0 0 L 10 5 L 0 10 z" fill="#b1b1b7" />
          </marker>
          <marker
            id="arrow-active"
            viewBox="0 0 10 10"
            refX="8"
            refY="5"
            markerWidth="6"
            markerHeight="6"
            orient="auto-start-reverse"
          >
            <path d="M 0 0 L 10 5 L 0 10 z" fill="#10b981" />
          </marker>
          <marker
            id="arrow-dimmed"
            viewBox="0 0 10 10"
            refX="8"
            refY="5"
            markerWidth="6"
            markerHeight="6"
            orient="auto-start-reverse"
          >
            <path d="M 0 0 L 10 5 L 0 10 z" fill="#e5e7eb" />
          </marker>
          {/* Arrow start markers (pointing backward for bidirectional) */}
          <marker
            id="arrow-start"
            viewBox="0 0 10 10"
            refX="2"
            refY="5"
            markerWidth="6"
            markerHeight="6"
            orient="auto"
          >
            <path d="M 10 0 L 0 5 L 10 10 z" fill="#b1b1b7" />
          </marker>
          <marker
            id="arrow-active-start"
            viewBox="0 0 10 10"
            refX="2"
            refY="5"
            markerWidth="6"
            markerHeight="6"
            orient="auto"
          >
            <path d="M 10 0 L 0 5 L 10 10 z" fill="#10b981" />
          </marker>
          <marker
            id="arrow-dimmed-start"
            viewBox="0 0 10 10"
            refX="2"
            refY="5"
            markerWidth="6"
            markerHeight="6"
            orient="auto"
          >
            <path d="M 10 0 L 0 5 L 10 10 z" fill="#e5e7eb" />
          </marker>
        </defs>
      </svg>

      {/* Level indicator */}
      <div className="level-indicator">
        {currentLevel}{" "}
        {currentLevel === "L1"
          ? "• System Context"
          : currentLevel === "L2"
            ? "• Containers"
            : currentLevel === "L3"
              ? "• Components"
              : ""}
      </div>

      {/* Layout engine selector */}
      <div className="layout-selector">
        <label>Layout:</label>
        <select
          value={layoutEngine}
          onChange={(e) => setLayoutEngine(e.target.value as "sruja" | "c4level")}
        >
          <option value="c4level">C4 Level</option>
          <option value="sruja">Sruja Compound</option>
        </select>
        <label style={{ marginLeft: 12 }}>Style:</label>
        <select value={c4Mode} onChange={(e) => setC4Mode(e.target.value as "rich" | "pure")}>
          <option value="pure">Pure</option>
          <option value="rich">Rich</option>
        </select>
      </div>

      {/* C4 Legend (optional - toggle button in bottom-left) */}
      <Legend />

      {/* Loading spinner overlay */}
      {!layoutReady && nodes.length > 0 && (
        <div className="loading-overlay">
          <div className="loading-spinner" />
        </div>
      )}

      {/* Fade transition wrapper */}
      <div className={`canvas-fade ${!layoutReady ? "loading" : ""}`}>
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onNodeClick={onNodeClick}
          onNodeDoubleClick={onNodeDoubleClick}
          nodeTypes={finalNodeTypes}
          edgeTypes={edgeTypes}
          nodesDraggable={dragEnabled}
          nodesConnectable={false}
          elementsSelectable={true}
          fitView={layoutReady && !lastLayoutWasIncrementalRef.current}
          fitViewOptions={{ padding: 0.2 }}
          minZoom={0.1}
          maxZoom={2}
        >
          <Controls />
          <MiniMap
            nodeColor={(node) => {
              const type = (node.data as C4NodeData)?.type;
              switch (type) {
                case "person":
                  return "var(--c4-person)";
                case "system":
                  return "var(--c4-system)";
                case "container":
                  return "var(--c4-container)";
                case "component":
                  return "var(--c4-component)";
                default:
                  return "#999";
              }
            }}
          />
          <Background gap={16} size={1} color="var(--border-color)" />

          {/* Heatmap Overlay */}
          <HeatmapCanvasOverlay nodeBadness={metrics?.nodeBadness || {}} visible={showHeatmap} />
        </ReactFlow>
      </div>

      <FlowController
        onHighlightEdge={handleHighlightEdge}
        onClearHighlight={handleClearHighlight}
      />

      {isEditMode() && (
        <div className={`canvas-fab ${fabOpen ? "open" : ""}`}>
          {!fabOpen && (
            <button className="fab-main" title="Create" onClick={() => setFabOpen(true)}>
              +
            </button>
          )}
          {fabOpen && (
            <div className="fab-menu">
              <div className="fab-actions">
                <button
                  className="fab-btn"
                  onClick={() => {
                    setCreateType("system");
                    setCreateName("");
                    setCreateTech("");
                    setCreateExternal(false);
                  }}
                  title="Add System"
                >
                  System
                </button>
                <button
                  className="fab-btn"
                  onClick={() => {
                    setCreateType("person");
                    setCreateName("");
                  }}
                  title="Add Person"
                >
                  Person
                </button>
                <button
                  className="fab-btn"
                  disabled={!focusedSystemId}
                  onClick={() => {
                    setCreateType("container");
                    setCreateName("");
                    setCreateTech("");
                    setCreateExternal(false);
                  }}
                  title="Add Container"
                >
                  Container
                </button>
                <button
                  className="fab-btn"
                  disabled={!focusedContainerId}
                  onClick={() => {
                    setCreateType("component");
                    setCreateName("");
                  }}
                  title="Add Component"
                >
                  Component
                </button>
                <button
                  className="fab-close"
                  onClick={() => {
                    setFabOpen(false);
                    setCreateType(null);
                  }}
                >
                  Close
                </button>
              </div>
              {createType && (
                <div className="fab-form">
                  <input
                    className="fab-input"
                    placeholder="Name"
                    value={createName}
                    onChange={(e) => setCreateName(e.target.value)}
                  />
                  {createType === "container" && (
                    <>
                      <input
                        className="fab-input"
                        placeholder="Technology (optional)"
                        value={createTech}
                        onChange={(e) => setCreateTech(e.target.value)}
                      />
                      <label className="fab-check">
                        <input
                          type="checkbox"
                          checked={createExternal}
                          onChange={(e) => setCreateExternal(e.target.checked)}
                        />{" "}
                        External
                      </label>
                    </>
                  )}
                  <div className="fab-form-actions">
                    <button
                      className="fab-submit"
                      onClick={async () => {
                        if (!createType || !createName.trim()) return;
                        const slug =
                          createName
                            .toLowerCase()
                            .trim()
                            .replace(/[^a-z0-9_-]+/g, "-") || "item";
                        await updateArchitecture((arch) => {
                          const a = arch.architecture || {};
                          if (createType === "system") {
                            const systems = [...(a.systems || [])];
                            systems.push({ id: slug, label: createName, containers: [] });
                            return { ...arch, architecture: { ...a, systems } };
                          }
                          if (createType === "person") {
                            const persons = [...(a.persons || [])];
                            persons.push({ id: slug, label: createName });
                            return { ...arch, architecture: { ...a, persons } };
                          }
                          if (createType === "container" && focusedSystemId) {
                            const systems = (a.systems || []).map((s) => {
                              if (s.id !== focusedSystemId) return s;
                              const containers = [...(s.containers || [])];
                              const tags = createExternal ? ["external"] : undefined;
                              containers.push({
                                id: slug,
                                label: createName,
                                technology: createTech || undefined,
                                tags,
                                components: [],
                              });
                              return { ...s, containers };
                            });
                            return { ...arch, architecture: { ...a, systems } };
                          }
                          if (createType === "component" && focusedContainerId) {
                            const systems = (a.systems || []).map((s) => {
                              const containers = (s.containers || []).map((c) => {
                                if (c.id !== focusedContainerId) return c;
                                const components = [...(c.components || [])];
                                components.push({ id: slug, label: createName });
                                return { ...c, components };
                              });
                              return { ...s, containers };
                            });
                            return { ...arch, architecture: { ...a, systems } };
                          }
                          return arch;
                        });
                        setCreateName("");
                        setCreateTech("");
                        setCreateExternal(false);
                        setCreateType(null);
                        setFabOpen(false);
                      }}
                    >
                      Create
                    </button>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
