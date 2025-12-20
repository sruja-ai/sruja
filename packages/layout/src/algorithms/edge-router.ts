import type { Point } from "../geometry/point";
import type { Rect } from "../geometry/rect";

/**
 * Calculate best port considering obstacles for better routing
 */
export function calculateBestPortWithObstacles(
  source: Rect,
  target: Rect,
  obstacles: Rect[] = [],
  usedPorts?: Map<string, number> // Track how many edges use each port side
): { side: "north" | "south" | "east" | "west"; position: Point; angle: number } {
  const sc = { x: source.x + source.width / 2, y: source.y + source.height / 2 };
  const tc = { x: target.x + target.width / 2, y: target.y + target.height / 2 };

  const srcRight = source.x + source.width;
  const srcBottom = source.y + source.height;
  const srcX = source.x;
  const srcY = source.y;
  const srcWidth = source.width;
  const srcHeight = source.height;

  // Calculate relative position
  const dx = tc.x - sc.x;
  const dy = tc.y - sc.y;
  const absDx = Math.abs(dx);
  const absDy = Math.abs(dy);

  // Candidate ports on all sides
  interface PortCandidate {
    side: "north" | "south" | "east" | "west";
    position: Point;
    angle: number;
    score: number;
  }

  // Generate multiple ports along each side for better edge distribution
  // More ports = better spread = lower congestion
  const candidates: PortCandidate[] = [];

  // Number of ports per side - adaptive based on node size
  // Optimized: cache width/height checks
  const horizontalPorts = srcWidth < 200 ? 3 : srcWidth < 400 ? 5 : 7;
  const verticalPorts = srcHeight < 200 ? 3 : srcHeight < 400 ? 5 : 7;

  // Optimized: pre-calculate denominator for ratio calculations
  const horizontalDenom = horizontalPorts + 1;
  const verticalDenom = verticalPorts + 1;

  // Optimize scoring weights
  // 1. Direction is primary but shouldn't be exclusive (allow side ports even if vertical)
  // 2. Clearance is critical
  // 3. Port usage distribution is important for large diagrams

  // East side (right)
  for (let i = 0; i < horizontalPorts; i++) {
    const ratio = (i + 1) / horizontalDenom; // Distribute evenly
    const y = srcY + srcHeight * ratio;
    // Base score: heavily favor direct line, but allow indirect if needed
    // If dx > 0 (target is right), score = absDx * 2
    // If target is vertical (absDy > absDx), give substantial score to side ports to allow routing around
    let score = 0;
    if (dx > 0) score = absDx * 2;
    else if (absDy > absDx) score = absDx * 0.8; // Allow side port even if vertical

    candidates.push({
      side: "east",
      position: { x: srcRight, y },
      angle: 0,
      score
    });
  }

  // West side (left)
  for (let i = 0; i < horizontalPorts; i++) {
    const ratio = (i + 1) / horizontalDenom;
    const y = srcY + srcHeight * ratio;
    let score = 0;
    if (dx < 0) score = absDx * 2;
    else if (absDy > absDx) score = absDx * 0.8;

    candidates.push({
      side: "west",
      position: { x: srcX, y },
      angle: 180,
      score
    });
  }

  // South side (bottom)
  for (let i = 0; i < verticalPorts; i++) {
    const ratio = (i + 1) / verticalDenom;
    const x = srcX + srcWidth * ratio;
    let score = 0;
    if (dy > 0) score = absDy * 2;
    else if (absDx > absDy) score = absDy * 0.8; // Allow bottom port even if horizontal

    candidates.push({
      side: "south",
      position: { x, y: srcBottom },
      angle: 90,
      score
    });
  }

  // North side (top)
  for (let i = 0; i < verticalPorts; i++) {
    const ratio = (i + 1) / verticalDenom;
    const x = srcX + srcWidth * ratio;
    let score = 0;
    if (dy < 0) score = absDy * 2;
    else if (absDx > absDy) score = absDy * 0.8;

    candidates.push({
      side: "north",
      position: { x, y: srcY },
      angle: 270,
      score
    });
  }

  // Check clearance for each port (short ray cast outward)
  const CLEARANCE_DISTANCE = 20;
  for (const candidate of candidates) {
    let rayEnd: Point;
    if (candidate.side === "east") {
      rayEnd = { x: candidate.position.x + CLEARANCE_DISTANCE, y: candidate.position.y };
    } else if (candidate.side === "west") {
      rayEnd = { x: candidate.position.x - CLEARANCE_DISTANCE, y: candidate.position.y };
    } else if (candidate.side === "south") {
      rayEnd = { x: candidate.position.x, y: candidate.position.y + CLEARANCE_DISTANCE };
    } else {
      rayEnd = { x: candidate.position.x, y: candidate.position.y - CLEARANCE_DISTANCE };
    }

    // Check if ray intersects any obstacle
    let hasClearance = true;
    for (const obstacle of obstacles) {
      if (segmentIntersectsRect(candidate.position, rayEnd, obstacle)) {
        hasClearance = false;
        break;
      }
    }

    // Boost score if port has clearance (very important for avoiding obstacles)
    if (hasClearance) {
      candidate.score += 150; // Increased from 100 to prioritize ports with clearance
    } else {
      // Heavily penalize ports blocked by obstacles, but don't eliminate them completely
      // Sometimes we need to use a blocked port if all others are worse
      candidate.score -= 80;
    }

    // Penalize ports that are already heavily used (distribute edges across all ports)
    // Track by actual position, not just side, to spread edges along each side
    if (usedPorts) {
      // Use precise position for port key to differentiate ports on same side
      // Optimized: cache source coordinates and rounded position
      const portKey = `${srcX},${srcY}:${candidate.side}:${Math.round(candidate.position.x)},${Math.round(candidate.position.y)}`;
      const usageCount = usedPorts.get(portKey) || 0;
      // More aggressive penalty to better distribute edges across all ports
      // For dense graphs, this helps reduce crossings significantly
      // Optimized: cache obstacles length
      const obstaclesLength = obstacles.length;
      const penaltyMultiplier = obstaclesLength > 10 ? 80 : 60;
      candidate.score -= usageCount * penaltyMultiplier;
    }

    // Additional scoring: prefer sides that point away from obstacles
    // This helps route edges around obstacles more effectively
    let obstacleDistance = Infinity;
    for (const obstacle of obstacles) {
      const dist = distanceFromPortToObstacle(candidate.position, candidate.side, obstacle);
      obstacleDistance = Math.min(obstacleDistance, dist);
    }
    if (obstacleDistance < 50 && obstacleDistance > 0) {
      // Boost score if port is reasonably far from obstacles
      candidate.score += Math.min(30, obstacleDistance / 2);
    }
  }

  // Sort by score (highest first)
  candidates.sort((a, b) => b.score - a.score);

  // Return best candidate
  const best = candidates[0];

  // Track usage if provided (use precise position to differentiate ports on same side)
  if (usedPorts) {
    const portKey = `${source.x},${source.y}:${best.side}:${Math.round(best.position.x)},${Math.round(best.position.y)}`;
    usedPorts.set(portKey, (usedPorts.get(portKey) || 0) + 1);
  }

  // For dense graphs or when we have many obstacles, consider top 2-3 candidates
  // and pick the one that balances direction, clearance, and usage distribution
  if (obstacles.length > 5 && candidates.length >= 2) {
    // Re-score top candidates considering all factors more holistically
    const topCandidates = candidates.slice(0, Math.min(3, candidates.length));
    for (const candidate of topCandidates) {
      // Give bonus to candidates that are less used (better distribution)
      if (usedPorts) {
        const portKey = `${source.x},${source.y}:${candidate.side}:${Math.round(candidate.position.x)},${Math.round(candidate.position.y)}`;
        const usageCount = usedPorts.get(portKey) || 0;
        if (usageCount === 0) {
          candidate.score += 40; // Bonus for unused ports
        }
      }
    }
    // Re-sort with updated scores
    topCandidates.sort((a, b) => b.score - a.score);
    const best = topCandidates[0];

    // Track usage if provided (use precise position)
    if (usedPorts) {
      const portKey = `${source.x},${source.y}:${best.side}:${Math.round(best.position.x)},${Math.round(best.position.y)}`;
      usedPorts.set(portKey, (usedPorts.get(portKey) || 0) + 1);
    }

    return { side: best.side, position: best.position, angle: best.angle };
  }

  // Track usage if provided (use precise position)
  if (usedPorts) {
    const portKey = `${source.x},${source.y}:${best.side}:${Math.round(best.position.x)},${Math.round(best.position.y)}`;
    usedPorts.set(portKey, (usedPorts.get(portKey) || 0) + 1);
  }

  return { side: best.side, position: best.position, angle: best.angle };
}

/**
 * Calculate distance from a port position to an obstacle
 */
function distanceFromPortToObstacle(
  portPos: Point,
  side: "north" | "south" | "east" | "west",
  obstacle: Rect
): number {
  let checkPoint: Point;

  // Project port outward along its side direction
  const PROJECTION_DISTANCE = 30;
  if (side === "east") {
    checkPoint = { x: portPos.x + PROJECTION_DISTANCE, y: portPos.y };
  } else if (side === "west") {
    checkPoint = { x: portPos.x - PROJECTION_DISTANCE, y: portPos.y };
  } else if (side === "south") {
    checkPoint = { x: portPos.x, y: portPos.y + PROJECTION_DISTANCE };
  } else {
    checkPoint = { x: portPos.x, y: portPos.y - PROJECTION_DISTANCE };
  }

  // Calculate distance to obstacle
  const closestX = Math.max(obstacle.x, Math.min(checkPoint.x, obstacle.x + obstacle.width));
  const closestY = Math.max(obstacle.y, Math.min(checkPoint.y, obstacle.y + obstacle.height));

  return Math.hypot(checkPoint.x - closestX, checkPoint.y - closestY);
}

export function calculateBestPort(
  source: Rect,
  target: Rect
): { side: "north" | "south" | "east" | "west"; position: Point; angle: number } {
  const sc = { x: source.x + source.width / 2, y: source.y + source.height / 2 };
  const tc = { x: target.x + target.width / 2, y: target.y + target.height / 2 };

  const srcRight = source.x + source.width;
  const srcBottom = source.y + source.height;

  // Calculate relative position
  const dx = tc.x - sc.x;
  const dy = tc.y - sc.y;
  const absDx = Math.abs(dx);
  const absDy = Math.abs(dy);

  // Score each side based on:
  // 1. Direction toward target (primary factor)
  // 2. Distance to target (secondary factor)
  // 3. Clearance from target (tertiary factor)

  interface SideScore {
    side: "north" | "south" | "east" | "west";
    score: number;
  }

  const sides: SideScore[] = [
    {
      side: "east",
      score: dx > 0 ? absDx * 2 : 0, // Strong preference if target is to the right
    },
    {
      side: "west",
      score: dx < 0 ? absDx * 2 : 0, // Strong preference if target is to the left
    },
    {
      side: "south",
      score: dy > 0 ? absDy * 2 : 0, // Strong preference if target is below
    },
    {
      side: "north",
      score: dy < 0 ? absDy * 2 : 0, // Strong preference if target is above
    },
  ];

  // For diagonal cases, prefer the side with stronger directional component
  // But allow any side if it provides a good path
  if (absDx > 0 && absDy > 0) {
    const primaryIsHorizontal = absDx >= absDy;
    if (primaryIsHorizontal) {
      sides.find((s) => s.side === (dx > 0 ? "east" : "west"))!.score += absDx;
    } else {
      sides.find((s) => s.side === (dy > 0 ? "south" : "north"))!.score += absDy;
    }
  }

  // Sort by score (highest first)
  sides.sort((a, b) => b.score - a.score);

  // Select the best side (always pick one, even if scores are low)
  const bestSide = sides[0].side;

  // Calculate sliding position on the chosen side
  // Position should be as close as possible to the target while staying on the side
  let pos: Point;
  let angle: number;

  if (bestSide === "east") {
    // Connect from right side, align vertically with target center
    const py = Math.max(source.y, Math.min(srcBottom, tc.y));
    pos = { x: srcRight, y: py };
    angle = 0;
  } else if (bestSide === "west") {
    // Connect from left side, align vertically with target center
    const py = Math.max(source.y, Math.min(srcBottom, tc.y));
    pos = { x: source.x, y: py };
    angle = 180;
  } else if (bestSide === "south") {
    // Connect from bottom side, align horizontally with target center
    const px = Math.max(source.x, Math.min(srcRight, tc.x));
    pos = { x: px, y: srcBottom };
    angle = 90;
  } else {
    // north - Connect from top side, align horizontally with target center
    const px = Math.max(source.x, Math.min(srcRight, tc.x));
    pos = { x: px, y: source.y };
    angle = 270;
  }

  return { side: bestSide, position: pos, angle };
}

export function routeOrthogonal(
  source: { position: Point; side: string },
  target: { position: Point; side: string }
): Point[] {
  const points: Point[] = [source.position];
  const sx = source.position.x,
    sy = source.position.y;
  const tx = target.position.x,
    ty = target.position.y;

  // For bidirectional edges (same source/target), offset paths to avoid crossings
  // Check if we need to offset (this will be handled by the routing algorithm)

  if (source.side === "south" || source.side === "north") {
    const midY = (sy + ty) / 2;
    if (sx !== tx) {
      points.push({ x: sx, y: midY });
      points.push({ x: tx, y: midY });
    }
  } else {
    const midX = (sx + tx) / 2;
    if (sy !== ty) {
      points.push({ x: midX, y: sy });
      points.push({ x: midX, y: ty });
    }
  }
  points.push(target.position);
  return points;
}

// Default padding increased from 6 to 20 for better visual clearance around nodes
export function routeOrthogonalAvoid(
  source: { position: Point; side: string },
  target: { position: Point; side: string },
  obstacles: Rect[],
  padding = 20,
  maxIterations = 50
): Point[] {
  // Expand obstacles by a visual clearance buffer to prevent edges from appearing too close
  // Increased clearance for better edge distribution and reduced crossings, especially for expanded nodes
  const VISUAL_CLEARANCE = obstacles.length > 10 ? 18 : 15; // More clearance for dense/expanded graphs
  const expandedObstacles = obstacles.map((r) => ({
    x: r.x - VISUAL_CLEARANCE,
    y: r.y - VISUAL_CLEARANCE,
    width: r.width + VISUAL_CLEARANCE * 2,
    height: r.height + VISUAL_CLEARANCE * 2,
  }));

  let path = routeOrthogonal(source, target);
  let iterations = 0;

  // Track which obstacles we've detoured around to vary detour distances
  const detouredObstacles = new Map<Rect, number>();

  for (let i = 1; i < path.length && iterations < maxIterations; i++) {
    const a = path[i - 1];
    const b = path[i];
    const hit = firstObstacleHit(a, b, expandedObstacles);
    if (hit) {
      // Use original obstacle for detour calculation, but with increased padding
      const originalObstacle =
        obstacles.find(
          (o) =>
            hit.x >= o.x - VISUAL_CLEARANCE - 1 &&
            hit.y >= o.y - VISUAL_CLEARANCE - 1 &&
            hit.x <= o.x + 1 &&
            hit.y <= o.y + 1
        ) || hit;

      // Vary padding based on how many times we've detoured around this obstacle
      // This spreads edges out to reduce congestion and crossings
      const detourCount = detouredObstacles.get(originalObstacle) || 0;
      // Increase padding more aggressively for subsequent detours to reduce crossings
      const variedPadding = padding + detourCount * (obstacles.length > 10 ? 25 : 20);
      detouredObstacles.set(originalObstacle, detourCount + 1);

      const detour = detourAround(a, b, originalObstacle, variedPadding, expandedObstacles);
      path = [...path.slice(0, i), ...detour, ...path.slice(i)];
      i += detour.length;
      iterations++;
    }
  }
  return path;
}

function firstObstacleHit(a: Point, b: Point, obstacles: Rect[]): Rect | undefined {
  for (const r of obstacles) {
    if (segmentIntersectsRect(a, b, r)) return r;
  }
  return undefined;
}

function segmentIntersectsRect(a: Point, b: Point, r: Rect): boolean {
  const withinX = (x: number) => x >= r.x && x <= r.x + r.width;
  const withinY = (y: number) => y >= r.y && y <= r.y + r.height;
  if (a.x === b.x) {
    const x = a.x;
    const minY = Math.min(a.y, b.y),
      maxY = Math.max(a.y, b.y);
    return withinX(x) && !(maxY < r.y || minY > r.y + r.height);
  } else if (a.y === b.y) {
    const y = a.y;
    const minX = Math.min(a.x, b.x),
      maxX = Math.max(a.x, b.x);
    return withinY(y) && !(maxX < r.x || minX > r.x + r.width);
  }
  return false;
}

/**
 * Improved detour selection that considers both path length and obstacle clearance.
 * Prefers routes that maximize distance from all obstacles, not just the current one.
 */
function detourAround(
  a: Point,
  b: Point,
  r: Rect,
  pad: number,
  allObstacles: Rect[] = []
): Point[] {
  // Try multiple detour strategies with varying distances to spread edges
  const strategies = [
    { offset: pad, name: "close" },
    { offset: pad * 1.5, name: "medium" },
    { offset: pad * 2, name: "far" },
  ];

  let best: Point[] | null = null;
  let bestScore = Infinity;

  for (const strategy of strategies) {
    const offset = strategy.offset;
    const left = { x: r.x - offset, y: a.y };
    const right = { x: r.x + r.width + offset, y: a.y };
    const top = { x: a.x, y: r.y - offset };
    const bottom = { x: a.x, y: r.y + r.height + offset };
    const candidates: Point[][] = [
      [left, { x: left.x, y: b.y }],
      [right, { x: right.x, y: b.y }],
      [top, { x: b.x, y: top.y }],
      [bottom, { x: b.x, y: bottom.y }],
    ];

    for (const candidate of candidates) {
      const fullPath = [a, ...candidate, b];
      const len = pathLength(fullPath);

      // Check if this detour passes through any other obstacle
      let crossesOther = false;
      for (let i = 1; i < fullPath.length && !crossesOther; i++) {
        for (const obs of allObstacles) {
          if (obs !== r && segmentIntersectsRect(fullPath[i - 1], fullPath[i], obs)) {
            crossesOther = true;
            break;
          }
        }
      }

      // Score: path length + heavy penalty for crossing obstacles + preference for medium distance (spreads edges)
      const distanceBonus = strategy.name === "medium" ? -10 : 0; // Prefer medium distance to spread edges
      const score = len + (crossesOther ? 10000 : 0) - distanceBonus;
      if (score < bestScore) {
        best = candidate;
        bestScore = score;
      }
    }
  }

  return best || [a, b];
}

export function pathLength(points: Point[]): number {
  let len = 0;
  for (let i = 1; i < points.length; i++) {
    len += Math.hypot(points[i].x - points[i - 1].x, points[i].y - points[i - 1].y);
  }
  return len;
}

export function pathMidpoint(points: Point[]): Point {
  const half = pathLength(points) / 2;
  let acc = 0;
  for (let i = 1; i < points.length; i++) {
    const seg = Math.hypot(points[i].x - points[i - 1].x, points[i].y - points[i - 1].y);
    if (acc + seg >= half) {
      const t = (half - acc) / seg;
      return {
        x: points[i - 1].x + (points[i].x - points[i - 1].x) * t,
        y: points[i - 1].y + (points[i].y - points[i - 1].y) * t,
      };
    }
    acc += seg;
  }
  return points[Math.floor(points.length / 2)];
}

export function arrowAngle(p1: Point, p2: Point): number {
  return (Math.atan2(p2.y - p1.y, p2.x - p1.x) * 180) / Math.PI;
}

export function routeSpline(
  source: { position: Point; side: string },
  target: { position: Point; side: string }
) {
  const p0 = source.position;
  const p3 = target.position;
  const dx = p3.x - p0.x;
  const dy = p3.y - p0.y;
  const k = 0.3;
  let c1: Point;
  let c2: Point;
  if (source.side === "east" || source.side === "west") {
    c1 = { x: p0.x + dx * k, y: p0.y };
    c2 = { x: p3.x - dx * k, y: p3.y };
  } else {
    c1 = { x: p0.x, y: p0.y + dy * k };
    c2 = { x: p3.x, y: p3.y - dy * k };
  }
  return { points: [p0, p3], controlPoints: [c1, c2] };
}

// ============================================================================
// SMOOTH CORNER ROUNDING
// ============================================================================

export type CornerStyle = "sharp" | "rounded" | "smooth";

export interface RoundingOptions {
  /** Corner radius in pixels. Default: 8 */
  cornerRadius?: number;
  /** Corner style. Default: 'rounded' */
  style?: CornerStyle;
  /** Minimum segment length to apply rounding. Default: 20 */
  minSegmentLength?: number;
}

export interface RoundedPathResult {
  /** The path points (may be modified for smooth corners) */
  points: Point[];
  /** Quadratic bezier control points for each corner */
  cornerControlPoints: Array<{ corner: Point; control: Point }>;
  /** Segment types for rendering */
  segmentTypes: Array<"line" | "arc">;
}

/**
 * Apply smooth corner rounding to an orthogonal path.
 * Converts sharp 90° bends to smooth quadratic bezier curves.
 *
 * For each corner, we:
 * 1. Move the points before/after the corner inward by `radius`
 * 2. Create a control point at the original corner position
 * 3. The renderer draws a quadratic bezier through these points
 */
export function applyCornerRounding(
  points: Point[],
  options: RoundingOptions = {}
): RoundedPathResult {
  const { cornerRadius = 8, style = "rounded", minSegmentLength = 20 } = options;

  // Sharp style means no rounding
  if (style === "sharp" || points.length < 3 || cornerRadius <= 0) {
    return {
      points: [...points],
      cornerControlPoints: [],
      segmentTypes: points.slice(1).map(() => "line" as const),
    };
  }

  const result: Point[] = [points[0]];
  const cornerControlPoints: Array<{ corner: Point; control: Point }> = [];
  const segmentTypes: Array<"line" | "arc"> = [];

  for (let i = 1; i < points.length - 1; i++) {
    const prev = points[i - 1];
    const curr = points[i]; // The corner
    const next = points[i + 1];

    // Calculate segment lengths
    const prevSegLen = Math.hypot(curr.x - prev.x, curr.y - prev.y);
    const nextSegLen = Math.hypot(next.x - curr.x, next.y - curr.y);

    // Adjust radius to fit within available segment length
    // Use half the shorter segment as max radius
    const maxRadius = Math.min(prevSegLen, nextSegLen) / 2;
    const radius = Math.min(cornerRadius, maxRadius);

    // Skip rounding for very short segments
    if (prevSegLen < minSegmentLength || nextSegLen < minSegmentLength || radius < 2) {
      result.push(curr);
      segmentTypes.push("line");
      continue;
    }

    // Calculate direction vectors
    const prevDir = {
      x: (curr.x - prev.x) / prevSegLen,
      y: (curr.y - prev.y) / prevSegLen,
    };
    const nextDir = {
      x: (next.x - curr.x) / nextSegLen,
      y: (next.y - curr.y) / nextSegLen,
    };

    // Point before the corner (pulled back by radius)
    const beforeCorner: Point = {
      x: curr.x - prevDir.x * radius,
      y: curr.y - prevDir.y * radius,
    };

    // Point after the corner (moved forward by radius)
    const afterCorner: Point = {
      x: curr.x + nextDir.x * radius,
      y: curr.y + nextDir.y * radius,
    };

    // Add the point before the corner
    result.push(beforeCorner);
    segmentTypes.push("line");

    // Store the control point info for bezier rendering
    cornerControlPoints.push({
      corner: curr, // Original corner (becomes control point)
      control: curr, // For quadratic bezier, control is at original corner
    });

    // For 'smooth' style, use a gentler curve
    if (style === "smooth") {
      // Add an intermediate point for smoother curve
      const midControl: Point = {
        x: curr.x,
        y: curr.y,
      };
      cornerControlPoints[cornerControlPoints.length - 1].control = midControl;
    }

    // Add the point after the corner
    result.push(afterCorner);
    segmentTypes.push("arc");
  }

  // Add the final point
  result.push(points[points.length - 1]);
  segmentTypes.push("line");

  return {
    points: result,
    cornerControlPoints,
    segmentTypes,
  };
}

/**
 * Convert corner control points to SVG path data.
 * Useful for rendering in browsers or SVG export.
 */
export function cornerRoundingToSvgPath(roundedPath: RoundedPathResult): string {
  const { points, cornerControlPoints, segmentTypes } = roundedPath;

  if (points.length === 0) return "";

  let path = `M ${points[0].x} ${points[0].y}`;
  let cornerIndex = 0;

  for (let i = 1; i < points.length; i++) {
    const segType = segmentTypes[i - 1];

    if (segType === "arc" && cornerIndex < cornerControlPoints.length) {
      // Quadratic bezier curve through the corner
      const cc = cornerControlPoints[cornerIndex];
      path += ` Q ${cc.control.x} ${cc.control.y} ${points[i].x} ${points[i].y}`;
      cornerIndex++;
    } else {
      // Straight line
      path += ` L ${points[i].x} ${points[i].y}`;
    }
  }

  return path;
}

/**
 * Calculate the approximate length of a rounded path
 * (useful for label placement)
 */
export function roundedPathLength(roundedPath: RoundedPathResult): number {
  const { points, segmentTypes, cornerControlPoints } = roundedPath;
  let len = 0;
  let cornerIndex = 0;

  for (let i = 1; i < points.length; i++) {
    const p1 = points[i - 1];
    const p2 = points[i];
    const segType = segmentTypes[i - 1];

    if (segType === "arc" && cornerIndex < cornerControlPoints.length) {
      // Approximate arc length using control point
      const cc = cornerControlPoints[cornerIndex];
      // Use chord + 1/4 of difference to control point as approximation
      const chord = Math.hypot(p2.x - p1.x, p2.y - p1.y);
      const toControl =
        Math.hypot(cc.control.x - p1.x, cc.control.y - p1.y) +
        Math.hypot(p2.x - cc.control.x, p2.y - cc.control.y);
      len += chord + (toControl - chord) * 0.25;
      cornerIndex++;
    } else {
      len += Math.hypot(p2.x - p1.x, p2.y - p1.y);
    }
  }

  return len;
}
