/**
 * Unified Level-Aware Router
 *
 * Single routing interface that dispatches to level-specific strategies:
 * - L0: Simple Manhattan (grid-based)
 * - L1: Manhattan with boundary awareness
 * - L2: Node-avoidant Manhattan
 * - L3: Lane-aware routing
 *
 * Upgraded to "Phase 5" robust routing specs (BottomUpLayoutPlanner.md)
 */

import type { Point } from "../geometry/point";
import type { Rect } from "../geometry/rect";
import { SpatialGrid, getPaddedRect } from "./spatial-index";
import { NO_ENTRY_MARGIN, LANE_HEIGHT } from "../constants";

export type RoutingLevel = "L0" | "L1" | "L2" | "L3";

export interface RouteNode {
  id: string;
  bbox: Rect;
  isExternal?: boolean;
  laneIndex?: number;
}

export interface RouteBoundary {
  bbox: Rect;
}

export interface RouteLane {
  index: number;
  y: number;
  height: number;
}

export interface RouteEdge {
  id: string;
  sourceId: string;
  targetId: string;
}

export interface RouteResult {
  points: Point[];
  bendCount: number;
  totalLength: number;
}

export type PortSide = "north" | "south" | "east" | "west";

export interface Port {
  side: PortSide;
  position: Point;
}

// --- Geometry Helpers ---

function segmentIntersectsRect(
  p1: Point,
  p2: Point,
  ids: string[],
  nodes: Map<string, RouteNode>
): boolean {
  // Treat as strict intersection (not just touching)
  const minX = Math.min(p1.x, p2.x);
  const maxX = Math.max(p1.x, p2.x);
  const minY = Math.min(p1.y, p2.y);
  const maxY = Math.max(p1.y, p2.y);

  const isHorizontal = Math.abs(p1.y - p2.y) < 0.1;
  const isVertical = Math.abs(p1.x - p2.x) < 0.1;

  for (const id of ids) {
    const node = nodes.get(id);
    if (!node) continue;

    const r = getPaddedRect(node.bbox, 5); // Small margin

    if (isHorizontal) {
      // Y must be within range, and X span must overlap
      if (p1.y > r.y && p1.y < r.y + r.height) {
        if (maxX > r.x && minX < r.x + r.width) return true;
      }
    } else if (isVertical) {
      // X must be within range, and Y span must overlap
      if (p1.x > r.x && p1.x < r.x + r.width) {
        if (maxY > r.y && minY < r.y + r.height) return true;
      }
    }
  }
  return false;
}

// --- Smart Port Selection ---

/**
 * Generate multiple ports along each side of a node to reduce edge congestion
 * More ports = better edge distribution = lower congestion
 */
function generatePorts(node: RouteNode, padding: number): Port[] {
  const ports: Port[] = [];
  const { x, y, width, height } = node.bbox;

  // Number of ports per side - more for larger nodes
  // For small nodes (width/height < 200): 3 ports per side
  // For medium nodes (200-400): 5 ports per side
  // For large nodes (>400): 7 ports per side
  const horizontalPorts = width < 200 ? 3 : width < 400 ? 5 : 7;
  const verticalPorts = height < 200 ? 3 : height < 400 ? 5 : 7;

  // Generate ports along each side
  // East side (right)
  for (let i = 0; i < horizontalPorts; i++) {
    const ratio = (i + 1) / (horizontalPorts + 1); // Distribute evenly, avoid corners
    ports.push({
      side: "east",
      position: { x: x + width + padding, y: y + height * ratio },
    });
  }

  // West side (left)
  for (let i = 0; i < horizontalPorts; i++) {
    const ratio = (i + 1) / (horizontalPorts + 1);
    ports.push({
      side: "west",
      position: { x: x - padding, y: y + height * ratio },
    });
  }

  // South side (bottom)
  for (let i = 0; i < verticalPorts; i++) {
    const ratio = (i + 1) / (verticalPorts + 1);
    ports.push({
      side: "south",
      position: { x: x + width * ratio, y: y + height + padding },
    });
  }

  // North side (top)
  for (let i = 0; i < verticalPorts; i++) {
    const ratio = (i + 1) / (verticalPorts + 1);
    ports.push({
      side: "north",
      position: { x: x + width * ratio, y: y - padding },
    });
  }

  return ports;
}

// Port usage tracking to avoid congestion - shared across all edges
const portUsageMap = new Map<string, Map<string, number>>(); // nodeId -> portKey -> usage count

function getPortKey(port: Port): string {
  // Create unique key for port position (rounded to avoid floating point issues)
  return `${port.side}-${Math.round(port.position.x)}-${Math.round(port.position.y)}`;
}

function getPortUsage(nodeId: string, port: Port): number {
  const nodeUsage = portUsageMap.get(nodeId);
  if (!nodeUsage) return 0;
  const key = getPortKey(port);
  return nodeUsage.get(key) || 0;
}

function incrementPortUsage(nodeId: string, port: Port): void {
  if (!portUsageMap.has(nodeId)) {
    portUsageMap.set(nodeId, new Map());
  }
  const nodeUsage = portUsageMap.get(nodeId)!;
  const key = getPortKey(port);
  nodeUsage.set(key, (nodeUsage.get(key) || 0) + 1);
}

export function resetPortUsage(): void {
  portUsageMap.clear();
}

export function pickSmartPort(
  node: RouteNode,
  towardNode: RouteNode,
  obstacles: string[],
  nodes: Map<string, RouteNode>,
  nodeId?: string // Optional node ID for usage tracking
): Port {
  const srcCenter = {
    x: node.bbox.x + node.bbox.width / 2,
    y: node.bbox.y + node.bbox.height / 2,
  };
  const dstCenter = {
    x: towardNode.bbox.x + towardNode.bbox.width / 2,
    y: towardNode.bbox.y + towardNode.bbox.height / 2,
  };

  const dx = dstCenter.x - srcCenter.x;
  const dy = dstCenter.y - srcCenter.y;
  const padding = NO_ENTRY_MARGIN;

  // Generate multiple ports along each side for better edge distribution
  const ports = generatePorts(node, padding);

  // Score each port based on direction, distance, and usage diversity
  const scoredPorts = ports.map((port) => {
    const directionScore = getDirectionScore(port.side, dx, dy);
    const distanceScore = getDistanceScore(port.position, dstCenter);
    // Diversity bonus: prefer unused or less-used ports (penalty for high usage)
    const usageCount = nodeId ? getPortUsage(nodeId, port) : 0;
    const diversityBonus = Math.max(0, 20 - usageCount * 5); // Up to 20 bonus for unused ports

    return {
      port,
      score: directionScore + distanceScore + diversityBonus,
    };
  });

  // Sort by score (higher is better)
  scoredPorts.sort((a, b) => b.score - a.score);

  // Check clearance (short ray cast) - try best ports first
  for (const { port } of scoredPorts) {
    // Cast a small ray outward
    const rayEnd = { ...port.position };
    if (port.side === "east") rayEnd.x += 20;
    if (port.side === "west") rayEnd.x -= 20;
    if (port.side === "south") rayEnd.y += 20;
    if (port.side === "north") rayEnd.y -= 20;

    if (!segmentIntersectsRect(port.position, rayEnd, obstacles, nodes)) {
      // Track usage if nodeId provided
      if (nodeId) {
        incrementPortUsage(nodeId, port);
      }
      return port;
    }
  }

  // Fallback: Best directional port even if blocked (router will handle detour)
  const selectedPort = scoredPorts[0].port;
  if (nodeId) {
    incrementPortUsage(nodeId, selectedPort);
  }
  return selectedPort;
}

function getDistanceScore(portPos: Point, targetCenter: Point): number {
  // Prefer ports closer to target (inverse distance, normalized)
  const dx = targetCenter.x - portPos.x;
  const dy = targetCenter.y - portPos.y;
  const distance = Math.sqrt(dx * dx + dy * dy);
  // Normalize: closer ports get higher score (max distance ~1000px gives score ~10)
  return Math.max(0, 10 - distance / 100);
}

function getDirectionScore(side: PortSide, dx: number, dy: number): number {
  // Normalized direction score (0-50 range) to balance with distance score
  // This prevents direction from completely dominating port selection
  let rawScore = 0;
  if (side === "east" && dx > 0) rawScore = Math.abs(dx);
  else if (side === "west" && dx < 0) rawScore = Math.abs(dx);
  else if (side === "south" && dy > 0) rawScore = Math.abs(dy);
  else if (side === "north" && dy < 0) rawScore = Math.abs(dy);

  // Normalize: scale to 0-50 range (typical distances 0-500px)
  // This makes direction and distance scores comparable
  return Math.min(50, rawScore / 10);
}

// --- Robust Manhattan Router ---

function routeManhattan(
  start: Point,
  end: Point,
  startSide: PortSide,
  _endSide: PortSide, // Unused for now
  obstacles: string[],
  nodes: Map<string, RouteNode>
): Point[] {
  // Strategy 1: Straight (L0/L1 mostly) - check alignment
  const dx = Math.abs(end.x - start.x);
  const dy = Math.abs(end.y - start.y);
  const aligned = dx < 10 || dy < 10;

  if (aligned) {
    const straightPath = [start, end];
    if (isValidPath(straightPath, obstacles, nodes)) return straightPath;
  }

  // Strategy 2: L-Shape (1 bend) - try both orientations
  const lPathHV = [start, { x: end.x, y: start.y }, end];
  const lPathVH = [start, { x: start.x, y: end.y }, end];

  // Prefer Horizontal-first for East/West starts; otherwise use end side as tie-breaker
  if (startSide === "east" || startSide === "west") {
    if (isValidPath(lPathHV, obstacles, nodes)) return lPathHV;
    if (isValidPath(lPathVH, obstacles, nodes)) return lPathVH;
  } else {
    const endPrefersHorizontal = _endSide === "east" || _endSide === "west";
    if (endPrefersHorizontal) {
      if (isValidPath(lPathHV, obstacles, nodes)) return lPathHV;
      if (isValidPath(lPathVH, obstacles, nodes)) return lPathVH;
    } else {
      if (isValidPath(lPathVH, obstacles, nodes)) return lPathVH;
      if (isValidPath(lPathHV, obstacles, nodes)) return lPathHV;
    }
  }

  // Strategy 3: Z-Shape (2 bends) - try multiple midpoints with more offsets
  const midX = (start.x + end.x) / 2;
  const midY = (start.y + end.y) / 2;

  // Try horizontal-first Z
  const zPathH = [start, { x: midX, y: start.y }, { x: midX, y: end.y }, end];
  if (isValidPath(zPathH, obstacles, nodes)) return zPathH;

  // Try vertical-first Z
  const zPathV = [start, { x: start.x, y: midY }, { x: end.x, y: midY }, end];
  if (isValidPath(zPathV, obstacles, nodes)) return zPathV;

  // Strategy 4: Try multiple offset midpoints to avoid obstacles and reduce crossings
  const offsets = [40, 60, 80, -40, -60, -80]; // More offset options
  for (const offset of offsets) {
    // Try horizontal offset
    const zPathHOffset = [
      start,
      { x: midX + offset, y: start.y },
      { x: midX + offset, y: end.y },
      end,
    ];
    if (isValidPath(zPathHOffset, obstacles, nodes)) return zPathHOffset;

    // Try vertical offset
    const zPathVOffset = [
      start,
      { x: start.x, y: midY + offset },
      { x: end.x, y: midY + offset },
      end,
    ];
    if (isValidPath(zPathVOffset, obstacles, nodes)) return zPathVOffset;
  }

  // Strategy 5: Push-out / Detour
  // If blocked, try to go around
  return routeDetour(start, end);
}

// Function kept for future use but currently unused
// function canFactorsMatch(_p1: Point, _p2: Point, _side1: PortSide, _side2: PortSide): boolean {
//     // Rough check if direction aligns
//     return true
// }

function isValidPath(points: Point[], obstacles: string[], nodes: Map<string, RouteNode>): boolean {
  for (let i = 0; i < points.length - 1; i++) {
    if (segmentIntersectsRect(points[i], points[i + 1], obstacles, nodes)) return false;
  }
  return true;
}

function routeDetour(start: Point, end: Point): Point[] {
  // Improved detour: try multiple paths to avoid obstacles
  // Strategy 1: Go around the top
  const topPath = [
    start,
    { x: start.x, y: Math.min(start.y, end.y) - 50 }, // Go up
    { x: end.x, y: Math.min(start.y, end.y) - 50 }, // Across
    { x: end.x, y: end.y }, // Down
    end,
  ];

  // Strategy 2: Go around the bottom
  const bottomPath = [
    start,
    { x: start.x, y: Math.max(start.y, end.y) + 50 }, // Go down
    { x: end.x, y: Math.max(start.y, end.y) + 50 }, // Across
    { x: end.x, y: end.y }, // Up
    end,
  ];

  // Strategy 3: Go around the left
  const leftPath = [
    start,
    { x: Math.min(start.x, end.x) - 50, y: start.y }, // Go left
    { x: Math.min(start.x, end.x) - 50, y: end.y }, // Down/up
    { x: end.x, y: end.y }, // Right
    end,
  ];

  // Strategy 4: Go around the right
  const rightPath = [
    start,
    { x: Math.max(start.x, end.x) + 50, y: start.y }, // Go right
    { x: Math.max(start.x, end.x) + 50, y: end.y }, // Down/up
    { x: end.x, y: end.y }, // Left
    end,
  ];

  // Prefer shorter path
  const paths = [topPath, bottomPath, leftPath, rightPath];
  paths.sort((a, b) => pathLength(a) - pathLength(b));
  return paths[0];
}

function pathLength(points: Point[]): number {
  let len = 0;
  for (let i = 1; i < points.length; i++) {
    len += Math.hypot(points[i].x - points[i - 1].x, points[i].y - points[i - 1].y);
  }
  return len;
}

// --- Main Routing Levels ---

export function routeL0(
  src: Port,
  dst: Port,
  obstacles: string[],
  nodes: Map<string, RouteNode>
): RouteResult {
  // L0 is simple, but should still avoid obstacles
  const points = routeManhattan(src.position, dst.position, src.side, dst.side, obstacles, nodes);
  return { points, bendCount: points.length - 2, totalLength: pathLength(points) };
}

export function routeL1(
  src: Port,
  dst: Port,
  obstacles: string[],
  nodes: Map<string, RouteNode>,
  _boundary?: RouteBoundary
): RouteResult {
  const nodesMap = new Map(nodes);
  const obsIds = obstacles.slice();
  if (_boundary) {
    const id = "__boundary__";
    nodesMap.set(id, { id, bbox: _boundary.bbox });
    obsIds.push(id);
  }
  const points = routeManhattan(src.position, dst.position, src.side, dst.side, obsIds, nodesMap);
  return { points, bendCount: points.length - 2, totalLength: pathLength(points) };
}

export function routeL2(
  srcPOrt: Port,
  dstPort: Port,
  obstacles: RouteNode[],
  _spatialIndex?: SpatialGrid
): RouteResult {
  const obsIds = obstacles.map((o) => o.id);
  const nodesMap = new Map(obstacles.map((o) => [o.id, o]));

  // Robust routing with retries
  const points = routeManhattan(
    srcPOrt.position,
    dstPort.position,
    srcPOrt.side,
    dstPort.side,
    obsIds,
    nodesMap
  );

  return {
    points,
    bendCount: points.length - 2,
    totalLength: pathLength(points),
  };
}

/**
 * L3 Routing: Lane-aware routing
 * Components organized in horizontal lanes
 */
export function routeL3(
  sourcePort: Port,
  targetPort: Port,
  sourceLane: number,
  targetLane: number,
  _obstacles: RouteNode[],
  lanes: RouteLane[]
): RouteResult {
  const points: Point[] = [sourcePort.position];
  const start = sourcePort.position;
  const end = targetPort.position;

  // Route through lane corridors
  if (sourceLane === targetLane) {
    // Same lane: simple horizontal route
    points.push({ x: end.x, y: start.y });
  } else {
    // Different lanes: go to corridor, traverse lanes, then to target
    const corridorX = Math.max(start.x, end.x) + 50; // Route through right corridor

    // Exit to corridor
    points.push({ x: corridorX, y: start.y });

    // Traverse to target lane
    const targetLaneY = lanes.find((l) => l.index === targetLane)?.y ?? end.y;
    points.push({ x: corridorX, y: targetLaneY + LANE_HEIGHT / 2 });

    // Enter target lane
    points.push({ x: end.x, y: targetLaneY + LANE_HEIGHT / 2 });
  }

  points.push(end);

  return {
    points,
    bendCount: points.length - 2,
    totalLength: pathLength(points),
  };
}

/**
 * Unified routing entry point
 */
export function routeEdge(
  edge: RouteEdge,
  level: RoutingLevel,
  nodes: Map<string, RouteNode>,
  options: {
    boundary?: RouteBoundary;
    lanes?: RouteLane[];
    spatialIndex?: SpatialGrid;
  } = {}
): RouteResult | null {
  const sourceNode = nodes.get(edge.sourceId);
  const targetNode = nodes.get(edge.targetId);

  if (!sourceNode || !targetNode) return null;

  // Filter obstacles (don't collide with self)
  const obstacles = [...nodes.values()].filter(
    (n) => n.id !== edge.sourceId && n.id !== edge.targetId
  );
  const obstacleIds = obstacles.map((o) => o.id);

  // Phase 5: Smart Port Selection (with usage tracking for diversity)
  // Use node IDs for port usage tracking to spread edges across ports
  const sourcePort = pickSmartPort(sourceNode, targetNode, obstacleIds, nodes, edge.sourceId);
  const targetPort = pickSmartPort(targetNode, sourceNode, obstacleIds, nodes, edge.targetId);

  switch (level) {
    case "L0":
      return routeL0(sourcePort, targetPort, obstacleIds, nodes);

    case "L1":
      return routeL1(sourcePort, targetPort, obstacleIds, nodes, options.boundary);

    case "L2": {
      return routeL2(sourcePort, targetPort, obstacles, options.spatialIndex);
    }

    case "L3": {
      const sourceLane = sourceNode.laneIndex ?? 0;
      const targetLane = targetNode.laneIndex ?? 0;
      return routeL3(
        sourcePort,
        targetPort,
        sourceLane,
        targetLane,
        obstacles,
        options.lanes ?? []
      );
    }
  }
}
