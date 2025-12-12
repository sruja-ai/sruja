import React, { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { ReactFlow, Background, ReactFlowProvider, Node, Edge } from "@xyflow/react";
import {
  nodeTypes,
  edgeTypes,
  jsonToReactFlow,
  applySrujaLayout,
  Legend,
  type ArchitectureJSON,
  type C4NodeData,
} from "../src/index";
// @ts-ignore
import {
  initWasmAuto,
  convertDslToJson,
  loadExamplesManifest,
  loadExampleFile,
} from "@sruja/shared";

import "@xyflow/react/dist/style.css";
import "../src/styles/index.css";

function App() {
  const [nodes, setNodes] = useState<Node<C4NodeData>[]>([]);
  const [edges, setEdges] = useState<Edge[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    async function load() {
      try {
        setLoading(true);
        // Init WASM
        await initWasmAuto({ base: "/" });

        // Get params
        const params = new URLSearchParams(window.location.search);
        const exampleName = params.get("example");
        const fileName = params.get("file");

        let dsl = "";
        if (exampleName) {
          const manifest = await loadExamplesManifest();
          const ex = manifest.examples.find((e: any) => e.name === exampleName);
          if (!ex) throw new Error(`Example not found: ${exampleName}`);
          dsl = await loadExampleFile(ex.file);
        } else if (fileName) {
          dsl = await loadExampleFile(fileName);
        } else {
          setLoading(false);
          return;
        }

        const json = await convertDslToJson(dsl);
        if (!json) throw new Error("Failed to parse DSL");
        const arch = json as ArchitectureJSON;

        const level = (params.get("level") as any) || "L1";

        // Initial conversion
        const initial = jsonToReactFlow(arch, { level });

        // Apply Layout
        const { nodes: n, edges: e } = applySrujaLayout(initial.nodes, initial.edges, arch, {
          level,
          direction: "TB",
        });

        setNodes(n);
        setEdges(e);

        // Expose for testing
        (window as any).__CYBER_GRAPH__ = { nodes: n, edges: e };
      } catch (err: any) {
        console.error(err);
        setError(err.message);
      } finally {
        setLoading(false);
      }
    }

    load();
  }, []);

  if (loading) return <div className="loading-overlay">Loading...</div>;
  if (error) return <div>Error: {error}</div>;

  return (
    <div style={{ width: "100vw", height: "100vh" }} className="architecture-canvas">
      <ReactFlow nodes={nodes} edges={edges} nodeTypes={nodeTypes} edgeTypes={edgeTypes} fitView>
        <Background />
        <Legend />
      </ReactFlow>
    </div>
  );
}

const root = createRoot(document.getElementById("root")!);
root.render(<App />);
