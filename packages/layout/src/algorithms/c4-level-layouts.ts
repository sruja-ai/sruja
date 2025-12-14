/**
 * C4-Level Specific Layout Algorithms
 *
 * Specialized layouts for each C4 abstraction level:
 * - L1 (System Context): Circular-grid around central system
 * - L2 (Container): Grid layout inside system boundary
 * - L3 (Component): Vertical lane-based architecture layout
 */

import type { C4Id } from "../brand";
import type { Point } from "../geometry/point";
import { H_SPACING, V_SPACING, BOUNDARY_PADDING, EXTERNAL_GAP } from "../constants";
import type { C4LayoutOptions } from "../c4-options";
import { InteractivePreset } from "../c4-options";
import { layoutSugiyama } from "./sugiyama";

export interface C4LayoutNode {
  id: C4Id;
  size: { width: number; height: number };
  isExternal?: boolean;
  laneHint?: "controller" | "service" | "repository" | "other";
}

export interface LayoutResult {
  positions: Map<C4Id, Point>;
  boundarySize?: { width: number; height: number };
}

// ============================================================================
// L1: SYSTEM CONTEXT LAYOUT
// Central system with external nodes distributed on 4 sides
// ============================================================================

/**
 * L1 System Context layout
 * Places the primary system in the center with external systems/persons
 * distributed evenly on all 4 sides (top, right, bottom, left)
 */
export function layoutL1SystemContext(
  centralSystem: C4LayoutNode,
  externals: C4LayoutNode[],
  canvasCenter: Point = { x: 600, y: 400 }
): LayoutResult {
  const positions = new Map<C4Id, Point>();

  // Position central system at canvas center
  const systemX = canvasCenter.x - centralSystem.size.width / 2;
  const systemY = canvasCenter.y - centralSystem.size.height / 2;
  positions.set(centralSystem.id, { x: systemX, y: systemY });

  if (externals.length === 0) {
    return { positions };
  }

  // Distribute externals on 4 sides
  const perSide = Math.ceil(externals.length / 4);

  externals.forEach((ext, i) => {
    const side = Math.floor(i / perSide) % 4;
    const indexOnSide = i % perSide;
    const sideCount = Math.min(perSide, externals.length - Math.floor(i / perSide) * perSide);

    let x: number, y: number;
    const centerX = canvasCenter.x;
    const centerY = canvasCenter.y;

    switch (side) {
      case 0: // Top
        x =
          centerX -
          ((sideCount - 1) * H_SPACING) / 2 +
          indexOnSide * H_SPACING -
          ext.size.width / 2;
        y = systemY - EXTERNAL_GAP - ext.size.height;
        break;
      case 1: // Right
        x = systemX + centralSystem.size.width + EXTERNAL_GAP;
        y =
          centerY -
          ((sideCount - 1) * V_SPACING) / 2 +
          indexOnSide * V_SPACING -
          ext.size.height / 2;
        break;
      case 2: // Bottom
        x =
          centerX -
          ((sideCount - 1) * H_SPACING) / 2 +
          indexOnSide * H_SPACING -
          ext.size.width / 2;
        y = systemY + centralSystem.size.height + EXTERNAL_GAP;
        break;
      case 3: // Left
        x = systemX - EXTERNAL_GAP - ext.size.width;
        y =
          centerY -
          ((sideCount - 1) * V_SPACING) / 2 +
          indexOnSide * V_SPACING -
          ext.size.height / 2;
        break;
      default:
        x = centerX;
        y = centerY;
    }

    positions.set(ext.id, { x, y });
  });

  return { positions };
}

// ============================================================================
// L2: CONTAINER LAYOUT
// Grid layout inside system boundary with externals around
// ============================================================================

export interface C4LayoutEdge {
  source: string;
  target: string;
}

/**
 * L2 Container layout
 * Places containers in a grid inside the system boundary
 * External dependencies are placed around the boundary
 *
 * Implements "Rule 4: Directional Flow" via topological sorting hints
 */
export function layoutL2Containers(
  containers: C4LayoutNode[],
  externals: C4LayoutNode[],
  edges: C4LayoutEdge[] = [],
  boundaryStart: Point = { x: 100, y: 100 },
  options: C4LayoutOptions = InteractivePreset
): LayoutResult {
  const positions = new Map<C4Id, Point>();

  if (containers.length === 0) {
    return { positions, boundarySize: { width: 0, height: 0 } };
  }

  // Use Sugiyama Layout for internal structure
  const internalNodes = containers.map((c) => ({ id: c.id, size: c.size }));
  const internalEdges = edges.map((e) => ({ from: e.source as C4Id, to: e.target as C4Id }));

  // Provide a stable ordering hint by label to reduce jitter
  const sorted = [...containers].sort((a, b) => String(a.id).localeCompare(String(b.id)));
  const orderHint: Partial<Record<C4Id, number>> = {};
  sorted.forEach((c, idx) => {
    orderHint[c.id] = idx;
  });

  const constrainedOptions = {
    ...options,
    constraints: {
      ...(options.constraints || {}),
      orderHint,
    },
  };

  const sugiyamaResult = layoutSugiyama(internalNodes, internalEdges, constrainedOptions);

  // Apply internal positions relative to boundaryStart
  // Sugiyama returns 0-indexed coordinates, so we add boundaryStart + padding
  sugiyamaResult.nodes.forEach((node) => {
    positions.set(node.id, {
      x: boundaryStart.x + BOUNDARY_PADDING + node.x,
      y: boundaryStart.y + BOUNDARY_PADDING + node.y,
    });
  });

  // Calculate boundary size
  const boundaryWidth = sugiyamaResult.width + BOUNDARY_PADDING * 2;
  const boundaryHeight = sugiyamaResult.height + BOUNDARY_PADDING * 2;

  // Position externals around the boundary
  if (externals.length > 0) {
    const perSide = Math.ceil(externals.length / 4);
    const centerX = boundaryStart.x + boundaryWidth / 2;
    const centerY = boundaryStart.y + boundaryHeight / 2;

    externals.forEach((ext, i) => {
      const side = Math.floor(i / perSide) % 4;
      const indexOnSide = i % perSide;
      const sideCount = Math.min(
        perSide,
        Math.max(1, externals.length - Math.floor(i / perSide) * perSide)
      );

      let x: number, y: number;

      switch (side) {
        case 0: // Top
          x =
            centerX -
            ((sideCount - 1) * H_SPACING) / 2 +
            indexOnSide * H_SPACING -
            ext.size.width / 2;
          y = boundaryStart.y - EXTERNAL_GAP - ext.size.height;
          break;
        case 1: // Right
          x = boundaryStart.x + boundaryWidth + EXTERNAL_GAP;
          y =
            centerY -
            ((sideCount - 1) * V_SPACING) / 2 +
            indexOnSide * V_SPACING -
            ext.size.height / 2;
          break;
        case 2: // Bottom
          x =
            centerX -
            ((sideCount - 1) * H_SPACING) / 2 +
            indexOnSide * H_SPACING -
            ext.size.width / 2;
          y = boundaryStart.y + boundaryHeight + EXTERNAL_GAP;
          break;
        case 3: // Left
          x = boundaryStart.x - EXTERNAL_GAP - ext.size.width;
          y =
            centerY -
            ((sideCount - 1) * V_SPACING) / 2 +
            indexOnSide * V_SPACING -
            ext.size.height / 2;
          break;
        default:
          x = centerX;
          y = centerY;
      }

      positions.set(ext.id, { x, y });
    });
  }

  return { positions, boundarySize: { width: boundaryWidth, height: boundaryHeight } };
}

// ============================================================================
// L3: COMPONENT LAYOUT
// Vertical lane-based architecture (Controllers → Services → Repositories)
// ============================================================================

type Lane = "controller" | "service" | "repository" | "other";

/**
 * External element types for smart placement
 */
type ExternalType = "database" | "queue" | "service" | "person" | "other";

function categorizeExternal(id: string): ExternalType {
  const lower = id.toLowerCase();
  if (
    lower.includes("db") ||
    lower.includes("database") ||
    lower.includes("datastore") ||
    lower.includes("sql") ||
    lower.includes("mongo") ||
    lower.includes("redis")
  ) {
    return "database";
  }
  if (
    lower.includes("queue") ||
    lower.includes("kafka") ||
    lower.includes("rabbit") ||
    lower.includes("sqs")
  ) {
    return "queue";
  }
  if (
    lower.includes("person") ||
    lower.includes("user") ||
    lower.includes("customer") ||
    lower.includes("admin")
  ) {
    return "person";
  }
  if (lower.includes("service") || lower.includes("api") || lower.includes("gateway")) {
    return "service";
  }
  return "other";
}

/**

/**
 * L3 Enhanced Layout Result with lane metadata
 */
export interface L3LayoutResult extends LayoutResult {
  lanes: {
    name: Lane;
    label: string;
    y: number;
    height: number;
  }[];
}

/**
 * L3 Component layout - ENHANCED
 *
 * Features:
 * - Components organized in horizontal lanes by role
 * - Lane labels and separators for visual clarity
 * - Smart external placement:
 *   - Databases at BOTTOM (data flows down)
 *   - Persons at TOP (user interaction)
 *   - External services on RIGHT
 *   - Other containers on LEFT
 */
export function layoutL3Components(
  components: C4LayoutNode[],
  externals: C4LayoutNode[],
  edges: C4LayoutEdge[] = [],
  boundaryStart: Point = { x: 100, y: 100 },
  options: C4LayoutOptions = InteractivePreset
): L3LayoutResult {
  const positions = new Map<C4Id, Point>();
  const laneMetadata: L3LayoutResult["lanes"] = [];

  if (components.length === 0) {
    return { positions, boundarySize: { width: 0, height: 0 }, lanes: [] };
  }

  // Use Sugiyama Layout for internal structure
  const internalNodes = components.map((c) => ({ id: c.id, size: c.size }));
  const internalEdges = edges.map((e) => ({ from: e.source as C4Id, to: e.target as C4Id }));

  // Build rank constraints from lane hints to create clear lanes
  const rankOf: Partial<Record<C4Id, number>> = {};
  for (const c of components) {
    const lane = c.laneHint || "other";
    const rk =
      lane === "controller" ? 0 : lane === "service" ? 1 : lane === "repository" ? 2 : undefined;
    if (rk !== undefined) {
      rankOf[c.id] = rk;
    }
  }

  const constrainedOptions = {
    ...options,
    constraints: {
      ...(options.constraints || {}),
      rankOf,
    },
  };

  const sugiyamaResult = layoutSugiyama(internalNodes, internalEdges, constrainedOptions);

  // Apply internal positions relative to boundaryStart
  sugiyamaResult.nodes.forEach((node) => {
    positions.set(node.id, {
      x: boundaryStart.x + BOUNDARY_PADDING + node.x,
      y: boundaryStart.y + BOUNDARY_PADDING + node.y,
    });
  });

  // Calculate boundary size
  const boundaryWidth = sugiyamaResult.width + BOUNDARY_PADDING * 2;
  const boundaryHeight = sugiyamaResult.height + BOUNDARY_PADDING * 2;

  // Compute lane metadata from component hints
  // Group components by their lane hint
  const laneGroups = new Map<Lane, { minY: number; maxY: number; components: C4LayoutNode[] }>();
  const laneOrder: Lane[] = ["controller", "service", "repository", "other"];
  const laneLabels: Record<Lane, string> = {
    controller: "Controllers / API Handlers",
    service: "Services / Business Logic",
    repository: "Repositories / Data Access",
    other: "Other Components",
  };

  for (const component of components) {
    const lane = component.laneHint || "other";
    const pos = positions.get(component.id);
    if (!pos) continue;

    const yTop = pos.y;
    const yBottom = pos.y + component.size.height;

    if (!laneGroups.has(lane)) {
      laneGroups.set(lane, { minY: yTop, maxY: yBottom, components: [component] });
    } else {
      const group = laneGroups.get(lane)!;
      group.minY = Math.min(group.minY, yTop);
      group.maxY = Math.max(group.maxY, yBottom);
      group.components.push(component);
    }
  }

  // Build lane metadata in order
  for (const laneName of laneOrder) {
    const group = laneGroups.get(laneName);
    if (group && group.components.length > 0) {
      const LANE_PADDING = 10;
      laneMetadata.push({
        name: laneName,
        label: laneLabels[laneName],
        y: group.minY - LANE_PADDING,
        height: group.maxY - group.minY + LANE_PADDING * 2,
      });
    }
  }

  // Smart external placement by type
  if (externals.length > 0) {
    // Categorize externals
    const databases: C4LayoutNode[] = [];
    const queues: C4LayoutNode[] = [];
    const persons: C4LayoutNode[] = [];
    const services: C4LayoutNode[] = [];
    const others: C4LayoutNode[] = [];

    externals.forEach((ext) => {
      const type = categorizeExternal(String(ext.id));
      switch (type) {
        case "database":
          databases.push(ext);
          break;
        case "queue":
          queues.push(ext);
          break;
        case "person":
          persons.push(ext);
          break;
        case "service":
          services.push(ext);
          break;
        default:
          others.push(ext);
      }
    });

    const centerX = boundaryStart.x + boundaryWidth / 2;
    const centerY = boundaryStart.y + boundaryHeight / 2;

    // Place databases at BOTTOM (data layer)
    databases.forEach((db, i) => {
      const x =
        centerX - ((databases.length - 1) * H_SPACING) / 2 + i * H_SPACING - db.size.width / 2;
      const y = boundaryStart.y + boundaryHeight + EXTERNAL_GAP;
      positions.set(db.id, { x, y });
    });

    // Place persons at TOP (user layer)
    persons.forEach((person, i) => {
      const x =
        centerX - ((persons.length - 1) * H_SPACING) / 2 + i * H_SPACING - person.size.width / 2;
      const y = boundaryStart.y - EXTERNAL_GAP - person.size.height;
      positions.set(person.id, { x, y });
    });

    // Place external services on RIGHT
    const rightItems = [...services, ...queues];
    rightItems.forEach((ext, i) => {
      const x = boundaryStart.x + boundaryWidth + EXTERNAL_GAP;
      const y =
        centerY - ((rightItems.length - 1) * V_SPACING) / 2 + i * V_SPACING - ext.size.height / 2;
      positions.set(ext.id, { x, y });
    });

    // Place other containers on LEFT
    others.forEach((ext, i) => {
      const x = boundaryStart.x - EXTERNAL_GAP - ext.size.width;
      const y =
        centerY - ((others.length - 1) * V_SPACING) / 2 + i * V_SPACING - ext.size.height / 2;
      positions.set(ext.id, { x, y });
    });
  }

  return {
    positions,
    boundarySize: { width: boundaryWidth, height: boundaryHeight },
    lanes: laneMetadata,
  };
}

// ============================================================================
// L4: DEPLOYMENT LAYOUT
// Hierarchical deployment groups with infrastructure nodes
// ============================================================================

export interface DeploymentGroup {
  id: C4Id;
  label: string;
  parentId?: C4Id;
  children?: C4Id[];
  nodes: C4LayoutNode[];
}

export interface L4LayoutResult extends LayoutResult {
  groups: {
    id: C4Id;
    label: string;
    x: number;
    y: number;
    width: number;
    height: number;
  }[];
}

/**
 * L4 Deployment layout
 *
 * Features:
 * - Deployment groups as nested containers (cloud, region, cluster, pod)
 * - Infrastructure nodes (load balancers, databases) positioned at edges
 * - Container instances mapped to deployment nodes
 * - Left-to-right flow (LR direction typical for deployments)
 */
export function layoutL4Deployment(
  deploymentNodes: C4LayoutNode[],
  infrastructureNodes: C4LayoutNode[],
  containerInstances: C4LayoutNode[],
  edges: C4LayoutEdge[] = [],
  groups: DeploymentGroup[] = [],
  boundaryStart: Point = { x: 50, y: 50 },
  options: C4LayoutOptions = InteractivePreset
): L4LayoutResult {
  const positions = new Map<C4Id, Point>();
  const groupMetadata: L4LayoutResult["groups"] = [];

  // Use LR (left-to-right) direction for deployment diagrams
  const direction = options?.direction || "LR";
  const isHorizontal = direction === "LR" || direction === "RL";

  // Combine all nodes for layout
  const allNodes = [...deploymentNodes, ...infrastructureNodes, ...containerInstances];

  if (allNodes.length === 0) {
    return { positions, boundarySize: { width: 0, height: 0 }, groups: [] };
  }

  // If we have groups, layout hierarchically
  if (groups.length > 0) {
    let currentX = boundaryStart.x;
    let currentY = boundaryStart.y;
    let maxHeight = 0;
    let maxWidth = 0;

    // Layout each top-level group
    const topLevelGroups = groups.filter((g) => !g.parentId);

    topLevelGroups.forEach((group, _) => {
      const groupNodes = group.nodes;
      const cols = Math.ceil(Math.sqrt(groupNodes.length));
      const groupHSpacing = H_SPACING * 0.8;
      const groupVSpacing = V_SPACING * 0.8;

      // Position nodes within group
      groupNodes.forEach((node, nodeIndex) => {
        const col = nodeIndex % cols;
        const row = Math.floor(nodeIndex / cols);
        const x = currentX + BOUNDARY_PADDING + col * groupHSpacing;
        const y = currentY + BOUNDARY_PADDING + 40 + row * groupVSpacing; // 40 for group header
        positions.set(node.id, { x, y });
      });

      // Calculate group bounds
      const rows = Math.ceil(groupNodes.length / cols);
      const groupWidth = cols * groupHSpacing + BOUNDARY_PADDING * 2;
      const groupHeight = rows * groupVSpacing + BOUNDARY_PADDING * 2 + 40; // 40 for header

      groupMetadata.push({
        id: group.id,
        label: group.label,
        x: currentX,
        y: currentY,
        width: groupWidth,
        height: groupHeight,
      });

      // Move to next group position
      if (isHorizontal) {
        currentX += groupWidth + EXTERNAL_GAP;
        maxHeight = Math.max(maxHeight, groupHeight);
        maxWidth = currentX - boundaryStart.x;
      } else {
        currentY += groupHeight + EXTERNAL_GAP;
        maxWidth = Math.max(maxWidth, groupWidth);
        maxHeight = currentY - boundaryStart.y;
      }
    });

    return {
      positions,
      boundarySize: {
        width: maxWidth,
        height: maxHeight,
      },
      groups: groupMetadata,
    };
  }

  // No groups - use Sugiyama for hierarchical layout
  const result = layoutSugiyama(
    allNodes,
    edges.map((e) => ({ from: e.source as C4Id, to: e.target as C4Id })),
    {
      ...options,
      direction,
    }
  );

  // Map Sugiyama result to positions
  result.nodes.forEach((node) => {
    positions.set(node.id as C4Id, { x: node.x, y: node.y });
  });

  // Calculate bounds
  let minX = Infinity,
    minY = Infinity,
    maxX = 0,
    maxY = 0;
  positions.forEach((pos, id) => {
    const node = allNodes.find((n) => n.id === id);
    if (node) {
      minX = Math.min(minX, pos.x);
      minY = Math.min(minY, pos.y);
      maxX = Math.max(maxX, pos.x + node.size.width);
      maxY = Math.max(maxY, pos.y + node.size.height);
    }
  });

  return {
    positions,
    boundarySize: {
      width: maxX - minX + BOUNDARY_PADDING * 2,
      height: maxY - minY + BOUNDARY_PADDING * 2,
    },
    groups: [],
  };
}
