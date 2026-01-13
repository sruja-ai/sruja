// apps/website/src/features/playground/components/SimpleDiagramViewer.tsx
// Simplified diagram viewer for website homepage - uses same DOT -> React Flow pipeline as Designer
import { useEffect, useState, useCallback } from "react";
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  type Node,
  type Edge,
  MarkerType,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { convertDslToDot, type SrujaModelDump } from "@sruja/shared";
import { SrujaLoader } from "@sruja/ui";
import { Graphviz } from "@hpcc-js/wasm-graphviz";

// Simple Graphviz result type for this component
interface GraphvizResult {
  nodes: Record<
    string,
    {
      x: number;
      y: number;
      width: number;
      height: number;
    }
  >;
  edges: Array<{
    from: string;
    to: string;
    points?: Array<[number, number]>;
    label?: string;
  }>;
}

// Simple Graphviz JSON data structure
interface GraphvizJsonData {
  bb?: string;
  objects?: Array<{
    _gvid?: number;
    name?: string;
    pos?: string;
    width?: string | number; // Graphviz returns as string in inches
    height?: string | number; // Graphviz returns as string in inches
  }>;
  edges?: Array<{
    _gvid?: number;
    tail?: number;
    head?: number;
    _draw_?: Array<{
      op?: string;
      points?: string;
    }>;
    lp?: string;
  }>;
}

// Helper to parse edge points from Graphviz draw operations
function parseEdgePoints(
  drawOps?: Array<{ op?: string; points?: string }>
): Array<[number, number]> | undefined {
  if (!drawOps || drawOps.length === 0) return undefined;

  const drawOp = drawOps[0];
  if (!drawOp.points) return undefined;

  // Points format: "x1,y1 x2,y2 ..."
  return drawOp.points
    .trim()
    .split(/\s+/)
    .map((pt) => {
      const [x, y] = pt.split(",").map(Number);
      return [x * 72, y * 72] as [number, number]; // Convert inches to points
    });
}

// Helper to extract nodes from Graphviz JSON
function extractNodes(
  objects: GraphvizJsonData["objects"],
  gvidToNodeName: Map<number, string>
): GraphvizResult["nodes"] {
  const nodes: GraphvizResult["nodes"] = {};

  if (!objects) return nodes;

  for (const obj of objects) {
    if (obj._gvid !== undefined && obj.name) {
      const nodeName = obj.name.replace(/"/g, "");
      gvidToNodeName.set(obj._gvid, nodeName);

      // Parse position (format: "x,y" in inches)
      const pos = obj.pos ? obj.pos.split(",").map(Number) : [0, 0];
      // Parse width/height (may be string or number, in inches)
      const widthInches = typeof obj.width === "string" ? parseFloat(obj.width) : obj.width || 1;
      const heightInches =
        typeof obj.height === "string" ? parseFloat(obj.height) : obj.height || 0.5;
      const width = widthInches * 72; // Convert inches to points
      const height = heightInches * 72;

      nodes[nodeName] = {
        x: pos[0] * 72, // Convert inches to points
        y: pos[1] * 72,
        width,
        height,
      };
    }
  }

  return nodes;
}

// Helper to extract edges from Graphviz JSON
function extractEdges(
  edges: GraphvizJsonData["edges"],
  gvidToNodeName: Map<number, string>
): GraphvizResult["edges"] {
  const result: GraphvizResult["edges"] = [];

  if (!edges) return result;

  for (const edge of edges) {
    if (edge.tail !== undefined && edge.head !== undefined) {
      const from = gvidToNodeName.get(edge.tail);
      const to = gvidToNodeName.get(edge.head);

      if (from && to) {
        const points = parseEdgePoints(edge._draw_);
        result.push({
          from,
          to,
          points,
          label: edge.lp ? edge.lp.replace(/"/g, "") : undefined,
        });
      }
    }
  }

  return result;
}

// Simple function to run Graphviz and parse result
async function runGraphviz(dot: string): Promise<GraphvizResult> {
  const graphviz = await Graphviz.load();
  const jsonString = graphviz.layout(dot, "json", "dot");
  const data = JSON.parse(jsonString) as GraphvizJsonData;

  // Build gvid to node name mapping
  const gvidToNodeName = new Map<number, string>();

  // Extract nodes and edges
  const nodes = extractNodes(data.objects, gvidToNodeName);
  const edges = extractEdges(data.edges, gvidToNodeName);

  return { nodes, edges };
}

interface SimpleDiagramViewerProps {
  model: SrujaModelDump;
  dsl: string;
}

// Simple node component for the viewer
function SimpleNode({ data }: { data: { label: string; kind?: string } }) {
  return (
    <div
      style={{
        padding: "12px 16px",
        background: "var(--color-background, #fff)",
        border: "2px solid var(--color-border, #e2e8f0)",
        borderRadius: "8px",
        minWidth: "120px",
        textAlign: "center",
        boxShadow: "0 2px 4px rgba(0,0,0,0.1)",
      }}
    >
      <div style={{ fontWeight: 600, fontSize: "14px", marginBottom: "4px" }}>{data.label}</div>
      {data.kind && (
        <div style={{ fontSize: "11px", color: "var(--color-text-secondary, #64748b)" }}>
          {data.kind}
        </div>
      )}
    </div>
  );
}

const nodeTypes = {
  default: SimpleNode,
};

export function SimpleDiagramViewer({ model, dsl }: SimpleDiagramViewerProps) {
  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const renderDiagram = useCallback(async () => {
    if (!model || !dsl) return;

    setLoading(true);
    setError(null);

    try {
      // Step 1: Convert DSL to DOT
      const dotResult = await convertDslToDot(dsl, 1); // Level 1 = Context view
      if (!dotResult || !dotResult.dot) {
        throw new Error("Failed to generate DOT from DSL");
      }

      // Step 2: Run Graphviz layout
      const layoutResult = await runGraphviz(dotResult.dot);

      // Step 3: Convert Graphviz result to React Flow nodes and edges
      const flowNodes: Node[] = [];
      const flowEdges: Edge[] = [];

      // Convert nodes
      if (layoutResult.nodes) {
        for (const [nodeId, nodeData] of Object.entries(layoutResult.nodes)) {
          const element = model.elements?.[nodeId];
          flowNodes.push({
            id: nodeId,
            type: "default",
            position: {
              x: nodeData.x || 0,
              y: nodeData.y || 0,
            },
            data: {
              label: element?.title || nodeId,
              kind: element?.kind,
            },
            style: {
              width: nodeData.width || 72,
              height: nodeData.height || 36,
            },
          });
        }
      }

      // Convert edges
      if (layoutResult.edges) {
        for (const edge of layoutResult.edges) {
          const points = edge.points || [];
          if (points.length >= 2) {
            flowEdges.push({
              id: `${edge.from}-${edge.to}`,
              source: edge.from,
              target: edge.to,
              type: "smoothstep",
              markerEnd: {
                type: MarkerType.ArrowClosed,
              },
              label: edge.label || "",
              style: {
                stroke: "var(--color-border, #94a3b8)",
                strokeWidth: 2,
              },
            });
          }
        }
      }

      setNodes(flowNodes);
      setEdges(flowEdges);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Failed to render diagram";
      setError(errorMessage);
      console.error("Diagram rendering error:", err);
    } finally {
      setLoading(false);
    }
  }, [model, dsl, setNodes, setEdges]);

  useEffect(() => {
    renderDiagram();
  }, [renderDiagram]);

  if (loading) {
    return (
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          height: "100%",
          background: "var(--color-background-secondary, #f8fafc)",
        }}
      >
        <SrujaLoader size={32} />
      </div>
    );
  }

  if (error) {
    return (
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          height: "100%",
          color: "var(--color-error, #dc2626)",
          padding: "16px",
          textAlign: "center",
        }}
      >
        <div>
          <div style={{ fontWeight: 600, marginBottom: "8px" }}>Failed to render diagram</div>
          <div style={{ fontSize: "13px" }}>{error}</div>
        </div>
      </div>
    );
  }

  return (
    <div
      style={{
        width: "100%",
        height: "100%",
        background: "var(--color-background-secondary, #f8fafc)",
      }}
    >
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        nodeTypes={nodeTypes}
        fitView
        minZoom={0.1}
        maxZoom={2}
        defaultViewport={{ x: 0, y: 0, zoom: 0.8 }}
      >
        <Background color="#e2e8f0" gap={16} />
        <Controls />
        <MiniMap nodeColor={() => "var(--color-primary, #6366f1)"} maskColor="rgba(0, 0, 0, 0.1)" />
      </ReactFlow>
    </div>
  );
}
