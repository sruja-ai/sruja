/**
 * Bridge Adapter - Integrates modular layout engine with diagram package
 */

import type { Node, Edge } from "@xyflow/react";
import { createLayoutEngine, type LayoutOptions } from "./core/engine";
import type { LayoutResult, LayoutNode, LayoutEdge } from "./core/types";
// import { getLayoutWorker } from "./worker/client";


// Import types from diagram package
interface C4NodeData {
  id: string;
  type?: string;
  label?: string;
  description?: string;
  technology?: string;
  isExternal?: boolean;
  expanded?: boolean;
  childCount?: number;
  parent?: string;
  [key: string]: unknown; // Allow additional properties
}

export interface ArchitectureJSON {
  metadata?: {
    name?: string;
    version?: string;
  };
  architecture?: {
    persons?: any[];
    systems?: any[];
    relations?: any[];
  };
}

// Helper function to create simple graph structure
function createSimpleGraph(nodes: Node<C4NodeData>[], edges: Edge[]) {
  const nodeMap = new Map(nodes.map((n) => [n.id, n]));
  const edgeList = edges.map((e) => ({
    from: e.source,
    to: e.target,
    id: e.id || `${e.source}-${e.target}`,
    label: e.label,
  }));

  return { nodes: nodeMap, relationships: edgeList };
}

/**
 * Main bridge function - uses the consolidated modular layout engine
 */
export async function applySrujaLayout<T extends C4NodeData = C4NodeData>(
  nodes: Node<T>[],
  edges: Edge[],
  architectureData?: ArchitectureJSON,
  options?: any
): Promise<{ nodes: Node<T>[]; edges: Edge[] }> {
  const layoutOptions = options ?? { level: "L1" };

  console.log("🚀 Using consolidated modular layout engine");

  try {
    // Create simple graph structure
    const graph = architectureData
      ? createGraphFromArchitecture(architectureData)
      : createSimpleGraph(nodes, edges);

    // Convert to new modular options
    const modularOptions: Partial<LayoutOptions> = {
      strategy:
        layoutOptions.level === "L0"
          ? "L0-landscape"
          : layoutOptions.level === "L1"
            ? "L1-context"
            : layoutOptions.level === "L2"
              ? "L2-container"
              : layoutOptions.level === "L3"
                ? "L3-component"
                : "L1-context",
      quality: {
        targetGrade: "B",
        strictMode: false,
        validateConstraints: true,
        enforceMetrics: false,
        earlyExit: true,
      },
      debug: {
        enabled: typeof process !== "undefined" && (process.env?.LAYOUT_DEBUG === "true" || process.env?.NODE_ENV === "development"),
        saveIntermediates: false,
        showMetrics: true,
        showHeatmap: false,
        showPortUsage: false,
        verboseLogging: false,
      },
    };

    // Create and execute new layout engine
    // Worker DISABLED due to serialization crashes - always use main thread
    /*
    try {
      const client = getLayoutWorker();
      const result = await client.layout(graph, {
        level: layoutOptions.level,
        expandedNodes: new Set(),
        hiddenNodes: new Set(),
        gridSize: layoutOptions.gridSize || 10,
        snapToGrid: layoutOptions.snapToGrid || false,
      }, modularOptions);

      // Convert result back to React Flow format
      return convertResultToReactFlow(result, nodes, edges);
    } catch (workerError) {
      console.warn("Worker layout failed, falling back to main thread:", workerError);
    */

    // Fallback to main thread execution
    const engine = createLayoutEngine(modularOptions);
    const result = await engine.layout(graph, {
      level: layoutOptions.level,
      expandedNodes: new Set(),
      hiddenNodes: new Set(),
      gridSize: layoutOptions.gridSize || 10,
      snapToGrid: layoutOptions.snapToGrid || false,
    });

    return convertResultToReactFlow(result, nodes, edges);
    // } // Removed catch block brace

  } catch (error) {
    console.error("Layout engine failed:", error);

    // Fallback to simple layout
    console.warn("Using fallback layout");
    return applyFallbackLayout(nodes, edges);
  }
}

/**
 * Simple graph creation from architecture data
 */
function createGraphFromArchitecture(architectureData: ArchitectureJSON) {
  const nodes: any[] = [];
  const relationships: any[] = [];

  if (architectureData?.architecture) {
    const { persons = [], systems = [], relations = [] } = architectureData.architecture;

    // Add persons
    persons.forEach((person: any) => {
      nodes.push({
        id: `c4_${person.id}`,
        label: person.label || person.id,
        kind: "Person",
        level: "context",
        description: person.description,
      });
    });

    // Add systems and their children (containers, components)
    systems.forEach((system: any) => {
      const isExternal = system.metadata?.some(
        (m: any) => m.key === "external" && m.value === "true"
      );

      nodes.push({
        id: `c4_${system.id}`,
        label: system.label || system.id,
        kind: isExternal ? "ExternalSystem" : "SoftwareSystem",
        level: "context",
        description: system.description,
      });

      // Helper to add container-like items (Containers, DataStores, Queues)
      const addContainerLike = (item: any, type: string) => {
        // Ensure FQN
        const itemId = item.id.includes(".") ? item.id : `${system.id}.${item.id}`;

        nodes.push({
          id: `c4_${itemId}`,
          label: item.label || item.id,
          kind: "Container", // Map specific types to Container for layout purposes
          tag: type, // Store original type in tag or metadata if needed
          level: "container",
          description: item.description,
          parentId: `c4_${system.id}`,
        });

        // Add components if present (only for containers usually, but generic handling is safe)
        if (item.components) {
          item.components.forEach((component: any) => {
            const componentId = component.id.includes(".") ? component.id : `${itemId}.${component.id}`;
            nodes.push({
              id: `c4_${componentId}`,
              label: component.label || component.id,
              kind: "Component",
              level: "component",
              description: component.description,
              parentId: `c4_${itemId}`,
            });
          });
        }
      };

      if (system.containers) system.containers.forEach((c: any) => addContainerLike(c, "Container"));
      // Handle lowercase keys from parser
      if (system.datastores) system.datastores.forEach((d: any) => addContainerLike(d, "DataStore"));
      if (system.queues) system.queues.forEach((q: any) => addContainerLike(q, "Queue"));
      if (system.queues) system.queues.forEach((q: any) => addContainerLike(q, "Queue"));
    });

    // Create maps for relationship resolution
    const localNodeMap = new Map(nodes.map((n) => [n.id, n]));
    const localShortIdMap = new Map<string, string[]>();

    nodes.forEach(node => {
      if (!node.id) {
        console.warn(`[Bridge] Node missing id, skipping:`, node);
        return;
      }
      const cleanId = node.id.replace(/^c4_/, "");
      const parts = cleanId.split(".");
      const shortName = parts[parts.length - 1];
      if (!localShortIdMap.has(shortName)) localShortIdMap.set(shortName, []);
      localShortIdMap.get(shortName)!.push(node.id);
    });

    // Add relationships
    if (relations) {
      relations.forEach((rel: any) => {
        // Validate relationship has required fields
        if (!rel.from || !rel.to) {
          console.warn(`[Bridge] Relationship missing from/to, skipping:`, rel);
          return;
        }

        let fromId = `c4_${rel.from}`;
        let toId = `c4_${rel.to}`;

        // Try to resolve if missing
        if (!localNodeMap.has(fromId)) {
          const candidates = localShortIdMap.get(rel.from);
          if (candidates && candidates.length === 1) {
            fromId = candidates[0];
          } else if (candidates && candidates.length > 1) {
            console.warn(`[Bridge] Ambiguous source node '${rel.from}'. Candidates: ${candidates.join(", ")}`);
            fromId = candidates[0];
          } else {
            console.warn(`[Bridge] Source node '${rel.from}' not found, skipping relationship`);
            return;
          }
        }

        if (!localNodeMap.has(toId)) {
          const candidates = localShortIdMap.get(rel.to);
          if (candidates && candidates.length === 1) {
            toId = candidates[0];
          } else if (candidates && candidates.length > 1) {
            console.warn(`[Bridge] Ambiguous target node '${rel.to}'. Candidates: ${candidates.join(", ")}`);
            toId = candidates[0];
          } else {
            console.warn(`[Bridge] Target node '${rel.to}' not found, skipping relationship`);
            return;
          }
        }

        relationships.push({
          from: fromId,
          to: toId,
          id: `rel_${fromId.replace("c4_", "")}_${toId.replace("c4_", "")}`,
          label: rel.label || rel.verb,
        });
      });
    }
  }

  return { nodes: new Map(nodes.map((n) => [n.id, n])), relationships };
}

/**
 * Convert layout result back to React Flow format
 */
function convertResultToReactFlow<T extends C4NodeData = C4NodeData>(
  result: LayoutResult,
  originalNodes: Node<T>[],
  originalEdges: Edge[]
): { nodes: Node<T>[]; edges: Edge[] } {
  const positionedMap = new Map<string, LayoutNode>(result.nodes);

  // Track nodes that weren't positioned to give them non-overlapping default positions
  const unpositionedNodes: Node<T>[] = [];

  const updatedNodes = originalNodes.map((node, index) => {
    const positioned: LayoutNode | undefined = positionedMap.get(`c4_${node.id}`);
    // If node wasn't positioned by layout engine, give it a default position
    if (!positioned) {
      // Ensure node has default dimensions if missing
      const defaultWidth = 180;
      const defaultHeight = 100;
      const defaultNode = {
        ...node,
        // Use grid layout for unpositioned nodes to avoid overlaps
        position: {
          x: (index % 5) * (defaultWidth + 50),
          y: Math.floor(index / 5) * (defaultHeight + 50),
        },
        width: node.width || defaultWidth,
        height: node.height || defaultHeight,
        style: {
          ...node.style,
          width: node.width || (node.style as any)?.width || defaultWidth,
          height: node.height || (node.style as any)?.height || defaultHeight,
        },
      };
      unpositionedNodes.push(defaultNode);
      return defaultNode;
    }

    // Debug logging for problematic nodes
    // Note: node.id is the original ID (Customer, WebApp, etc.), not c4_ prefixed
    const isProblematic = node.id.includes('Customer') || node.id.includes('WebApp') ||
      node.id.includes('ProductDB') || node.id.includes('OrderDB') ||
      node.id.includes('UserDB') || node.id.includes('Database');

    if (isProblematic && typeof process !== 'undefined' && process.env?.LAYOUT_DEBUG) {
      console.log(`[Bridge] Converting ${node.id}:`, {
        layoutId: `c4_${node.id}`,
        absolute: `(${positioned.bbox.x.toFixed(1)}, ${positioned.bbox.y.toFixed(1)})`,
        size: `${positioned.bbox.width}x${positioned.bbox.height}`,
        parent: positioned.parent ? positioned.parent.id : 'none',
        parentPos: positioned.parent ? `(${positioned.parent.bbox.x.toFixed(1)}, ${positioned.parent.bbox.y.toFixed(1)})` : 'N/A'
      });
    }

    // CRITICAL: React Flow uses relative positions for children
    // If node has a parent, convert absolute position to relative
    let positionX = positioned.bbox.x;
    let positionY = positioned.bbox.y;

    // Use positioned.parent directly (it's a LayoutNode reference, not just an ID)
    if (positioned.parent) {
      // Convert absolute to relative: child position relative to parent's top-left (0,0)
      positionX = positioned.bbox.x - positioned.parent.bbox.x;
      positionY = positioned.bbox.y - positioned.parent.bbox.y;

      if (isProblematic && typeof process !== 'undefined' && process.env?.LAYOUT_DEBUG) {
        console.log(`[Bridge] ${node.id} relative position: (${positionX.toFixed(1)}, ${positionY.toFixed(1)})`);
      }

      // Safety check: ensure relative position is valid (should be >= 0 with padding)
      // If negative, it means the child was outside the parent in absolute space
      // This should have been fixed by containment enforcement, but double-check
      const MIN_PADDING = 50;
      if (positionX < MIN_PADDING || positionY < MIN_PADDING + 30) {
        console.warn(
          `[Bridge] Child ${node.id} has invalid relative position (${positionX.toFixed(1)}, ${positionY.toFixed(1)}) ` +
          `after conversion from absolute (${positioned.bbox.x.toFixed(1)}, ${positioned.bbox.y.toFixed(1)}) ` +
          `relative to parent ${positioned.parent.id} at (${positioned.parent.bbox.x.toFixed(1)}, ${positioned.parent.bbox.y.toFixed(1)})`
        );
        // Clamp to minimum padding
        positionX = Math.max(MIN_PADDING, positionX);
        positionY = Math.max(MIN_PADDING + 30, positionY);
      }
    }

    return {
      ...node,
      position: {
        x: positionX,
        y: positionY,
      },
      // CRITICAL: Set width and height directly on node for React Flow
      // React Flow uses these for collision detection and layout
      width: positioned.bbox.width,
      height: positioned.bbox.height,
      style: {
        ...node.style,
        width: positioned.bbox.width,
        height: positioned.bbox.height,
      },
      data: {
        ...node.data,
        layoutMetrics: {
          qualityGrade: result.quality.grade,
          qualityScore: result.quality.score,
          processingTime: result.metrics.processingTime,
          edgeCrossings: result.metrics.edgeCrossings,
        },
      },
    };
  });

  // Create a map of layout edges by source/target for matching
  const layoutEdgeMap = new Map<string, LayoutEdge>();
  for (const layoutEdge of result.edges) {
    // Match by source and target IDs (layout engine uses c4_ prefix)
    const sourceId = layoutEdge.source.id.replace(/^c4_/, '');
    const targetId = layoutEdge.target.id.replace(/^c4_/, '');
    const key = `${sourceId}::${targetId}`;
    layoutEdgeMap.set(key, layoutEdge);
    // Also try with edge ID if it matches
    if (layoutEdge.id) {
      const edgeId = layoutEdge.id.replace(/^c4_/, '').replace(/^rel_/, '');
      layoutEdgeMap.set(edgeId, layoutEdge);
    }
  }

  const updatedEdges = originalEdges.map((edge) => {
    // Try to find matching layout edge
    const key = `${edge.source}::${edge.target}`;
    const layoutEdge = layoutEdgeMap.get(key) || layoutEdgeMap.get(edge.id);

    const edgeData: any = {
      ...edge.data,
      layoutMetrics: {
        qualityGrade: result.quality.grade,
        qualityScore: result.quality.score,
        processingTime: result.metrics.processingTime,
        edgeCrossings: result.metrics.edgeCrossings,
      },
    };

    // If we found a matching layout edge, extract points and label position
    if (layoutEdge) {
      // Convert points from layout engine (absolute coordinates) to React Flow coordinates
      // Points need to be relative to the viewport, not absolute
      edgeData.points = layoutEdge.points.map((p: { x: number; y: number }) => ({ x: p.x, y: p.y }));
      edgeData.labelPosition = layoutEdge.labelPosition
        ? { x: layoutEdge.labelPosition.x, y: layoutEdge.labelPosition.y }
        : undefined;
    }

    return {
      ...edge,
      type: "routed",
      data: edgeData,
    };
  });

  // CRITICAL: Final overlap fix SHOULD BE DISABLED
  // This "safety net" is too aggressive and causes layout instability/explosion
  // The core layout engine is now robust enough to handle overlaps correctly
  // const finalNodes = fixOverlapsInReactFlowSpace(updatedNodes);
  const finalNodes = updatedNodes;

  return { nodes: finalNodes, edges: updatedEdges };
}

/**
 * Fix overlaps in React Flow coordinate space (after absolute->relative conversion)
 * This is a final safety net to catch any overlaps that persist after layout optimization
 * DISABLED: This function causes instability. Kept for reference but not called.
 */
/**
 * Calculate absolute position of a node, recursively walking up parent chain
 */
function getAbsolutePosition<T extends C4NodeData = C4NodeData>(
  node: Node<T>,
  nodes: Node<T>[]
): { x: number; y: number } {
  let x = node.position.x;
  let y = node.position.y;

  let currentId = node.parentId;
  while (currentId) {
    const parent = nodes.find((n) => n.id === currentId);
    if (!parent) break;
    x += parent.position.x;
    y += parent.position.y;
    currentId = parent.parentId;
  }

  return { x, y };
}

/**
 * Calculate depth of a node in the hierarchy (0 for root nodes)
 */
function getNodeDepth<T extends C4NodeData = C4NodeData>(
  node: Node<T>,
  nodes: Node<T>[]
): number {
  let depth = 0;
  let currentId = node.parentId;
  while (currentId) {
    depth++;
    const parent = nodes.find((n) => n.id === currentId);
    if (!parent) break;
    currentId = parent.parentId;
  }
  return depth;
}

// @ts-ignore: Unused because we disabled the safety net which caused instability
function fixOverlapsInReactFlowSpace<T extends C4NodeData = C4NodeData>(nodes: Node<T>[]): Node<T>[] {
  const fixedNodes = [...nodes];
  const padding = 30; // Increased minimum separation
  let hasOverlap = true;
  let iterations = 0;
  const maxIterations = 15; // Increased iterations for better overlap resolution

  while (hasOverlap && iterations < maxIterations) {
    hasOverlap = false;
    iterations++;

    for (let i = 0; i < fixedNodes.length; i++) {
      const a = fixedNodes[i];
      // Get width/height from node directly or from style
      const aWidth = a.width || (a.style as any)?.width || 180;
      const aHeight = a.height || (a.style as any)?.height || 100;
      if (!aWidth || !aHeight) continue;

      // Calculate absolute position for a (recursively for nested children)
      const aAbs = getAbsolutePosition(a, fixedNodes);
      const aX = aAbs.x;
      const aY = aAbs.y;
      const aRight = aX + aWidth;
      const aBottom = aY + aHeight;

      for (let j = i + 1; j < fixedNodes.length; j++) {
        const b = fixedNodes[j];
        // Get width/height from node directly or from style
        const bWidth = b.width || (b.style as any)?.width || 180;
        const bHeight = b.height || (b.style as any)?.height || 100;
        if (!bWidth || !bHeight) continue;

        // Skip parent-child relationships (children should be inside parents)
        if (a.parentId === b.id || b.parentId === a.id) continue;

        // Skip if one is an ancestor of the other
        let isAncestor = false;
        let checkId = a.parentId;
        while (checkId) {
          if (checkId === b.id) {
            isAncestor = true;
            break;
          }
          const parent = fixedNodes.find((n) => n.id === checkId);
          checkId = parent?.parentId;
        }
        if (!isAncestor) {
          checkId = b.parentId;
          while (checkId) {
            if (checkId === a.id) {
              isAncestor = true;
              break;
            }
            const parent = fixedNodes.find((n) => n.id === checkId);
            checkId = parent?.parentId;
          }
        }
        if (isAncestor) continue;

        // Calculate absolute position for b (recursively for nested children)
        const bAbs = getAbsolutePosition(b, fixedNodes);
        const bX = bAbs.x;
        const bY = bAbs.y;
        const bRight = bX + bWidth;
        const bBottom = bY + bHeight;

        // Check overlap
        const overlapsX = aX < bRight && aRight > bX;
        const overlapsY = aY < bBottom && aBottom > bY;

        if (overlapsX && overlapsY) {
          hasOverlap = true;

          // Calculate overlap
          const overlapWidth = Math.min(aRight - bX, bRight - aX);
          const overlapHeight = Math.min(aBottom - bY, bBottom - aY);

          // Fix: move the node that's easier to move (root nodes or nodes without strict constraints)
          const aIsRoot = !a.parentId;
          const bIsRoot = !b.parentId;
          const minSeparation = padding + 50; // Increased separation for better spacing

          // Determine which node to move and calculate new position
          // Prefer moving root nodes over children
          let nodeToMove: Node<T>;
          let moveIndex: number;
          let newAbsX: number;
          let newAbsY: number;

          if (aIsRoot && !bIsRoot) {
            // Move a (root) away from b (child)
            nodeToMove = a;
            moveIndex = i;
            const pushDist = Math.max(minSeparation, Math.max(overlapWidth, overlapHeight) * 2.5);
            if (overlapWidth < overlapHeight) {
              newAbsX = aX;
              newAbsY = aY + pushDist;
            } else {
              newAbsX = aX + pushDist;
              newAbsY = aY;
            }
          } else if (!aIsRoot && bIsRoot) {
            // Move b (root) away from a (child)
            nodeToMove = b;
            moveIndex = j;
            const pushDist = Math.max(minSeparation, Math.max(overlapWidth, overlapHeight) * 2.5);
            if (overlapWidth < overlapHeight) {
              newAbsX = bX;
              newAbsY = bY + pushDist;
            } else {
              newAbsX = bX + pushDist;
              newAbsY = bY;
            }
          } else if (aIsRoot && bIsRoot) {
            // Both root - move b
            nodeToMove = b;
            moveIndex = j;
            const pushDist = Math.max(minSeparation, Math.max(overlapWidth, overlapHeight) * 2.5);
            if (overlapWidth < overlapHeight) {
              newAbsX = bX;
              newAbsY = bY + pushDist;
            } else {
              newAbsX = bX + pushDist;
              newAbsY = bY;
            }
          } else {
            // Both are children - move the one with fewer ancestors (closer to root)
            const aDepth = getNodeDepth(a, fixedNodes);
            const bDepth = getNodeDepth(b, fixedNodes);
            if (aDepth <= bDepth) {
              nodeToMove = a;
              moveIndex = i;
              const pushDist = Math.max(minSeparation, Math.max(overlapWidth, overlapHeight) * 2.0);
              if (overlapWidth < overlapHeight) {
                newAbsX = aX;
                newAbsY = aY + pushDist;
              } else {
                newAbsX = aX + pushDist;
                newAbsY = aY;
              }
            } else {
              nodeToMove = b;
              moveIndex = j;
              const pushDist = Math.max(minSeparation, Math.max(overlapWidth, overlapHeight) * 2.0);
              if (overlapWidth < overlapHeight) {
                newAbsX = bX;
                newAbsY = bY + pushDist;
              } else {
                newAbsX = bX + pushDist;
                newAbsY = bY;
              }
            }
          }

          // Convert absolute position back to relative if node has a parent
          let newRelX = newAbsX;
          let newRelY = newAbsY;
          if (nodeToMove.parentId) {
            const parentAbs = getAbsolutePosition(
              fixedNodes.find((n) => n.id === nodeToMove.parentId)!,
              fixedNodes
            );
            newRelX = newAbsX - parentAbs.x;
            newRelY = newAbsY - parentAbs.y;
          }

          // Update the node with new position
          fixedNodes[moveIndex] = {
            ...nodeToMove,
            position: { x: newRelX, y: newRelY },
            width: nodeToMove === a ? aWidth : bWidth,
            height: nodeToMove === a ? aHeight : bHeight,
          };
          // For siblings, rely on layout engine - don't fix here to avoid breaking containment
        }
      }
    }
  }

  return fixedNodes;
}

/**
 * Fallback simple layout
 */
function applyFallbackLayout<T extends C4NodeData = C4NodeData>(
  nodes: Node<T>[],
  edges: Edge[]
): { nodes: Node<T>[]; edges: Edge[] } {
  const gridSize = 200;
  const updatedNodes = nodes.map((node, index) => ({
    ...node,
    position: {
      x: (index % 5) * gridSize,
      y: Math.floor(index / 5) * gridSize,
    },
    style: {
      ...node.style,
      width: 180,
      height: 100,
    },
    data: {
      ...node.data,
      layoutMetrics: {
        fallbackUsed: true,
      },
    },
  }));

  const updatedEdges = edges.map((edge) => ({
    ...edge,
    type: "fallback",
    data: {
      ...edge.data,
      fallbackUsed: true,
    },
  }));

  return { nodes: updatedNodes, edges: updatedEdges };
}

// Export development utilities - modular engine is always used
export const isNewLayoutEngineEnabled = () => true;

export function enableNewLayoutEngine() {
  console.log("ℹ️ Modular layout engine is always enabled");
}

export function disableNewLayoutEngine() {
  console.log("ℹ️ Cannot disable - legacy engine has been removed");
}

// Export for testing
export async function testNewLayoutEngine<T extends C4NodeData = C4NodeData>(
  nodes: Node<T>[],
  edges: Edge[],
  architectureData?: ArchitectureJSON,
  options?: any
) {
  return applySrujaLayout(nodes, edges, architectureData, options ?? { level: "L1" });
}
