/**
 * Clutter Detection Algorithm
 * 
 * Computes a clutter score based on:
 * - Node proximity (nodes too close together)
 * - Edge crossings
 * - Edge routing complexity (bends, length)
 * - Layout density
 */

import type { Point } from '../geometry/point'
import type { Rect } from '../geometry/rect'
import { CLUTTER_THRESHOLD, NO_ENTRY_MARGIN } from '../constants'

export interface ClutterScore {
    /** 0-1: How close nodes are to each other (0 = well spaced, 1 = overlapping) */
    nodeProximity: number
    /** Number of edge crossings normalized by total edges */
    edgeCrossings: number
    /** Average bends per edge normalized */
    routingComplexity: number
    /** Ratio of used space to total bounding box (higher = denser) */
    density: number
    /** Combined weighted score 0-1 */
    total: number
    /** Whether the score exceeds the optimization threshold */
    needsOptimization: boolean
}

export interface NodeBox {
    id: string
    bbox: Rect
}

export interface EdgePath {
    id: string
    points: readonly Point[]
}

/**
 * Calculate the minimum distance between two rectangles
 */
function rectDistance(a: Rect, b: Rect): number {
    const left = b.x + b.width < a.x
    const right = a.x + a.width < b.x
    const top = b.y + b.height < a.y
    const bottom = a.y + a.height < b.y

    if (left && top) {
        return Math.hypot(a.x - (b.x + b.width), a.y - (b.y + b.height))
    }
    if (left && bottom) {
        return Math.hypot(a.x - (b.x + b.width), b.y - (a.y + a.height))
    }
    if (right && top) {
        return Math.hypot(b.x - (a.x + a.width), a.y - (b.y + b.height))
    }
    if (right && bottom) {
        return Math.hypot(b.x - (a.x + a.width), b.y - (a.y + a.height))
    }
    if (left) return a.x - (b.x + b.width)
    if (right) return b.x - (a.x + a.width)
    if (top) return a.y - (b.y + b.height)
    if (bottom) return b.y - (a.y + a.height)

    // Overlapping
    return 0
}

/**
 * Check if two line segments intersect
 */
function segmentsIntersect(a1: Point, a2: Point, b1: Point, b2: Point): boolean {
    const ccw = (p1: Point, p2: Point, p3: Point) =>
        (p3.y - p1.y) * (p2.x - p1.x) > (p2.y - p1.y) * (p3.x - p1.x)

    return (
        ccw(a1, b1, b2) !== ccw(a2, b1, b2) &&
        ccw(a1, a2, b1) !== ccw(a1, a2, b2)
    )
}

/**
 * Count intersections between two edge paths
 */
function countEdgeCrossings(edges: EdgePath[]): number {
    let crossings = 0

    for (let i = 0; i < edges.length; i++) {
        for (let j = i + 1; j < edges.length; j++) {
            const pathA = edges[i].points
            const pathB = edges[j].points

            for (let a = 0; a < pathA.length - 1; a++) {
                for (let b = 0; b < pathB.length - 1; b++) {
                    if (segmentsIntersect(pathA[a], pathA[a + 1], pathB[b], pathB[b + 1])) {
                        crossings++
                    }
                }
            }
        }
    }

    return crossings
}

/**
 * Calculate bounding box of all nodes
 */
function calculateBoundingBox(nodes: NodeBox[]): Rect {
    if (nodes.length === 0) {
        return { x: 0, y: 0, width: 0, height: 0 }
    }

    let minX = Infinity, minY = Infinity
    let maxX = -Infinity, maxY = -Infinity

    for (const node of nodes) {
        minX = Math.min(minX, node.bbox.x)
        minY = Math.min(minY, node.bbox.y)
        maxX = Math.max(maxX, node.bbox.x + node.bbox.width)
        maxY = Math.max(maxY, node.bbox.y + node.bbox.height)
    }

    return { x: minX, y: minY, width: maxX - minX, height: maxY - minY }
}

/**
 * Compute the clutter score for a layout
 */
export function computeClutterScore(
    nodes: NodeBox[],
    edges: EdgePath[],
    options: {
        idealNodeSpacing?: number
        maxEdgeCrossings?: number
        threshold?: number
    } = {}
): ClutterScore {
    const {
        idealNodeSpacing = NO_ENTRY_MARGIN * 4,
        maxEdgeCrossings = Math.max(10, edges.length * 2),
        threshold = CLUTTER_THRESHOLD
    } = options

    // 1. Node proximity score
    let proximitySum = 0
    let proximityCount = 0

    for (let i = 0; i < nodes.length; i++) {
        for (let j = i + 1; j < nodes.length; j++) {
            const dist = rectDistance(nodes[i].bbox, nodes[j].bbox)
            // Score: 1 if overlapping/touching, 0 if >= idealSpacing
            const score = dist < idealNodeSpacing
                ? 1 - (dist / idealNodeSpacing)
                : 0
            proximitySum += score
            proximityCount++
        }
    }

    const nodeProximity = proximityCount > 0
        ? Math.min(1, proximitySum / proximityCount)
        : 0

    // 2. Edge crossings score
    const crossings = countEdgeCrossings(edges)
    const edgeCrossings = Math.min(1, crossings / maxEdgeCrossings)

    // 3. Routing complexity score (average bends per edge)
    let totalBends = 0
    for (const edge of edges) {
        totalBends += Math.max(0, edge.points.length - 2)
    }
    const avgBends = edges.length > 0 ? totalBends / edges.length : 0
    const routingComplexity = Math.min(1, avgBends / 4) // 4+ bends = max complexity

    // 4. Density score
    const bbox = calculateBoundingBox(nodes)
    const totalArea = bbox.width * bbox.height
    const usedArea = nodes.reduce((sum, n) => sum + n.bbox.width * n.bbox.height, 0)
    const density = totalArea > 0
        ? Math.min(1, usedArea / totalArea)
        : 0

    // Weighted total (weights sum to 1)
    const total =
        nodeProximity * 0.35 +
        edgeCrossings * 0.30 +
        routingComplexity * 0.20 +
        density * 0.15

    return {
        nodeProximity,
        edgeCrossings,
        routingComplexity,
        density,
        total,
        needsOptimization: total > threshold
    }
}

/**
 * Quick check if layout needs optimization
 */
export function shouldReoptimize(score: ClutterScore, threshold = CLUTTER_THRESHOLD): boolean {
    return score.total > threshold
}

/**
 * Get optimization suggestions based on clutter score
 */
export function getOptimizationSuggestions(score: ClutterScore): string[] {
    const suggestions: string[] = []

    if (score.nodeProximity > 0.5) {
        suggestions.push('Increase node spacing or expand grid')
    }
    if (score.edgeCrossings > 0.3) {
        suggestions.push('Reorder nodes to reduce edge crossings')
    }
    if (score.routingComplexity > 0.5) {
        suggestions.push('Consider curved edges or repositioning nodes')
    }
    if (score.density > 0.6) {
        suggestions.push('Expand layout boundaries for more breathing room')
    }

    return suggestions
}
