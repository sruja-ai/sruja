/**
 * Hierarchy Building Phase
 * Constructs parent-child relationships and calculates node depths
 */

import type { LayoutPhase, LayoutContext, LayoutNode } from "../core/types";

export function createHierarchyPhase(): LayoutPhase {
  return {
    name: "hierarchy",
    description: "Build hierarchy and calculate node depths",
    dependencies: [],
    execute: (context: LayoutContext): LayoutContext => {
      const nodes = new Map<string, LayoutNode>();
      // CRITICAL: Sort nodes for deterministic processing order
      const nodeArray = Array.from(context.graph.nodes.values())
        .sort((a, b) => a.id.localeCompare(b.id));

      // First pass: create basic nodes
      for (const c4Node of nodeArray) {
        const isVisible = !context.view.hiddenNodes.has(c4Node.id) && !c4Node.hidden;

        nodes.set(c4Node.id, {
          id: c4Node.id,
          original: c4Node,
          bbox: {
            x: c4Node.position?.x || 0,
            y: c4Node.position?.y || 0,
            width: c4Node.size?.width || 200,
            height: c4Node.size?.height || 100,
          },
          contentBox: {
            x: c4Node.position?.x || 0,
            y: c4Node.position?.y || 0,
            width: c4Node.size?.width || 200,
            height: c4Node.size?.height || 100,
          },
          labelBox: {
            x: (c4Node.position?.x || 0) + 10,
            y: (c4Node.position?.y || 0) + 10,
            width: (c4Node.size?.width || 200) - 20,
            height: 30,
          },
          parent: undefined,
          children: [],
          depth: 0,
          level: c4Node.level,
          collapsed: !!c4Node.collapsed,
          visible: isVisible,
          zIndex: 0,
          ports: [],
          constraints: {
            position: c4Node.position,
            size: c4Node.size,
            padding: 50,
          },
          metadata: {
            processingOrder: 0,
            weight: c4Node.layoutPriority || 1,
            importance: 1,
            special: false,
            tags: Array.from(c4Node.tags || []),
          },
        });
      }

      // Second pass: establish parent-child relationships
      for (const c4Node of nodeArray) {
        const node = nodes.get(c4Node.id);
        if (!node) continue;

        if (c4Node.parentId) {
          const parent = nodes.get(c4Node.parentId);
          if (parent) {
            node.parent = parent;
            parent.children = [...parent.children, node];
          }
        }
      }

      // Third pass: calculate depths
      const calculateDepth = (node: LayoutNode, depth: number = 0): number => {
        node.depth = depth;
        let maxChildDepth = depth;
        for (const child of node.children) {
          maxChildDepth = Math.max(maxChildDepth, calculateDepth(child, depth + 1));
        }
        return maxChildDepth;
      };

      // CRITICAL: Sort nodes for deterministic processing order
      const sortedNodes = Array.from(nodes.values()).sort((a, b) => a.id.localeCompare(b.id));
      
      for (const node of sortedNodes) {
        if (!node.parent) {
          calculateDepth(node, 0);
        }
      }

      // Update spatial index
      for (const node of sortedNodes) {
        if (node.visible) {
          context.spatialIndex.insert(node);
        }
      }

      return {
        ...context,
        nodes,
      };
    },
    validate: (context: LayoutContext): boolean => {
      return context.graph.nodes.size > 0;
    },
  };
}
