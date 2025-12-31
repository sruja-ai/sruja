import { Graphviz } from "@hpcc-js/wasm-graphviz";
import type { GraphvizResult } from "./types";
import { measureQuality, type LayoutQuality } from "./qualityMetrics";

import { logger } from "@sruja/shared";

let graphvizInstance: Graphviz | null = null;

// Graphviz uses points (1/72 inch). Input width/height was in pixels but divided by 72 for DOT.
// Output positions are in points.
// We treat 1 point = 1 pixel for React Flow? Or scale up?
// React Flow defaults are usually pixels. If we defined node width=200 in projection,
// and divided by 72 for DOT (width=2.77), output bb will be in points.
// 2.77 inches * 72 = 200 points.
// So 1 unit in Graphviz output = 1 unit in React Flow (approx).
const SCALE = 1;

// Interface for Graphviz edge draw operations
interface GraphvizDrawOp {
  op: string;
  points?: [number, number][];
}

interface GraphvizEdge {
  _gvid: number;
  tail: number;
  head: number;
  _draw_?: GraphvizDrawOp[];
  _ldraw_?: GraphvizDrawOp[];
  pos?: string;
  lp?: string; // Label position "x,y"
}

export async function runGraphviz(dot: string): Promise<GraphvizResult> {
  if (!graphvizInstance) {
    graphvizInstance = await Graphviz.load();
  }

  // Request JSON output
  const jsonString = graphvizInstance.layout(dot, "json", "dot");
  const data = JSON.parse(jsonString);

  // Parse bounding box "llx,lly,urx,ury" or assumption from objects
  // The root object has 'bb' property: "0,0,width,height"
  let canvasHeight = 0;
  let canvasWidth = 0;

  if (data.bb) {
    const parts = data.bb.split(",").map(Number);
    // [llx, lly, urx, ury]
    canvasWidth = parts[2];
    canvasHeight = parts[3];
  }

  const nodes: GraphvizResult["nodes"] = [];
  const edges: GraphvizResult["edges"] = [];

  // Build a map of _gvid to node name for edge resolution
  const gvidToNodeName: Map<number, string> = new Map();

  // Helper to parse "x,y" string
  const parsePos = (posStr: string | undefined): { x: number; y: number } | null => {
    if (!posStr) return null;
    const [x, y] = posStr.split(",").map(Number);
    return { x, y };
  };

  // Helper to flatten objects (nodes and clusters)
  // Graphviz JSON has 'objects' array which may contain subgraphs
  const traverseObjects = (objs: any[]) => {
    objs.forEach((obj) => {
      // Build gvid to name mapping for all nodes
      if (obj._gvid !== undefined && obj.name) {
        const cleanName = obj.name.replace(/"/g, "");
        // Map ALL objects with _gvid, even clusters/subgraphs,
        // because edges might incorrectly reference them (though usually point to nodes)
        // or in case of compound edges.
        gvidToNodeName.set(obj._gvid, cleanName);
      }

      // Check if it's a node (has name, pos, width, height)
      if (obj.pos && obj.name && !obj.name.startsWith("cluster_")) {
        const center = parsePos(obj.pos);
        if (center) {
          // Graphviz 'pos' is CENTER of the node (in points)
          // ... (rest of processing)

          // Parse width/height from inches to pixels
          const w = parseFloat(obj.width) * 72;
          const h = parseFloat(obj.height) * 72;

          // Convert Graphviz center coordinates to React Flow top-left
          const gvX = center.x;
          const gvY = center.y;
          const rfYCenter = canvasHeight - gvY;

          const x = gvX * SCALE - w / 2;
          const y = rfYCenter * SCALE - h / 2;

          nodes.push({
            id: obj.name.replace(/"/g, ""),
            x,
            y,
            width: w,
            height: h,
          });
        }
      }

      // Recurse for subgraphs - Critical for finding nested nodes
      if (obj.subgraphs) {
        traverseObjects(obj.subgraphs);
      }
      if (obj.objects) {
        traverseObjects(obj.objects);
      }
      // Some Graphviz JSON versions might use '_subgraph_cnt' or implicit structure
    });
  };

  if (data.objects) {
    traverseObjects(data.objects);
  }

  // Parse Edges with spline points
  if (data.edges) {
    (data.edges as GraphvizEdge[]).forEach((gvEdge) => {
      const sourceId = gvidToNodeName.get(gvEdge.tail);
      const targetId = gvidToNodeName.get(gvEdge.head);

      if (!sourceId || !targetId) {
        logger.warn("Edge has unknown source/target gvid", {
          component: "layoutEngine",
          action: "parseEdges",
          tailGvid: gvEdge.tail,
          headGvid: gvEdge.head,
        });
        return;
      }

      // Parse bezier spline points from _draw_ array
      let points: [number, number][] = [];

      if (gvEdge._draw_) {
        // Find bezier operations (op === 'b' or 'B')
        const bezierOps = gvEdge._draw_.filter((op) => op.op.toLowerCase() === "b");
        if (bezierOps.length > 0 && bezierOps[0].points) {
          // Convert points from Graphviz coordinates to React Flow coordinates
          points = bezierOps[0].points.map(([x, y]) => {
            // Invert Y axis
            return [x * SCALE, (canvasHeight - y) * SCALE] as [number, number];
          });
        }
      }

      // Parse label position
      let labelPos: { x: number; y: number } | undefined;
      if (gvEdge.lp) {
        const pos = parsePos(gvEdge.lp);
        if (pos) {
          // Invert Y axis for React Flow
          labelPos = {
            x: pos.x * SCALE,
            y: (canvasHeight - pos.y) * SCALE,
          };
        }
      } else if (gvEdge._ldraw_) {
        // Fallback: try to find text operation in _ldraw_
        const textOps = gvEdge._ldraw_.filter((op) => op.op === "T");
        if (textOps.length > 0 && textOps[0].points && textOps[0].points.length > 0) {
          const [x, y] = textOps[0].points[0];
          labelPos = {
            x: x * SCALE,
            y: (canvasHeight - y) * SCALE,
          };
        }
      }

      edges.push({
        id: `${sourceId}-${targetId}`,
        source: sourceId,
        target: targetId,
        points: points.length > 0 ? points : undefined,
        labelPos: labelPos,
      });
    });
  }

  // Parsed nodes and edges from Graphviz

  return {
    nodes,
    edges,
    width: canvasWidth,
    height: canvasHeight,
  };
}

/**
 * Layout result with quality metrics
 */
export interface LayoutWithQuality {
  layoutResult: GraphvizResult;
  quality: LayoutQuality | null;
  iteration: number;
}

/**
 * Perform layout and measure quality metrics
 *
 * This function:
 * - Generates diagram layout using Graphviz (one pass, no iteration)
 * - Measures quality metrics (dev-only, for reporting to E2E tests)
 * - Does NOT iteratively refine the layout
 *
 * For iterative quality improvement, see the development workflow:
 * - Run E2E tests to get quality scores
 * - Tune Graphviz configuration/layout engine based on scores
 * - Test again (manual iteration during development)
 *
 * Quality metrics are only calculated in development for developer tooling.
 * In production, quality measurement is skipped to avoid performance overhead.
 */
export async function layoutAndMeasureQuality(
  initialDot: string,
  _relations?: any[]
): Promise<LayoutWithQuality> {
  // Use standard Graphviz layout directly
  const layoutResult = await runGraphviz(initialDot);

  // Only measure quality in development (developer tool, not user-facing)
  // Skip in production to avoid performance overhead
  const isDev = import.meta.env.DEV || import.meta.env.MODE === "development";
  const quality = isDev ? measureQuality(layoutResult) : null;

  return {
    layoutResult,
    quality,
    iteration: 1,
  };
}
