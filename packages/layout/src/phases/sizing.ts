import type { LayoutPhase, LayoutContext, LayoutNode } from "../core/types";
import { calculateSizes } from "../algorithms/sizing";
import { CanvasTextMeasurer } from "../utils/canvas-text-measurer";
import { InteractivePreset, type C4LayoutOptions } from "../c4-options";
import type { HierarchyNode, HierarchyTree } from "../algorithms/hierarchy";

export function createSizingPhase(): LayoutPhase {
  return {
    name: "sizing",
    description: "Calculate node sizes based on content and text measurement",
    dependencies: ["hierarchy"],
    execute: (context: LayoutContext): LayoutContext => {
      const nodes = new Map(context.nodes);

      // 1. Convert LayoutNodes to HierarchyTree
      // We need to map the recursive structure
      const rootLayoutNodes = Array.from(nodes.values()).filter(n => !n.parent);

      // Adapter function to map Core C4Node to Algorithm C4Node
      function mapToAlgorithmC4Node(layoutNode: LayoutNode): any {
        const original = layoutNode.original;

        // Map 'type' to 'kind' - prioritize 'kind' if available (from Bridge), fallback to 'type' (from Core)
        const rawNode = original as any;
        let kind = (rawNode.kind || rawNode.type || "") as string;

        // Normalize kinds
        if (kind === 'System') kind = 'SoftwareSystem';
        else if (kind === 'DataStore') kind = 'Database';
        // Check if original type was generic but metadata says specific
        if (original.metadata) {
          // Logic to extract specific type if stored in metadata? 
          // For now assume type property is accurate enough or matches c4-model close enough
        }

        // Map 'level'
        let level = layoutNode.level as string;
        if (level === 'L1') level = 'context';
        else if (level === 'L2') level = 'container';
        else if (level === 'L3') level = 'component';
        else if (level === 'L0') level = 'landscape';

        return {
          id: original.id,
          label: original.label,
          kind: kind,
          level: level,
          parentId: original.parentId,
          description: original.description,
          technology: original.technology,
          tags: new Set<string>(), // Empty set for now
          links: [],
          widthHint: (original.size?.width ?? 0) > 0 ? original.size?.width : undefined, // Respect existing size if set as hint?
          heightHint: (original.size?.height ?? 0) > 0 ? original.size?.height : undefined,
          aspectRatio: undefined,
          layoutPriority: undefined,
          hidden: layoutNode.visible === false,
          pinnedPosition: undefined,
          collapseChildren: layoutNode.collapsed, // CRITICAL FIX
          sortKey: undefined
        };
      }

      // Helper to recursively map LayoutNode to HierarchyNode
      function mapToHierarchy(node: LayoutNode, depth: number): HierarchyNode {
        const hNode: HierarchyNode = {
          id: node.id as any,
          node: mapToAlgorithmC4Node(node), // Use adapter
          parent: undefined,
          children: [], // Populated below
          depth: depth,
          subtreeSize: 1, // Will be calculated if needed, or we can approximate
          subtreeDepth: 0
        };

        if (!node.collapsed && node.children.length > 0) {
          // Only map children if expanded? 
          // Algorithm processes children if they exist in the array. 
          // If collapsed is true (mapped to collapseChildren), algorithm treats as leaf.
          // So we can safely populate children.
          hNode.children = node.children.map(child => {
            const childHNode = mapToHierarchy(child, depth + 1);
            childHNode.parent = hNode;
            return childHNode;
          });
        }

        return hNode;
      }
      const roots = rootLayoutNodes.map(n => mapToHierarchy(n, 0));

      const hierarchyTree: HierarchyTree = {
        roots,
        nodeMap: new Map(), // Not strictly needed for calculateSizes if it traverses roots
        maxDepth: 0
      };

      // 2. Prepare Relationships
      const relationships = context.graph.relationships.map(r => ({
        from: r.from as any,
        to: r.to as any
      }));

      // 3. Prepare Options
      // Use InteractivePreset as base and override with context options
      const measurer = new CanvasTextMeasurer();
      const layoutOptions: C4LayoutOptions = {
        ...InteractivePreset,
        direction: "TB", // Default direction
        minSize: { width: 140, height: 80 }, // Sensible minimums
        maxSize: { width: 2000, height: 2000 },
        spacing: {
          ...InteractivePreset.spacing,
          node: { ...InteractivePreset.spacing.node },
          padding: { ...InteractivePreset.spacing.padding }
        },
        measurer
      };

      // Override specific spacing if provided in context options
      if (context.options.spacing) {
        // Apply relevant spacing overrides if keys match
        // For now, InteractivePreset defaults are good for "Smart" sizing
      }

      // 4. Calculate Sizes
      const sizedNodesMap = calculateSizes(hierarchyTree, relationships, measurer, layoutOptions);

      // 5. Apply results back to context nodes
      for (const [id, sizedNode] of sizedNodesMap) {
        const layoutNode = nodes.get(id as string);
        if (layoutNode) {
          // Update dimensions
          const { width, height } = sizedNode.size;
          const { width: contentWidth, height: contentHeight } = sizedNode.contentSize;

          nodes.set(id as string, {
            ...layoutNode,
            bbox: { ...layoutNode.bbox, width, height },
            contentBox: {
              ...layoutNode.contentBox,
              width: contentWidth,
              height: contentHeight
            },
            // Center the label box in the content area for now, or match styles
            labelBox: {
              x: layoutNode.bbox.x + (width - contentWidth) / 2,
              y: layoutNode.bbox.y + 10,
              width: contentWidth,
              height: contentHeight
            }
          });
        }
      }

      return {
        ...context,
        nodes,
      };
    },
  };
}
