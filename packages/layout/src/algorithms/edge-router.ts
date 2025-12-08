import type { Point } from '../geometry/point'
import type { Rect } from '../geometry/rect'

export function calculateBestPort(source: Rect, target: Rect): { side: 'north' | 'south' | 'east' | 'west'; position: Point; angle: number } {
  const sc = { x: source.x + source.width / 2, y: source.y + source.height / 2 }
  const tc = { x: target.x + target.width / 2, y: target.y + target.height / 2 }
  const dx = tc.x - sc.x
  const dy = tc.y - sc.y
  if (Math.abs(dx) > Math.abs(dy)) {
    return dx > 0 ? { side: 'east', position: { x: source.x + source.width, y: sc.y }, angle: 0 } : { side: 'west', position: { x: source.x, y: sc.y }, angle: 180 }
  } else {
    return dy > 0 ? { side: 'south', position: { x: sc.x, y: source.y + source.height }, angle: 90 } : { side: 'north', position: { x: sc.x, y: source.y }, angle: 270 }
  }
}

export function routeOrthogonal(source: { position: Point; side: string }, target: { position: Point; side: string }): Point[] {
  const points: Point[] = [source.position]
  const sx = source.position.x, sy = source.position.y
  const tx = target.position.x, ty = target.position.y
  if (source.side === 'south' || source.side === 'north') {
    const midY = (sy + ty) / 2
    if (sx !== tx) { points.push({ x: sx, y: midY }); points.push({ x: tx, y: midY }) }
  } else {
    const midX = (sx + tx) / 2
    if (sy !== ty) { points.push({ x: midX, y: sy }); points.push({ x: midX, y: ty }) }
  }
  points.push(target.position)
  return points
}

export function routeOrthogonalAvoid(source: { position: Point; side: string }, target: { position: Point; side: string }, obstacles: Rect[], maxIterations = 50): Point[] {
  let path = routeOrthogonal(source, target)
  let iterations = 0
  for (let i = 1; i < path.length && iterations < maxIterations; i++) {
    const a = path[i - 1]
    const b = path[i]
    const hit = firstObstacleHit(a, b, obstacles)
    if (hit) {
      const detour = detourAround(a, b, hit)
      path = [...path.slice(0, i), ...detour, ...path.slice(i)]
      i += detour.length
      iterations++
    }
  }
  return path
}

function firstObstacleHit(a: Point, b: Point, obstacles: Rect[]): Rect | undefined {
  for (const r of obstacles) {
    if (segmentIntersectsRect(a, b, r)) return r
  }
  return undefined
}

function segmentIntersectsRect(a: Point, b: Point, r: Rect): boolean {
  const withinX = (x: number) => x >= r.x && x <= r.x + r.width
  const withinY = (y: number) => y >= r.y && y <= r.y + r.height
  if (a.x === b.x) {
    const x = a.x
    const minY = Math.min(a.y, b.y), maxY = Math.max(a.y, b.y)
    return withinX(x) && !(maxY < r.y || minY > r.y + r.height)
  } else if (a.y === b.y) {
    const y = a.y
    const minX = Math.min(a.x, b.x), maxX = Math.max(a.x, b.x)
    return withinY(y) && !(maxX < r.x || minX > r.x + r.width)
  }
  return false
}

function detourAround(a: Point, b: Point, r: Rect): Point[] {
  const pad = 6
  const left = { x: r.x - pad, y: a.y }
  const right = { x: r.x + r.width + pad, y: a.y }
  const top = { x: a.x, y: r.y - pad }
  const bottom = { x: a.x, y: r.y + r.height + pad }
  const candidates: Point[][] = [
    [left, { x: left.x, y: b.y }],
    [right, { x: right.x, y: b.y }],
    [top, { x: b.x, y: top.y }],
    [bottom, { x: b.x, y: bottom.y }]
  ]
  let best: Point[] = candidates[0]
  let bestLen = pathLength([a, ...best, b])
  for (const c of candidates.slice(1)) {
    const len = pathLength([a, ...c, b])
    if (len < bestLen) { best = c; bestLen = len }
  }
  return best
}

export function pathLength(points: Point[]): number {
  let len = 0
  for (let i = 1; i < points.length; i++) {
    len += Math.hypot(points[i].x - points[i - 1].x, points[i].y - points[i - 1].y)
  }
  return len
}

export function pathMidpoint(points: Point[]): Point {
  const half = pathLength(points) / 2
  let acc = 0
  for (let i = 1; i < points.length; i++) {
    const seg = Math.hypot(points[i].x - points[i - 1].x, points[i].y - points[i - 1].y)
    if (acc + seg >= half) {
      const t = (half - acc) / seg
      return { x: points[i - 1].x + (points[i].x - points[i - 1].x) * t, y: points[i - 1].y + (points[i].y - points[i - 1].y) * t }
    }
    acc += seg
  }
  return points[Math.floor(points.length / 2)]
}

export function arrowAngle(p1: Point, p2: Point): number {
  return (Math.atan2(p2.y - p1.y, p2.x - p1.x) * 180) / Math.PI
}

export function routeSpline(source: { position: Point; side: string }, target: { position: Point; side: string }) {
  const p0 = source.position
  const p3 = target.position
  const dx = p3.x - p0.x
  const dy = p3.y - p0.y
  const k = 0.3
  let c1: Point
  let c2: Point
  if (source.side === 'east' || source.side === 'west') {
    c1 = { x: p0.x + dx * k, y: p0.y }
    c2 = { x: p3.x - dx * k, y: p3.y }
  } else {
    c1 = { x: p0.x, y: p0.y + dy * k }
    c2 = { x: p3.x, y: p3.y - dy * k }
  }
  return { points: [p0, p3], controlPoints: [c1, c2] }
}
