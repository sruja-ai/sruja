import { useState, useEffect } from "react";
import {
  ReactFlow,
  useNodesState,
  useEdgesState,
  useReactFlow,
  ReactFlowProvider,
  Panel,
  type Node,
  type Edge,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { Graphviz } from "@hpcc-js/wasm-graphviz";
import type { DotResult, DotElement } from "@sruja/shared";

interface SrujaCanvasLiteProps {
  dotResult: DotResult | null;
}

interface GraphvizNode {
  id: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

interface GraphvizEdge {
  id: string;
  source: string;
  target: string;
}

interface GraphvizObject {
  pos?: string;
  name?: string;
  width?: string;
  height?: string;
  _gvid?: number;
  objects?: GraphvizObject[];
  subgraphs?: GraphvizObject[];
}

interface GraphvizEdgeData {
  tail: number;
  head: number;
}

interface LayoutResult {
  nodes: GraphvizNode[];
  edges: GraphvizEdge[];
  width: number;
  height: number;
}

let graphvizInstance: Graphviz | null = null;

async function runLayout(dot: string): Promise<LayoutResult> {
  if (!graphvizInstance) {
    graphvizInstance = await Graphviz.load();
  }

  const jsonString = graphvizInstance.layout(dot, "json", "dot");
  const data = JSON.parse(jsonString);

  const canvasWidth = data.bb ? parseFloat(data.bb.split(",")[2]) : 800;
  const canvasHeight = data.bb ? parseFloat(data.bb.split(",")[3]) : 600;

  const nodes: GraphvizNode[] = [];
  const edges: GraphvizEdge[] = [];

  const traverse = (objs: GraphvizObject[]) => {
    objs.forEach((obj) => {
      if (obj.pos && obj.name && !obj.name.startsWith("cluster_")) {
        const [gx, gy] = obj.pos.split(",").map(Number);
        const w = parseFloat(obj.width || "0") * 72;
        const h = parseFloat(obj.height || "0") * 72;

        // Convert to React Flow coordinates (invert Y)
        const x = gx - w / 2;
        const y = canvasHeight - gy - h / 2;

        nodes.push({
          id: obj.name.replace(/"/g, ""),
          x,
          y,
          width: w,
          height: h,
        });
      }
      if (obj.objects) traverse(obj.objects);
      if (obj.subgraphs) traverse(obj.subgraphs);
    });
  };

  if (data.objects) traverse(data.objects);

  if (data.edges) {
    data.edges.forEach((e: GraphvizEdgeData) => {
      // Find source/target by gvid
      const sourceObj = data.objects?.find((o: GraphvizObject) => o._gvid === e.tail);
      const targetObj = data.objects?.find((o: GraphvizObject) => o._gvid === e.head);
      if (sourceObj && targetObj) {
        edges.push({
          id: `${sourceObj.name}-${targetObj.name}`,
          source: sourceObj.name.replace(/"/g, ""),
          target: targetObj.name.replace(/"/g, ""),
        });
      }
    });
  }

  return { nodes, edges, width: canvasWidth, height: canvasHeight };
}

function SrujaCanvasLiteInternal({ dotResult }: SrujaCanvasLiteProps) {
  const { fitView } = useReactFlow();
  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!dotResult) return;

    let active = true;
    async function performLayout() {
      setLoading(true);
      try {
        const layout = await runLayout(dotResult!.dot);
        if (!active) return;

        const elementMap = new Map<string, DotElement>();
        dotResult!.elements.forEach((e) => elementMap.set(e.id, e));

        const rfNodes: Node[] = layout.nodes.map((n) => {
          const meta = elementMap.get(n.id);
          return {
            id: n.id,
            position: { x: n.x, y: n.y },
            data: {
              label: meta?.title || n.id,
              kind: meta?.kind || "container",
              technology: meta?.technology,
            },
            style: {
              width: n.width,
              height: n.height,
            },
            type: "srujaNode",
          };
        });

        const rfEdges: Edge[] = dotResult!.relations.map((r, idx) => ({
          id: `e-${idx}`,
          source: r.from,
          target: r.to,
          label: r.label,
          type: "smoothstep",
          style: { stroke: "var(--color-border)" },
        }));

        setNodes(rfNodes);
        setEdges(rfEdges);

        setTimeout(() => {
          fitView({ padding: 0.2, duration: 400 });
        }, 50);
      } catch (err) {
        console.error("Layout failed", err);
      } finally {
        if (active) setLoading(false);
      }
    }

    performLayout();
    return () => {
      active = false;
    };
  }, [dotResult, setNodes, setEdges, fitView]);

  return (
    <div style={{ width: "100%", height: "100%", position: "relative" }}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        nodeTypes={nodeTypes}
        fitView
        proOptions={{ hideAttribution: true }}
      >
        <Panel
          position="bottom-right"
          style={{ fontSize: "10px", opacity: 0.5, color: "var(--color-text-secondary)" }}
        >
          {loading ? "Computing layout..." : "Sruja Engine"}
        </Panel>
      </ReactFlow>
    </div>
  );
}

interface SrujaNodeData {
  kind: string;
  label: string;
  technology?: string;
}

const nodeTypes = {
  srujaNode: ({ data, style }: { data: SrujaNodeData; style?: React.CSSProperties }) => {
    const isPerson = data.kind === "person" || data.kind === "actor";
    const colors = isPerson
      ? { bg: "#08427B", border: "#052E56", text: "#FFFFFF" }
      : { bg: "#438DD5", border: "#2E6295", text: "#FFFFFF" };

    return (
      <div
        style={{
          ...style,
          background: colors.bg,
          border: `1px solid ${colors.border}`,
          borderRadius: isPerson ? "20px" : "4px",
          color: colors.text,
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          padding: "8px",
          textAlign: "center",
          boxShadow: "0 2px 4px rgba(0,0,0,0.1)",
          boxSizing: "border-box",
        }}
      >
        <div style={{ fontWeight: "bold", fontSize: "13px" }}>{data.label}</div>
        {data.technology && (
          <div style={{ fontSize: "10px", opacity: 0.8, marginTop: "2px" }}>
            [{data.technology}]
          </div>
        )}
        <div
          style={{ fontSize: "9px", opacity: 0.6, marginTop: "4px", textTransform: "uppercase" }}
        >
          {data.kind}
        </div>
      </div>
    );
  },
};

export function SrujaCanvasLite({ dotResult }: SrujaCanvasLiteProps) {
  return (
    <div style={{ height: "100%", width: "100%", background: "var(--color-background-secondary)" }}>
      <ReactFlowProvider>
        <SrujaCanvasLiteInternal dotResult={dotResult} />
      </ReactFlowProvider>
    </div>
  );
}
