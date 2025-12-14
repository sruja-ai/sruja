// Rules-based layout engine configuration
// Replaces if-else conditions with declarative rules

import type { Node, Edge } from "@xyflow/react";
import type { C4NodeData, C4Level } from "../types";

export interface LayoutRule {
  id: string;
  name: string;
  priority: number; // Higher priority = evaluated first
  condition: (context: LayoutContext) => boolean;
  action: (context: LayoutContext) => LayoutConfig;
}

export interface LayoutContext {
  nodes: Node<C4NodeData>[];
  edges: Edge[];
  currentLevel: C4Level;
  focusedSystemId?: string;
  focusedContainerId?: string;
  expandedNodes: Set<string>;
  nodeCount: number;
  edgeCount: number;
  hasHierarchy: boolean; // Has parent-child relationships
  hasExpandedNodes: boolean;
  averageNodeSize: { width: number; height: number };
  complexity: "simple" | "medium" | "complex"; // Based on node/edge count
  // Extended metrics for better layout selection
  relationshipDensity: number; // edges/nodes ratio (higher = more connected)
  hasBidirectionalEdges: boolean; // Has pairs of edges A->B and B->A
}

export interface LayoutConfig {
  engine: "sruja" | "c4level"; // Only our custom layout engines
  direction: "DOWN" | "RIGHT" | "UP" | "LEFT";
  options?: {
    nodeSpacing?: number;
    layerSpacing?: number;
    [key: string]: any;
  };
  constraints?: {
    orderHint?: Record<string, number>;
    rankOf?: Record<string, number>;
    sameRank?: string[][];
  };
}

/**
 * Analyze layout context from nodes and edges
 */
export function analyzeLayoutContext(
  nodes: Node<C4NodeData>[],
  edges: Edge[],
  currentLevel: C4Level,
  focusedSystemId?: string,
  focusedContainerId?: string,
  expandedNodes?: Set<string>
): LayoutContext {
  const nodeCount = nodes.length;
  const edgeCount = edges.length;
  const hasHierarchy = nodes.some((n) => n.parentId);
  // Check if any nodes are expanded by checking if expandedNodes contains any node IDs
  // Also check if nodes have children (which indicates expansion in the layout)
  const hasExpandedNodes = expandedNodes
    ? expandedNodes.size > 0 ||
      nodes.some((n) => {
        const nodeId = n.id || n.data?.id;
        return expandedNodes.has(nodeId) || n.data?.expanded === true;
      })
    : nodes.some(
        (n) =>
          n.data?.expanded === true || (n.data?.childCount && n.data.childCount > 0 && n.parentId)
      );

  // Calculate average node size
  const totalWidth = nodes.reduce((sum, n) => sum + (n.width || 100), 0);
  const totalHeight = nodes.reduce((sum, n) => sum + (n.height || 100), 0);
  const averageNodeSize = {
    width: nodeCount > 0 ? totalWidth / nodeCount : 100,
    height: nodeCount > 0 ? totalHeight / nodeCount : 100,
  };

  // Determine complexity
  let complexity: "simple" | "medium" | "complex";
  if (nodeCount < 10 && edgeCount < 15) {
    complexity = "simple";
  } else if (nodeCount < 30 && edgeCount < 50) {
    complexity = "medium";
  } else {
    complexity = "complex";
  }

  // Calculate relationship density (edges per node)
  const relationshipDensity = nodeCount > 0 ? edgeCount / nodeCount : 0;

  // Detect bidirectional edges (pairs where A->B and B->A both exist)
  const edgePairs = new Set<string>();
  let hasBidirectionalEdges = false;
  for (const edge of edges) {
    const forwardKey = `${edge.source}->${edge.target}`;
    const reverseKey = `${edge.target}->${edge.source}`;
    if (edgePairs.has(reverseKey)) {
      hasBidirectionalEdges = true;
      break;
    }
    edgePairs.add(forwardKey);
  }

  return {
    nodes,
    edges,
    currentLevel,
    focusedSystemId,
    focusedContainerId,
    expandedNodes: expandedNodes || new Set(),
    nodeCount,
    edgeCount,
    hasHierarchy,
    hasExpandedNodes,
    averageNodeSize,
    complexity,
    relationshipDensity,
    hasBidirectionalEdges,
  };
}

/**
 * Default layout rules (ordered by priority)
 */
export const DEFAULT_LAYOUT_RULES: LayoutRule[] = [
  // Rule 1: Simple diagrams (few nodes) - use C4 level layout
  {
    id: "simple-c4",
    name: "Simple C4 Layout",
    priority: 100,
    condition: (ctx) => ctx.complexity === "simple" && !ctx.hasHierarchy && !ctx.hasExpandedNodes,
    action: () => ({
      engine: "sruja",
      direction: "DOWN",
      options: { nodeSpacing: 200, layerSpacing: 220 }, // Increased spacing to reduce edge crossings
    }),
  },

  // Rule 2: Hierarchical diagrams - use Sruja compound layout
  {
    id: "hierarchical-sruja",
    name: "Hierarchical Sruja Layout",
    priority: 95, // Increased priority - hierarchy is important
    condition: (ctx) => ctx.hasHierarchy,
    action: () => ({
      engine: "sruja",
      direction: "DOWN",
      options: {
        nodeSpacing: 250, // Increased for better edge routing in hierarchical layouts
        layerSpacing: 280, // Increased for better edge routing and containment
      },
    }),
  },

  // Rule 2b: Expanded nodes (showing internal structure) - needs even more spacing
  {
    id: "expanded-hierarchical",
    name: "Expanded Hierarchical Layout",
    priority: 96, // Higher than regular hierarchy
    condition: (ctx) => ctx.hasHierarchy && ctx.hasExpandedNodes,
    action: () => ({
      engine: "sruja",
      direction: "DOWN",
      options: {
        nodeSpacing: 280, // Extra spacing for expanded nodes
        layerSpacing: 320, // Extra vertical space for parent-child containment
      },
    }),
  },

  // Rule 3: L1 level (System Context) - use Sruja with generous spacing
  {
    id: "level-l1",
    name: "L1 System Context Layout",
    priority: 85,
    condition: (ctx) => ctx.currentLevel === "L1" && !ctx.hasExpandedNodes && !ctx.hasHierarchy,
    action: () => ({
      engine: "sruja",
      direction: "DOWN",
      options: {
        nodeSpacing: 220, // Increased for better edge routing and reduced crossings
        layerSpacing: 240, // Increased for better edge routing
      },
    }),
  },

  // Rule 3b: Complex L1 - for dense system contexts
  {
    id: "complex-l1",
    name: "Complex L1 Layout",
    priority: 86, // Higher than standard L1
    condition: (ctx) => ctx.currentLevel === "L1" && !ctx.hasExpandedNodes && ctx.edgeCount > 10,
    action: () => ({
      engine: "sruja",
      direction: "DOWN", // Usually DOWN is best for hierarchy
      options: {
        nodeSpacing: 240, // Increased for dense graphs
        layerSpacing: 280, // Even more vertical space for edge routing
        edgeRouting: "orthogonal", // Enforce orthogonal if engine supports hints
      },
    }),
  },

  // Rule 4: L2 level (containers) - use Sruja for better hierarchy
  {
    id: "level-l2",
    name: "L2 Container Layout",
    priority: 75,
    condition: (ctx) => ctx.currentLevel === "L2",
    action: () => ({
      engine: "sruja",
      direction: "DOWN",
      options: {
        nodeSpacing: 200, // Increased for better edge routing
        layerSpacing: 220, // Increased for better edge routing
      },
    }),
  },

  // Rule 5: L3 level (components) - use Sruja for detailed layouts
  {
    id: "level-l3",
    name: "L3 Component Layout",
    priority: 70,
    condition: (ctx) => ctx.currentLevel === "L3",
    action: () => ({
      engine: "sruja",
      direction: "DOWN",
      options: {},
    }),
  },

  // Rule 6: Complex diagrams - use Sruja for better edge routing
  {
    id: "complex-sruja",
    name: "Complex Sruja Layout",
    priority: 60,
    condition: (ctx) => ctx.complexity === "complex" && ctx.edgeCount > 30,
    action: () => ({
      engine: "sruja",
      direction: "DOWN",
      options: {
        nodeSpacing: 280, // Increased spacing for complex diagrams
        layerSpacing: 300, // More vertical space
      },
    }),
  },

  // Rule 6b: Dense graph - high relationship density needs more spacing
  {
    id: "dense-graph",
    name: "Dense Graph Layout",
    priority: 58,
    condition: (ctx) => ctx.relationshipDensity > 2.0, // More than 2 edges per node
    action: () => ({
      engine: "sruja",
      direction: "DOWN",
      options: {
        nodeSpacing: 300, // Increased from 250 - more space to reduce crossings
        layerSpacing: 320, // Increased from 280 - more vertical space for edge routing
        edgeRouting: "orthogonal",
      },
    }),
  },

  // Rule 6c: Bidirectional flow - prefer horizontal to show back-and-forth
  {
    id: "bidirectional-flow",
    name: "Bidirectional Flow Layout",
    priority: 55,
    condition: (ctx) => ctx.hasBidirectionalEdges && ctx.nodeCount > 5,
    action: () => ({
      engine: "sruja",
      direction: "RIGHT", // Horizontal shows bidirectional flow better
      options: {
        nodeSpacing: 180,
        layerSpacing: 200,
      },
    }),
  },

  // Rule 7: Wide diagrams (many nodes horizontally) - use vertical layout
  {
    id: "wide-vertical",
    name: "Wide Diagram Vertical Layout",
    priority: 50,
    condition: (ctx) => {
      // If average node width is large relative to height, prefer vertical
      const aspectRatio = ctx.averageNodeSize.width / ctx.averageNodeSize.height;
      return aspectRatio > 2.0 && ctx.nodeCount > 15;
    },
    action: () => ({
      engine: "sruja",
      direction: "DOWN",
      options: {},
    }),
  },

  // Rule 8: Tall diagrams (many nodes vertically) - use horizontal layout
  {
    id: "tall-horizontal",
    name: "Tall Diagram Horizontal Layout",
    priority: 45,
    condition: (ctx) => {
      const aspectRatio = ctx.averageNodeSize.width / ctx.averageNodeSize.height;
      return aspectRatio < 0.5 && ctx.nodeCount > 15;
    },
    action: () => ({
      engine: "sruja",
      direction: "RIGHT",
      options: {},
    }),
  },

  // Rule 9: Default fallback - use Sruja for general purpose
  {
    id: "default-sruja",
    name: "Default Sruja Layout",
    priority: 10,
    condition: () => true, // Always matches
    action: () => ({
      engine: "sruja",
      direction: "DOWN",
      options: {},
    }),
  },
];

/**
 * Select layout configuration based on rules
 */
export function selectLayoutConfig(
  nodes: Node<C4NodeData>[],
  edges: Edge[],
  currentLevel: C4Level,
  focusedSystemId?: string,
  focusedContainerId?: string,
  expandedNodes?: Set<string>,
  rules: LayoutRule[] = DEFAULT_LAYOUT_RULES
): LayoutConfig {
  // Analyze context
  const context = analyzeLayoutContext(
    nodes,
    edges,
    currentLevel,
    focusedSystemId,
    focusedContainerId,
    expandedNodes
  );

  // Sort rules by priority (highest first)
  const sortedRules = [...rules].sort((a, b) => b.priority - a.priority);

  // Find first matching rule
  for (const rule of sortedRules) {
    if (rule.condition(context)) {
      const config = rule.action(context);
      console.log(`Layout rule matched: ${rule.name} -> ${config.engine}-${config.direction}`);
      // Augment with deterministic ordering constraint
      const byLabel = [...nodes].sort((a, b) =>
        (a.data?.label || a.id).localeCompare(b.data?.label || b.id)
      );
      const orderHint: Record<string, number> = {};
      byLabel.forEach((n, idx) => {
        orderHint[n.id] = idx;
      });

      // Group persons and databases for clearer rows when hierarchical layout is used
      const persons = nodes.filter((n) => n.data?.type === "person").map((n) => n.id);
      const datastores = nodes.filter((n) => n.data?.type === "datastore").map((n) => n.id);
      const queues = nodes.filter((n) => n.data?.type === "queue").map((n) => n.id);
      const topics = nodes.filter((n) => n.data?.type === "topic").map((n) => n.id);
      const caches = nodes.filter((n) => n.data?.type === "cache").map((n) => n.id);
      const filesystems = nodes.filter((n) => n.data?.type === "filesystem").map((n) => n.id);
      const externalServices = nodes
        .filter(
          (n) =>
            n.data?.type === "external-component" ||
            n.data?.type === "external-container" ||
            (n.data?.type === "system" && n.data?.isExternal)
        )
        .map((n) => n.id);
      const sameRank: string[][] = [];
      if (persons.length > 1) sameRank.push(persons);
      if (datastores.length > 1) sameRank.push(datastores);

      // Level-specific rank constraints
      const rankOf: Record<string, number> = {};
      if (currentLevel === "L1" || currentLevel === "L2") {
        // Persons at TOP (rank 0)
        for (const id of persons) rankOf[id] = 0;
        // Datastores at BOTTOM (high rank)
        for (const id of datastores) rankOf[id] = 100;
        // Queues near bottom
        for (const id of queues) rankOf[id] = 90;
        // Topics just above datastores (below queues)
        for (const id of topics) rankOf[id] = 95;
        // Cache/filesystem along lower band but above queues
        for (const id of caches) rankOf[id] = 85;
        for (const id of filesystems) rankOf[id] = 85;
        // External services mid band
        for (const id of externalServices) rankOf[id] = 60;
      }

      // Boundary-aware row alignment: group siblings under the same parent into rows
      const pushGroupsByParent = (type: string) => {
        const byParent = new Map<string, string[]>();
        for (const n of nodes) {
          if (n.data?.type === type && n.parentId) {
            const pid = n.parentId;
            const arr = byParent.get(pid) || [];
            arr.push(n.id);
            byParent.set(pid, arr);
          }
        }
        for (const arr of byParent.values()) {
          if (arr.length > 1) sameRank.push(arr);
        }
      };
      if (currentLevel === "L2") {
        pushGroupsByParent("container");
      } else if (currentLevel === "L3") {
        pushGroupsByParent("component");
      }

      return {
        ...config,
        constraints: {
          orderHint,
          sameRank,
          rankOf,
        },
      };
    }
  }

  // Fallback (should never reach here due to default rule)
  return {
    engine: "sruja",
    direction: "DOWN",
    options: {},
    constraints: {
      orderHint: Object.fromEntries(nodes.map((n, i) => [n.id, i])),
    },
  };
}

/**
 * Create custom rule
 */
export function createLayoutRule(
  id: string,
  name: string,
  priority: number,
  condition: (context: LayoutContext) => boolean,
  action: (context: LayoutContext) => LayoutConfig
): LayoutRule {
  return { id, name, priority, condition, action };
}

/**
 * Merge custom rules with defaults
 */
export function mergeLayoutRules(
  customRules: LayoutRule[],
  defaultRules: LayoutRule[] = DEFAULT_LAYOUT_RULES
): LayoutRule[] {
  // Combine and deduplicate by ID
  const ruleMap = new Map<string, LayoutRule>();

  // Add defaults first
  defaultRules.forEach((rule) => ruleMap.set(rule.id, rule));

  // Override with custom rules
  customRules.forEach((rule) => ruleMap.set(rule.id, rule));

  return Array.from(ruleMap.values());
}
