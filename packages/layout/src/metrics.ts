import type { Point } from './geometry/point'

export function calculateMetrics(nodes: Map<string, { x: number; y: number; size: { width: number; height: number } }>, edges: { points: Point[]; bendCount: number; length: number }[]) {
  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity
  for (const n of nodes.values()) {
    minX = Math.min(minX, n.x)
    minY = Math.min(minY, n.y)
    maxX = Math.max(maxX, n.x + n.size.width)
    maxY = Math.max(maxY, n.y + n.size.height)
  }
  const bbox = { x: minX, y: minY, width: maxX - minX, height: maxY - minY }
  let nodeArea = 0
  for (const n of nodes.values()) nodeArea += n.size.width * n.size.height
  const coverage = nodeArea / (bbox.width * bbox.height || 1)
  let edgeCrossings = 0
  for (let i = 0; i < edges.length; i++) {
    for (let j = i + 1; j < edges.length; j++) {
      if (polylineCross(edges[i].points, edges[j].points)) edgeCrossings++
    }
  }
  const edgeBends = edges.reduce((s, e) => s + e.bendCount, 0)
  const totalEdgeLength = edges.reduce((s, e) => s + e.length, 0)
  return { aspectRatio: bbox.width / (bbox.height || 1), coverage, edgeCrossings, edgeBends, totalEdgeLength, uniformity: 1, balance: 1, compactness: coverage }
}

function polylineCross(a: Point[], b: Point[]): boolean {
  for (let i = 1; i < a.length; i++) {
    for (let j = 1; j < b.length; j++) {
      if (segmentsIntersect(a[i - 1], a[i], b[j - 1], b[j])) return true
    }
  }
  return false
}

function segmentsIntersect(a1: Point, a2: Point, b1: Point, b2: Point): boolean {
  function ccw(p1: Point, p2: Point, p3: Point) { return (p3.y - p1.y) * (p2.x - p1.x) > (p2.y - p1.y) * (p3.x - p1.x) }
  return ccw(a1, b1, b2) !== ccw(a2, b1, b2) && ccw(a1, a2, b1) !== ccw(a1, a2, b2)
}
