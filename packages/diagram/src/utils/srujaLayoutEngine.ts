// packages/react-flow-architecture/src/utils/srujaLayoutEngine.ts
// Sruja Layout Engine integration for React Flow
// This file now delegates to the bridge adapter for feature flag control
import type { Node, Edge } from "@xyflow/react";
import type { C4NodeData, ArchitectureJSON } from "../types";
import { applySrujaLayout as bridgeApplySrujaLayout } from "@sruja/layout/bridge";
// import { applySrujaLayout as bridgeApplySrujaLayout } from "../../../layout/src/bridge";
import type { C4Level } from "../types";

interface LayoutOptions {
  level: C4Level;
  focusedSystemId?: string;
  focusedContainerId?: string;
  direction?: "TB" | "LR" | "RL";
  expandedNodes?: Set<string>;
  viewportSize?: { width: number; height: number };
}

/**
 * Apply Sruja layout to nodes and edges
 * Returns nodes with updated positions and edges
 * 
 * This function now delegates to the bridge adapter which handles:
 * - Feature flag control (USE_NEW_LAYOUT_ENGINE)
 * - Automatic fallback to legacy engine
 * - Engine selection based on environment variable
 */
export async function applySrujaLayout(
  nodes: Node<C4NodeData>[],
  edges: Edge[],
  architectureData: ArchitectureJSON,
  options: LayoutOptions = { level: "L1" }
): Promise<{ nodes: Node<C4NodeData>[]; edges: Edge[] }> {
  // Delegate to bridge adapter which handles engine selection
  return bridgeApplySrujaLayout(nodes, edges, architectureData, options);
}
