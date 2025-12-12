import type { C4Graph } from './c4-model'
import type { C4ViewState } from './c4-view'
import { buildHierarchy } from './algorithms/hierarchy'
import { calculateSizes } from './algorithms/sizing'
import { assignCoordinates } from './algorithms/coordinates'
import { calculateBestPort, routeOrthogonalAvoid, routeSpline, pathMidpoint, arrowAngle, pathLength } from './algorithms/edge-router'
import { calculateMetrics } from './metrics'
import type { C4LayoutOptions } from './c4-options'
import { InteractivePreset } from './c4-options'
import { beautify } from './beautifier'
import { removeOverlaps } from './algorithms/overlap'
import { applyPostProcessors } from './plugin'
import type { Point } from './geometry/point'
import type { Rect as BBox } from './geometry/rect'
import { bundleEdges } from './algorithms/edge-bundler'
import { placeLabels, type NodeForLabeling } from './algorithms/label-placer'
import { applyMultiPassOptimization } from './algorithms/optimizer'

export interface PositionedC4Node {
  nodeId: string
  bbox: BBox
  contentBox: BBox
  labelBox: BBox
  parentId?: string
  childrenIds: readonly string[]
  depth: number
  level: string
  collapsed: boolean
  visible: boolean
  zIndex: number
  ports: readonly { side: 'north' | 'south' | 'east' | 'west'; position: Point; angle: number }[]
}

export interface PositionedC4Relationship {
  relationshipId: string
  sourceId: string
  targetId: string
  points: readonly Point[]
  controlPoints?: readonly Point[]
  segmentTypes: readonly ('line' | 'arc' | 'orthogonal')[]
  labelPosition: Point
  labelAngle: number
  labelBounds: BBox
  arrowEnd: Point
  arrowAngle: number
  length: number
  bendCount: number
  crossesBoundaries: boolean
}

export interface C4LayoutResult {
  nodes: ReadonlyMap<string, PositionedC4Node>
  relationships: readonly PositionedC4Relationship[]
  bbox: BBox
  center: Point
  metrics: {
    aspectRatio: number
    coverage: number
    edgeCrossings: number
    edgeBends: number
    totalEdgeLength: number
    uniformity: number
    balance: number
    compactness: number
  }
  debug?: { layoutTimeMs: number; phases: { name: string; durationMs: number; nodesProcessed: number }[]; warnings: string[] }
}

export function layout(graph: C4Graph, view: C4ViewState, options: C4LayoutOptions = InteractivePreset): C4LayoutResult {
  const start = Date.now()
  const phases: { name: string; durationMs: number; nodesProcessed: number }[] = []
  const h0 = Date.now()
  const tree = buildHierarchy(graph, view)
  phases.push({ name: 'buildHierarchy', durationMs: Date.now() - h0, nodesProcessed: tree.nodeMap.size })

  const s0 = Date.now()
  const sizes = calculateSizes(tree, graph.relationships.map(r => ({ from: r.from, to: r.to })), options.measurer, options)
  phases.push({ name: 'calculateSizes', durationMs: Date.now() - s0, nodesProcessed: sizes.size })

  const c0 = Date.now()
  const rels = graph.relationships.map(r => ({ from: r.from, to: r.to }))
  let positioned = assignCoordinates(tree, sizes, rels, options)
  phases.push({ name: 'assignCoordinates', durationMs: Date.now() - c0, nodesProcessed: positioned.size })

  // Phase 4: Multi-pass optimization (NEW!)
  // This replaces the old single-pass overlap removal with a comprehensive optimization system
  if (options.optimization?.enabled) {
    const opt0 = Date.now()
    positioned = applyMultiPassOptimization(positioned, [...graph.relationships], tree, options.optimization)
    phases.push({ name: 'optimizeLayout', durationMs: Date.now() - opt0, nodesProcessed: positioned.size })
  } else if (options.overlapRemoval?.enabled) {
    // Fallback to legacy overlap removal for backwards compatibility
    const o0 = Date.now()
    positioned = removeOverlaps(positioned, options.overlapRemoval.padding, options.overlapRemoval.iterations)
    phases.push({ name: 'removeOverlaps', durationMs: Date.now() - o0, nodesProcessed: positioned.size })
  }

  const e0 = Date.now()
  const edges: PositionedC4Relationship[] = []
  for (const rel of graph.relationships) {
    const src = positioned.get(rel.from as any)
    const dst = positioned.get(rel.to as any)
    if (!src || !dst) continue
    const sp = calculateBestPort(src.bbox, dst.bbox)
    const tp = calculateBestPort(dst.bbox, src.bbox)
    const routePref = rel.preferredRoute ?? options.edgeRouting.algorithm
    if (routePref === 'curved' || routePref === 'splines') {
      const { points: p, controlPoints } = routeSpline(sp, tp)
      const mid = pathMidpoint([p[0], p[1]])
      const angle = arrowAngle(p[0], p[1])
      edges.push({ relationshipId: rel.id, sourceId: rel.from as any, targetId: rel.to as any, points: p, controlPoints, segmentTypes: ['arc'], labelPosition: mid, labelAngle: 0, labelBounds: { x: mid.x - 50, y: mid.y - 10, width: 100, height: 20 }, arrowEnd: p[p.length - 1], arrowAngle: angle, length: pathLength(p), bendCount: 0, crossesBoundaries: false })
    } else {
      // Filter obstacles: Exclude source, target, and their ancestors (otherwise routing fails starting inside a parent)
      const getAncestors = (id: string | undefined): Set<string> => {
        const ancestors = new Set<string>()
        let curr = id
        while (curr) {
          ancestors.add(curr)
          const parent = positioned.get(curr as any)?.parent
          curr = parent?.id
        }
        return ancestors
      }

      const srcAncestors = getAncestors(src.parent?.id)
      const dstAncestors = getAncestors(dst.parent?.id)

      const obstacles = [...positioned.values()]
        .filter(n =>
          n.id !== src.id &&
          n.id !== dst.id &&
          !srcAncestors.has(n.id) &&
          !dstAncestors.has(n.id)
        )
        .map(n => n.bbox)

      const routingPadding = options.strategy === 'l1-context' ? 30 : 6
      const pts = routeOrthogonalAvoid(sp, tp, obstacles, routingPadding)
      const mid = pathMidpoint(pts)
      const angle = arrowAngle(pts[pts.length - 2] ?? pts[0], pts[pts.length - 1])
      edges.push({ relationshipId: rel.id, sourceId: rel.from as any, targetId: rel.to as any, points: pts, segmentTypes: pts.slice(1).map(() => 'orthogonal'), labelPosition: mid, labelAngle: 0, labelBounds: { x: mid.x - 50, y: mid.y - 10, width: 100, height: 20 }, arrowEnd: pts[pts.length - 1], arrowAngle: angle, length: pathLength(pts), bendCount: pts.length - 2, crossesBoundaries: false })
    }
  }
  phases.push({ name: 'routeEdges', durationMs: Date.now() - e0, nodesProcessed: edges.length })

  // Edge bundling: group parallel edges and fan out at endpoints
  const bundle0 = Date.now()
  const edgesForBundling = edges.map(e => ({
    id: e.relationshipId,
    sourceId: e.sourceId,
    targetId: e.targetId,
    points: [...e.points] as Point[]
  }))
  const { bundles, adjustedPoints } = bundleEdges(edgesForBundling)

  // Apply bundled points back to edges
  for (const edge of edges) {
    const adjusted = adjustedPoints.get(edge.relationshipId)
    if (adjusted) {
      (edge as any).points = adjusted
    }
  }
  phases.push({ name: 'bundleEdges', durationMs: Date.now() - bundle0, nodesProcessed: bundles.length })

  const nodesOut = new Map<string, PositionedC4Node>()
  for (const [id, n] of positioned) {
    nodesOut.set(id, { nodeId: id, bbox: n.bbox, contentBox: n.bbox, labelBox: n.bbox, parentId: n.parent?.id, childrenIds: n.children.map(c => c.id), depth: n.depth, level: n.node.level, collapsed: !!n.node.collapseChildren, visible: !n.node.hidden, zIndex: 0, ports: [] })
  }

  // Label placement: avoid collisions between edge labels and nodes
  const label0 = Date.now()
  const nodeMap = new Map<string, NodeForLabeling>()
  for (const [id, n] of nodesOut) {
    nodeMap.set(id, { id, bbox: n.bbox })
  }
  const edgesForLabeling = edges.map(e => ({
    id: e.relationshipId,
    label: graph.relationships.find(r => r.id === e.relationshipId)?.label,
    points: e.points
  }))
  const labelPlacements = placeLabels(edgesForLabeling, nodeMap, { padding: 8 })

  // Apply label placements back to edges
  for (const edge of edges) {
    const placement = labelPlacements.get(edge.relationshipId)
    if (placement) {
      (edge as any).labelPosition = placement.position
        ; (edge as any).labelAngle = placement.rotation
        ; (edge as any).labelBounds = placement.bounds
    }
  }
  phases.push({ name: 'placeLabels', durationMs: Date.now() - label0, nodesProcessed: labelPlacements.size })

  const b0 = Date.now()
  beautify(new Map([...positioned]), { alignNodes: options.beautify.alignNodes, gridSize: view.gridSize, snapToGrid: view.snapToGrid })
  applyPostProcessors(nodesOut as any, edges as any)
  phases.push({ name: 'beautify', durationMs: Date.now() - b0, nodesProcessed: positioned.size })

  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity
  for (const n of nodesOut.values()) {
    minX = Math.min(minX, n.bbox.x)
    minY = Math.min(minY, n.bbox.y)
    maxX = Math.max(maxX, n.bbox.x + n.bbox.width)
    maxY = Math.max(maxY, n.bbox.y + n.bbox.height)
  }
  const bbox = { x: minX, y: minY, width: maxX - minX, height: maxY - minY }
  const center = { x: bbox.x + bbox.width / 2, y: bbox.y + bbox.height / 2 }
  const metrics = calculateMetrics(new Map([...nodesOut].map(([id, n]) => [id, { x: n.bbox.x, y: n.bbox.y, size: { width: n.bbox.width, height: n.bbox.height } }])), edges.map(e => ({ ...e, points: [...e.points] })))
  const end = Date.now()
  return { nodes: nodesOut, relationships: edges, bbox, center, metrics, debug: { layoutTimeMs: end - start, phases, warnings: [] } }
}

export async function layoutAsync(graph: C4Graph, view: C4ViewState, options: C4LayoutOptions = InteractivePreset): Promise<C4LayoutResult> {
  return layout(graph, view, options)
}
