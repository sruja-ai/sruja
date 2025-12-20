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

        // Support expanded nodes via URL parameter (comma-separated list)
        const expandedParam = params.get("expanded");
        const expandedNodes = expandedParam
          ? new Set(expandedParam.split(",").map((id) => id.trim()))
          : new Set<string>();

        // Initial conversion with expanded nodes
        const initial = jsonToReactFlow(arch, { level, expandedNodes });

        // Apply Layout with expanded nodes
        const { nodes: n, edges: e } = await applySrujaLayout(initial.nodes, initial.edges, arch, {
          level,
          direction: "TB",
          expandedNodes, // Pass expanded nodes to layout engine
        });

        setNodes(n);
        setEdges(e);

        // Expose for testing (update when nodes change)
        (window as any).__CYBER_GRAPH__ = { nodes: n, edges: e, expandedNodes };
      } catch (err: any) {
        console.error(err);
        setError(err.message);
      } finally {
        setLoading(false);
      }
    }

    load();

    (window as any).loadGraph = async (
      jsonOrDsl: string | ArchitectureJSON,
      level: string = "L1",
      expandedNodesList: string[] = [] // Add optional expanded nodes
    ) => {
      try {
        let arch: ArchitectureJSON;

        if (typeof jsonOrDsl === "string") {
          // Try to parse as JSON first
          try {
            arch = JSON.parse(jsonOrDsl) as ArchitectureJSON;
          } catch {
            // Assume it's DSL, convert via WASM
            const json = await convertDslToJson(jsonOrDsl);
            if (!json) throw new Error("Failed to parse DSL");
            arch = json as ArchitectureJSON;
          }
        } else {
          arch = jsonOrDsl;
        }

        const expandedNodes = new Set<string>(expandedNodesList);
        const initial = jsonToReactFlow(arch, { level: level as any, expandedNodes });
        // Apply layout
        const { nodes: n, edges: e } = await applySrujaLayout(
          initial.nodes,
          initial.edges,
          arch,
          {
            level: level as any,
            direction: "TB", // Keep direction as it was in the original call
            expandedNodes: expandedNodes, // Use the already defined expandedNodes
          }
        );
        setNodes(n);
        setEdges(e);
        (window as any).__CYBER_GRAPH__ = { nodes: n, edges: e, expandedNodes };

        return { success: true, nodeCount: n.length, edgeCount: e.length };
      } catch (err: any) {
        console.error("[loadGraph] Error:", err);
        return { success: false, error: err.message };
      }
    };
  }, [window.location.search]); // Re-run when URL params change (including expanded)

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
