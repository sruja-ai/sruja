import { Graphviz } from "@hpcc-js/wasm-graphviz";
import type { GraphvizResult, GraphvizCluster } from "./types";
import {
  measureQuality,
  type LayoutQuality,
  type ParentChildRelationships,
} from "./qualityMetrics";

import { logger } from "@sruja/shared";
import { AppError, ErrorType } from "../../utils/errorHandling";

let graphvizInstance: Graphviz | null = null;

/**
 * Coordinate System and DPI Configuration
 *
 * Graphviz uses points as its coordinate unit (1 point = 1/72 inch).
 * React Flow uses pixels for its coordinate system.
 *
 * Default Graphviz DPI: 72 (1 inch = 72 points)
 * Standard Screen DPI: 96 (1 inch = 96 pixels)
 *
 * However, in our implementation:
 * - Node sizes are defined in pixels and divided by 72 for DOT input
 * - Graphviz outputs coordinates in points
 * - We treat 1 point = 1 pixel (SCALE = 1) because:
 *   - If node width=200px, we pass width=200/72≈2.77in to Graphviz
 *   - Graphviz outputs width in points: 2.77in * 72 = 200 points
 *   - So 1 point in Graphviz output ≈ 1 pixel in React Flow
 *
 * This approximation works well for screen display at standard viewing distances.
 * If precise DPI conversion is needed, use: SCALE = TARGET_DPI / GRAPHVIZ_DPI
 */
export const GRAPHVIZ_DEFAULT_DPI = 72; // Graphviz's default DPI (points per inch)
export const TARGET_DPI = 96; // Standard screen DPI (pixels per inch)
// Current implementation uses 1:1 scale (1 point = 1 pixel)
// This works because node sizes are pre-scaled when passed to Graphviz
export const SCALE = 1; // SCALE = TARGET_DPI / GRAPHVIZ_DEFAULT_DPI would be 1.33, but 1:1 works for our use case

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

/**
 * Graphviz JSON object structure (nodes, clusters, subgraphs)
 * This represents objects in the Graphviz JSON output
 */
interface GraphvizObject {
  _gvid?: number;
  name?: string;
  pos?: string; // Position "x,y" (for nodes)
  width?: string; // Width in inches (for nodes)
  height?: string; // Height in inches (for nodes)
  bb?: string; // Bounding box "llx,lly,urx,ury" (for clusters)
  objects?: GraphvizObject[];
  subgraphs?: GraphvizObject[];
}

/**
 * Graphviz JSON root structure
 */
interface GraphvizJsonData {
  bb?: string; // Root bounding box "llx,lly,urx,ury"
  objects?: GraphvizObject[];
  edges?: GraphvizEdge[];
}

/**
 * Error class for Graphviz layout failures
 */
export class GraphvizLayoutError extends AppError {
  constructor(message: string, context?: Record<string, unknown>) {
    super(message, ErrorType.UNKNOWN, context);
    this.name = "GraphvizLayoutError";
  }
}

/**
 * Future Enhancement: Web Worker Offloading
 *
 * For large/complex diagrams, Graphviz layout computation can block the main thread.
 * Consider offloading to a Web Worker for better UX:
 *
 * 1. Create: apps/designer/src/workers/layoutWorker.ts
 * 2. Move Graphviz.load() and layout() calls to worker
 * 3. Post message with DOT string, receive layout result
 *
 * Considerations:
 * - @hpcc-js/wasm-graphviz WASM module needs to load in worker context
 * - Worker.postMessage() can transfer large objects efficiently
 * - Error handling via worker message events
 * - Worker pool for parallel layouts (advanced)
 *
 * Example structure:
 * ```typescript
 * // layoutWorker.ts
 * import { Graphviz } from "@hpcc-js/wasm-graphviz";
 * let graphviz: Graphviz | null = null;
 * self.onmessage = async (e) => {
 *   if (!graphviz) graphviz = await Graphviz.load();
 *   try {
 *     const json = graphviz.layout(e.data.dot, "json", "dot");
 *     self.postMessage({ result: JSON.parse(json) });
 *   } catch (err) {
 *     self.postMessage({ error: err.message });
 *   }
 * };
 * ```
 *
 * Note: Verify WASM compatibility in Workers before implementation.
 * Test with large diagrams to measure performance improvement.
 */
export async function runGraphviz(dot: string): Promise<GraphvizResult> {
  try {
    if (!graphvizInstance) {
      graphvizInstance = await Graphviz.load();
    }

    // Request JSON output
    let jsonString: string;
    try {
      jsonString = graphvizInstance.layout(dot, "json", "dot");
    } catch (layoutError) {
      const errorMessage = layoutError instanceof Error ? layoutError.message : String(layoutError);
      logger.error("Graphviz layout computation failed", {
        component: "layoutEngine",
        action: "runGraphviz",
        error: errorMessage,
        dotLength: dot.length,
      });
      throw new GraphvizLayoutError(
        "Failed to compute diagram layout. The diagram may be too complex or contain invalid elements.",
        {
          originalError: errorMessage,
          dotLength: dot.length,
        }
      );
    }

    let data: GraphvizJsonData;
    try {
      data = JSON.parse(jsonString) as GraphvizJsonData;
    } catch (parseError) {
      const errorMessage = parseError instanceof Error ? parseError.message : String(parseError);
      logger.error("Failed to parse Graphviz JSON output", {
        component: "layoutEngine",
        action: "runGraphviz",
        error: errorMessage,
        jsonLength: jsonString?.length || 0,
      });
      throw new GraphvizLayoutError(
        "Failed to process diagram layout. The layout result may be invalid.",
        {
          originalError: errorMessage,
        }
      );
    }

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

    // Extract cluster information for parent-child relationships
    const clusters: Map<string, GraphvizCluster> = new Map();

    // Track all node IDs that we've seen to ensure we don't miss any
    const seenNodeIds = new Set<string>();

    // Helper to flatten objects (nodes and clusters)
    // Graphviz JSON has 'objects' array which may contain subgraphs
    const traverseObjects = (objs: GraphvizObject[]) => {
      objs.forEach((obj) => {
        // Build gvid to name mapping for all nodes
        if (obj._gvid !== undefined && obj.name) {
          const cleanName = obj.name.replace(/"/g, "");
          // Map ALL objects with _gvid, even clusters/subgraphs,
          // because edges might incorrectly reference them (though usually point to nodes)
          // or in case of compound edges.
          gvidToNodeName.set(obj._gvid, cleanName);
        }

        // Extract cluster information (clusters have names starting with "cluster_")
        if (obj.name && obj.name.startsWith("cluster_")) {
          const clusterName = obj.name.replace(/"/g, "");
          // Extract parent ID from cluster name (cluster_PARENTID -> PARENTID)
          const parentId = clusterName.replace(/^cluster_/, "");

          // Collect child node IDs from cluster's objects
          // This function recursively collects all node IDs within the cluster
          const childIds: string[] = [];
          const collectChildren = (objs: GraphvizObject[]) => {
            objs.forEach((childObj) => {
              // Collect node IDs (nodes have names that don't start with "cluster_")
              if (childObj.name && !childObj.name.startsWith("cluster_")) {
                const childId = childObj.name.replace(/"/g, "");
                if (!childIds.includes(childId)) {
                  childIds.push(childId);
                }
              }
              // Recurse into nested objects and subgraphs
              if (childObj.objects) collectChildren(childObj.objects);
              if (childObj.subgraphs) collectChildren(childObj.subgraphs);
            });
          };

          if (obj.objects) collectChildren(obj.objects);
          if (obj.subgraphs) collectChildren(obj.subgraphs);

          // Store cluster info (bb, children) for later use
          clusters.set(parentId, {
            bb: obj.bb, // Bounding box: "llx,lly,urx,ury"
            children: childIds,
          });

          // Log cluster info in dev mode for debugging
          if (typeof import.meta !== "undefined" && import.meta.env?.DEV) {
            logger.debug("Found cluster", {
              component: "layoutEngine",
              action: "traverseObjects",
              clusterName,
              parentId,
              bb: obj.bb,
              children: childIds,
              childCount: childIds.length,
            });
          }

          // Continue traversing cluster contents (for nested structures)
          // This ensures nodes inside clusters are processed and added to the nodes array
          if (obj.objects) traverseObjects(obj.objects);
          if (obj.subgraphs) traverseObjects(obj.subgraphs);
          return; // Don't process cluster as a node
        }

        // Check if it's a node (has name, pos, width, height)
        // Process ALL nodes, including those inside clusters
        if (obj.pos && obj.name && !obj.name.startsWith("cluster_") && obj.width && obj.height) {
          const nodeId = obj.name.replace(/"/g, "");
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

            // Only add node if we haven't seen it before (avoid duplicates)
            if (!seenNodeIds.has(nodeId)) {
              nodes.push({
                id: nodeId,
                x,
                y,
                width: w,
                height: h,
              });
              seenNodeIds.add(nodeId);
            }
          }
        }

        // Recurse for subgraphs - Critical for finding nested nodes
        // This ensures we process nodes that are nested inside subgraphs/clusters
        if (obj.subgraphs && obj.subgraphs.length > 0) {
          traverseObjects(obj.subgraphs);
        }
        if (obj.objects && obj.objects.length > 0) {
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

    // Log cluster information in dev mode
    if (typeof import.meta !== "undefined" && import.meta.env?.DEV && clusters.size > 0) {
      logger.debug("Parsed clusters", {
        component: "layoutEngine",
        action: "runGraphviz",
        clusterCount: clusters.size,
        clusters: Object.fromEntries(clusters),
      });
      // Verify that all child nodes referenced in clusters are in the nodes array
      const allChildIds = new Set<string>();
      clusters.forEach((cluster) => {
        cluster.children.forEach((childId) => allChildIds.add(childId));
      });
      const nodeIds = new Set(nodes.map((n) => n.id));
      const missingChildIds = Array.from(allChildIds).filter((id) => !nodeIds.has(id));
      if (missingChildIds.length > 0) {
        logger.warn("Child nodes referenced in clusters but not found in nodes array", {
          component: "layoutEngine",
          action: "runGraphviz",
          missingChildIds,
          missingCount: missingChildIds.length,
        });
      } else {
        logger.debug("All child nodes from clusters are present in nodes array", {
          component: "layoutEngine",
          action: "runGraphviz",
          childNodeCount: allChildIds.size,
        });
      }
    }

    return {
      nodes,
      edges,
      width: canvasWidth,
      height: canvasHeight,
      // Include cluster information for compound node building
      clusters: clusters.size > 0 ? Object.fromEntries(clusters) : undefined,
    };
  } catch (error) {
    // Re-throw GraphvizLayoutError as-is
    if (error instanceof GraphvizLayoutError) {
      throw error;
    }
    // Wrap unexpected errors
    const errorMessage = error instanceof Error ? error.message : String(error);
    logger.error("Unexpected error in runGraphviz", {
      component: "layoutEngine",
      action: "runGraphviz",
      error: errorMessage,
    });
    throw new GraphvizLayoutError("An unexpected error occurred during diagram layout.", {
      originalError: errorMessage,
    });
  }
}

/**
 * Layout result with quality metrics
 */
export interface LayoutWithQuality {
  layoutResult: GraphvizResult;
  quality: LayoutQuality | null;
  parentChildContainmentViolations: Array<{ childId: string; parentId: string }>;
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
  _relations?: unknown[], // Unused parameter, kept for API compatibility
  parentChildRelationships?: ParentChildRelationships
): Promise<LayoutWithQuality> {
  // Use standard Graphviz layout directly
  const layoutResult = await runGraphviz(initialDot);

  // Only measure quality in development (developer tool, not user-facing)
  // Skip in production to avoid performance overhead
  const isDev =
    typeof import.meta !== "undefined" &&
    (import.meta.env?.DEV || import.meta.env?.MODE === "development");
  const qualityResult = isDev ? measureQuality(layoutResult, parentChildRelationships) : null;

  return {
    layoutResult,
    quality: qualityResult?.quality ?? null,
    parentChildContainmentViolations: qualityResult?.parentChildContainmentViolations ?? [],
    iteration: 1,
  };
}
