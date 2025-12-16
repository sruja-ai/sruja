/**
 * C4 Level-Aware Layout Engine
 *
 * Uses the new unified layout engine from @sruja/layout
 */

import type { Node, Edge } from "@xyflow/react";
import type { C4NodeData, C4Level } from "../types";
import {
  layout,
  createC4Graph,
  createDefaultViewState,
  SystemContextView,
  ContainerView,
  ComponentView,
  DeploymentView,
  LandscapeView,
  InteractivePreset,
  type C4Node,
  type C4Relationship,
  type C4Id,
} from "@sruja/layout";
import {
  ensureParentChildSizing,
  ensureNodeContentSizing,
  adjustEdgeLabelPosition,
  getNodeWidth,
  getNodeHeight,
  mapTypeToKind,
} from "./layoutEngine"; // Import helper logic

interface LevelLayoutResult {
  nodes: Node<C4NodeData>[];
  edges: Edge[];
}

interface LevelLayoutOptions {
  level: C4Level;
  focusedSystemId?: string;
  focusedContainerId?: string;
  expandedNodes?: Set<string>; // When nodes are expanded, skip circular layout
}

/**
 * Apply Unified C4 Layout
 */
export async function applyC4LevelLayout(
  nodes: Node<C4NodeData>[],
  edges: Edge[],
  options: LevelLayoutOptions
): Promise<LevelLayoutResult> {
  const { level, focusedSystemId, focusedContainerId } = options;

  // 1. Convert React Flow nodes/edges to C4Graph
  const c4Nodes: C4Node[] = nodes.map((n) => {
    return {
      id: n.id as C4Id,
      label: n.data.label || n.id,
      kind: n.data.isExternal ? "ExternalSystem" : mapTypeToKind(n.data.type),
      level: "landscape", // simplified
      parentId: n.data.parentId as C4Id | undefined,
      description: n.data.description,
      technology: n.data.technology,
      tags: new Set(),
      isExternal: n.data.isExternal,
    } as any;
  });

  const badgeNodes: C4Node[] = [];
  nodes.forEach((n) => {
    if (n.data.expanded && n.data.badges && n.data.badges.length > 0) {
      n.data.badges.forEach((b) => {
        badgeNodes.push({
          id: b.id as C4Id,
          label: b.label,
          kind: mapTypeToKind(b.type),
          parentId: n.id as C4Id,
          level: "container",
          tags: new Set(),
          widthHint: b.width,
          heightHint: b.height,
        } as any);
      });
    }
  });

  const allC4Nodes = [...c4Nodes, ...badgeNodes];

  const c4Relationships: C4Relationship[] = edges.map((e) => ({
    id: e.id,
    from: e.source as C4Id,
    to: e.target as C4Id,
    label: (e.data?.label as string) || undefined,
  }));

  // Create valid graph
  // Note: We might have missing parents if incomplete dataset, createC4Graph validates this.
  // We suppress validation errors for visualization robustness.
  let graph;
  try {
    graph = createC4Graph(allC4Nodes, c4Relationships);
  } catch (e) {
    // Fallback: Use what we have, ignoring validation
    graph = {
      nodes: new Map(allC4Nodes.map((n) => [n.id, n])),
      relationships: c4Relationships,
    };
  }

  // 2. Create View State
  // We Map current app state to C4ViewState
  let view = createDefaultViewState();

  switch (level) {
    case "L1":
      view = focusedSystemId ? SystemContextView(focusedSystemId as C4Id) : LandscapeView();
      break;
    case "L2":
      view = focusedSystemId ? ContainerView(focusedSystemId as C4Id) : LandscapeView();
      break;
    case "L3":
      view = focusedContainerId ? ComponentView(focusedContainerId as C4Id) : LandscapeView();
      break;
    case "L4":
      view = DeploymentView("Production");
      break;
  }

  // 3. Run Layout
  // Configure options based on level
  const layoutOptions: any = { ...InteractivePreset };

  if (level === "L1") {
    // L1 uses Star/Radial layout, unless expanded nodes exist (mixed level needs hierarchical)
    if (!options.expandedNodes || options.expandedNodes.size === 0) {
      layoutOptions.strategy = "l1-context";
    }
    if (focusedSystemId) {
      layoutOptions.focusSystemId = focusedSystemId;
    }
  } else if (level === "L4") {
    // L4 Deployment uses LR direction by default
    layoutOptions.direction = "LR";
    delete layoutOptions.strategy;
  } else {
    // L2-L3 or expanded L1 use Sugiyama (default)
    delete layoutOptions.strategy;
  }

  // Increase spacing to improve quality metrics (consistency with Sruja settings)
  layoutOptions.minSize = { width: 180, height: 120 }; // Ensure minimum sizes prevent clipping
  layoutOptions.spacing = {
    ...(InteractivePreset as any).spacing,
    node: {
      ...(InteractivePreset as any).spacing?.node,
      SoftwareSystem: 180, // Increased from 150 for better overlap prevention
      Container: 140, // Increased from 120 for better spacing
      Component: 120, // Increased from 100 for better quality
    },
    padding: {
      ...(InteractivePreset as any).spacing?.padding,
      SoftwareSystem: 160, // Increased from 140 for better containment
      Container: 140, // Increased from 120 for better containment
      Component: 80, // Increased from 60 for better containment
    },
  };

  // Improve edge routing to avoid label overlaps
  layoutOptions.edgeRouting = {
    ...(InteractivePreset as any).edgeRouting,
    algorithm: "orthogonal",
    bendPenalty: 3,
    crossingPenalty: 20,
    segmentLength: 30,
    avoidNodes: true,
    preferOrthogonal: true,
  };

  const result = layout(graph, view, layoutOptions);

  // 4. Map back to React Flow
  // We only want to map the original nodes. Badge nodes (children) should be mapped back to badges data if possible,
  // OR we could just update the parent size and let the App render badges relatively.
  // The App currently renders badges.
  // The layout engine returns absolute positions for everyone.

  // Build a map of parent positions for converting absolute -> relative coords
  const parentPositions = new Map<string, { x: number; y: number }>();
  for (const n of nodes) {
    const posNode = result.nodes.get(n.id);
    if (posNode) {
      parentPositions.set(n.id, { x: posNode.bbox.x, y: posNode.bbox.y });
    }
  }

  const layoutedNodes = nodes.map((n) => {
    const posNode = result.nodes.get(n.id);
    if (posNode) {
      let updatedBadges = n.data.badges;

      // Update badges if we have layout info for them
      if (n.data.badges && n.data.badges.length > 0) {
        updatedBadges = n.data.badges.map((b) => {
          const bNode = result.nodes.get(b.id);
          if (bNode) {
            // Layout returns absolute positions.
            // Badges in the App are relative to parent.
            return {
              ...b,
              x: bNode.bbox.x - posNode.bbox.x,
              y: bNode.bbox.y - posNode.bbox.y,
              width: bNode.bbox.width,
              height: bNode.bbox.height,
            };
          }
          return b;
        });
      }

      // Calculate position: if node has parentId, convert absolute to relative
      let position = { x: posNode.bbox.x, y: posNode.bbox.y };
      if (n.parentId) {
        const parentPos = parentPositions.get(n.parentId);
        if (parentPos) {
          // Position relative to parent, with padding
          position = {
            x: posNode.bbox.x - parentPos.x,
            y: posNode.bbox.y - parentPos.y,
          };
        }
      }

      // Ensure minimum sizes to prevent label clipping
      const minWidth = getNodeWidth(n.data?.type);
      const minHeight = getNodeHeight(n.data?.type);
      const width = Math.max(minWidth, posNode.bbox.width);
      const height = Math.max(minHeight, posNode.bbox.height);

      return {
        ...n,
        position,
        width,
        height,
        style: {
          ...n.style,
          width,
          height,
        },
        data: {
          ...n.data,
          badges: updatedBadges,
        },
      };
    }
    return n;
  });

  // Post-process nodes to ensure adequate content sizing
  const nodesWithContentSizing = ensureNodeContentSizing(layoutedNodes);

  // Post-process: Ensure parent nodes are properly sized to contain their children
  // This is critical when "Mixed Level" diagrams occur (e.g. L1 system expanded to show containers)
  const finalNodes = ensureParentChildSizing(nodesWithContentSizing);

  // Map Edges with label position adjustment (using final nodes for accurate positions)
  const layoutedEdges = edges.map((e) => {
    const resEdge = result.relationships.find((r) => r.relationshipId === e.id);
    if (resEdge) {
      // Adjust label position to avoid overlapping with nodes
      let adjustedLabelPosition = resEdge.labelPosition;
      if (resEdge.labelPosition) {
        adjustedLabelPosition = adjustEdgeLabelPosition(
          resEdge.labelPosition,
          finalNodes,
          e.source,
          e.target
        );
      }

      return {
        ...e,
        type: "relation", // Ensure type is relation
        data: {
          ...e.data,
          points: resEdge.points,
          labelPosition: adjustedLabelPosition,
        },
      };
    }
    return e;
  });

  // Generate Lane Separators for L3 Component diagrams
  const laneSeparatorNodes: Node<C4NodeData>[] = [];
  if (level === "L3") {
    // Group components by Y position to infer lanes
    const componentNodes = layoutedNodes.filter((n) => n.data.type === "component");
    if (componentNodes.length > 0) {
      // Sort by Y position
      const sortedComponents = [...componentNodes].sort((a, b) => a.position.y - b.position.y);

      // Group into Y bands (lanes) - components within 50px of each other are in the same lane
      const lanes: { y: number; components: typeof sortedComponents }[] = [];
      const LANE_TOLERANCE = 50;

      sortedComponents.forEach((comp) => {
        // Find existing lane within tolerance
        let foundLane = false;
        for (const lane of lanes) {
          const avgY =
            lane.components.reduce((sum, c) => sum + c.position.y, 0) / lane.components.length;
          if (Math.abs(comp.position.y - avgY) < LANE_TOLERANCE) {
            lane.components.push(comp);
            foundLane = true;
            break;
          }
        }
        if (!foundLane) {
          lanes.push({ y: comp.position.y, components: [comp] });
        }
      });

      // Sort lanes by Y position
      lanes.sort((a, b) => {
        const avgYA = a.components.reduce((sum, c) => sum + c.position.y, 0) / a.components.length;
        const avgYB = b.components.reduce((sum, c) => sum + c.position.y, 0) / b.components.length;
        return avgYA - avgYB;
      });

      // Determine lane labels based on component names/IDs (matching layout algorithm logic)
      const categorizeLaneLabel = (components: typeof sortedComponents): string => {
        // Check all components in the lane to determine the lane type
        const laneTypes = new Set<string>();
        components.forEach((comp) => {
          const id = comp.id.toLowerCase();
          if (
            id.includes("controller") ||
            id.includes("handler") ||
            id.includes("api") ||
            id.includes("endpoint")
          ) {
            laneTypes.add("controller");
          } else if (
            id.includes("repo") ||
            id.includes("repository") ||
            id.includes("dao") ||
            id.includes("store")
          ) {
            laneTypes.add("repository");
          } else if (id.includes("service") || id.includes("manager") || id.includes("processor")) {
            laneTypes.add("service");
          } else {
            laneTypes.add("other");
          }
        });

        // Use the most common type, or default order
        if (laneTypes.has("controller")) return "Controllers / API Handlers";
        if (laneTypes.has("service")) return "Services / Business Logic";
        if (laneTypes.has("repository")) return "Repositories / Data Access";
        return "Other Components";
      };

      // Create lane separators
      lanes.forEach((lane, index) => {
        const avgY =
          lane.components.reduce((sum, c) => sum + c.position.y, 0) / lane.components.length;
        const minX = Math.min(...lane.components.map((c) => c.position.x));
        const maxX = Math.max(
          ...lane.components.map((c) => {
            const width = (c.width as number) || 200;
            return c.position.x + width;
          })
        );
        const laneWidth = maxX - minX + 100; // Add padding

        const laneLabel = categorizeLaneLabel(lane.components);

        // Create separator node positioned at the top of the lane
        laneSeparatorNodes.push({
          id: `lane-separator-${index}`,
          type: "lane-separator",
          position: {
            x: minX - 50, // Position to the left of components
            y: avgY - 30, // Position above the lane
          },
          data: {
            id: `lane-separator-${index}`,
            label: laneLabel,
            laneName: laneLabel.toLowerCase().replace(/\s+/g, "-"),
            laneWidth,
            type: "lane-separator" as any,
          },
          selectable: false,
          draggable: false,
        });
      });
    }
  }

  return {
    nodes: [...finalNodes, ...laneSeparatorNodes],
    edges: layoutedEdges,
  };
}
