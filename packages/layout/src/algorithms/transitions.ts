/**
 * Animated Transitions Engine
 * 
 * Provides smooth interpolation between layout states during drill-down/collapse.
 * Based on L0-Layout.md specification:
 * - Segment normalization for equal-length point arrays
 * - Point-by-point interpolation with easing
 * - Node and viewport transition support
 */

import type { Point } from '../geometry/point'
import type { Rect } from '../geometry/rect'

export interface TransitionState {
    nodes: Map<string, TransitionNode>
    edges: TransitionEdge[]
    viewport: Viewport
}

export interface TransitionNode {
    id: string
    position: Point
    size: { width: number; height: number }
    opacity: number
    scale: number
}

export interface TransitionEdge {
    id: string
    points: Point[]
    opacity: number
    strokeDashoffset?: number
}

export interface Viewport {
    x: number
    y: number
    zoom: number
}

export interface AnimationFrame {
    nodes: Map<string, TransitionNode>
    edges: TransitionEdge[]
    viewport: Viewport
    t: number
}

/**
 * Cubic ease-in-out function for smooth animations
 */
export function ease(t: number): number {
    if (t < 0.5) {
        return 4 * t * t * t
    }
    return 1 - Math.pow(-2 * t + 2, 3) / 2
}

/**
 * Linear interpolation between two numbers
 */
export function lerp(a: number, b: number, t: number): number {
    return a + (b - a) * t
}

/**
 * Interpolate between two points
 */
export function interpolatePoint(a: Point, b: Point, t: number): Point {
    return {
        x: lerp(a.x, b.x, t),
        y: lerp(a.y, b.y, t)
    }
}

/**
 * Interpolate between two rects
 */
export function interpolateRect(a: Rect, b: Rect, t: number): Rect {
    return {
        x: lerp(a.x, b.x, t),
        y: lerp(a.y, b.y, t),
        width: lerp(a.width, b.width, t),
        height: lerp(a.height, b.height, t)
    }
}

/**
 * Normalize two point arrays to have the same length
 * by expanding the shorter one with repeated endpoint
 */
export function normalizePoints(oldPoints: Point[], newPoints: Point[]): {
    oldNormalized: Point[]
    newNormalized: Point[]
} {
    const maxLen = Math.max(oldPoints.length, newPoints.length)

    const oldNormalized = expandPoints(oldPoints, maxLen)
    const newNormalized = expandPoints(newPoints, maxLen)

    return { oldNormalized, newNormalized }
}

/**
 * Expand a point array to target length by repeating points
 */
function expandPoints(points: Point[], targetLength: number): Point[] {
    if (points.length === 0) {
        return Array(targetLength).fill({ x: 0, y: 0 })
    }

    if (points.length >= targetLength) {
        return points.slice(0, targetLength)
    }

    const result: Point[] = [...points]

    // Distribute extra points evenly
    while (result.length < targetLength) {
        // Insert duplicates near the end to preserve shape
        const insertIndex = Math.floor(result.length / 2)
        result.splice(insertIndex, 0, { ...result[insertIndex] })
    }

    return result
}

/**
 * Match edges between old and new states by ID
 */
export function matchEdgesById(
    oldEdges: TransitionEdge[],
    newEdges: TransitionEdge[]
): {
    id: string
    oldPoints: Point[]
    newPoints: Point[]
    oldOpacity: number
    newOpacity: number
    appearing: boolean
    disappearing: boolean
}[] {
    const oldMap = new Map(oldEdges.map(e => [e.id, e]))
    const newMap = new Map(newEdges.map(e => [e.id, e]))

    const allIds = new Set([...oldMap.keys(), ...newMap.keys()])
    const result: {
        id: string
        oldPoints: Point[]
        newPoints: Point[]
        oldOpacity: number
        newOpacity: number
        appearing: boolean
        disappearing: boolean
    }[] = []

    for (const id of allIds) {
        const oldEdge = oldMap.get(id)
        const newEdge = newMap.get(id)

        if (oldEdge && newEdge) {
            // Edge exists in both states
            result.push({
                id,
                oldPoints: oldEdge.points,
                newPoints: newEdge.points,
                oldOpacity: oldEdge.opacity,
                newOpacity: newEdge.opacity,
                appearing: false,
                disappearing: false
            })
        } else if (newEdge) {
            // Edge appearing
            const startPoint = newEdge.points[0] ?? { x: 0, y: 0 }
            result.push({
                id,
                oldPoints: newEdge.points.map(() => startPoint), // Start from first point
                newPoints: newEdge.points,
                oldOpacity: 0,
                newOpacity: newEdge.opacity,
                appearing: true,
                disappearing: false
            })
        } else if (oldEdge) {
            // Edge disappearing
            const endPoint = oldEdge.points[oldEdge.points.length - 1] ?? { x: 0, y: 0 }
            result.push({
                id,
                oldPoints: oldEdge.points,
                newPoints: oldEdge.points.map(() => endPoint), // Collapse to end point
                oldOpacity: oldEdge.opacity,
                newOpacity: 0,
                appearing: false,
                disappearing: true
            })
        }
    }

    return result
}

/**
 * Match nodes between old and new states by ID
 */
export function matchNodesById(
    oldNodes: Map<string, TransitionNode>,
    newNodes: Map<string, TransitionNode>,
    parentPositions?: Map<string, Point>
): {
    id: string
    old: TransitionNode
    new: TransitionNode
    appearing: boolean
    disappearing: boolean
}[] {
    const allIds = new Set([...oldNodes.keys(), ...newNodes.keys()])
    const result: {
        id: string
        old: TransitionNode
        new: TransitionNode
        appearing: boolean
        disappearing: boolean
    }[] = []

    for (const id of allIds) {
        const oldNode = oldNodes.get(id)
        const newNode = newNodes.get(id)

        if (oldNode && newNode) {
            result.push({
                id,
                old: oldNode,
                new: newNode,
                appearing: false,
                disappearing: false
            })
        } else if (newNode) {
            // Node appearing - animate from parent or center
            const startPos = parentPositions?.get(id) ?? newNode.position
            result.push({
                id,
                old: { ...newNode, position: startPos, opacity: 0, scale: 0.8 },
                new: newNode,
                appearing: true,
                disappearing: false
            })
        } else if (oldNode) {
            // Node disappearing
            const endPos = parentPositions?.get(id) ?? oldNode.position
            result.push({
                id,
                old: oldNode,
                new: { ...oldNode, position: endPos, opacity: 0, scale: 0.8 },
                appearing: false,
                disappearing: true
            })
        }
    }

    return result
}

/**
 * Interpolate a node between old and new states
 */
export function interpolateNode(
    oldNode: TransitionNode,
    newNode: TransitionNode,
    t: number
): TransitionNode {
    const eased = ease(t)
    return {
        id: newNode.id,
        position: interpolatePoint(oldNode.position, newNode.position, eased),
        size: {
            width: lerp(oldNode.size.width, newNode.size.width, eased),
            height: lerp(oldNode.size.height, newNode.size.height, eased)
        },
        opacity: lerp(oldNode.opacity, newNode.opacity, eased),
        scale: lerp(oldNode.scale, newNode.scale, eased)
    }
}

/**
 * Interpolate viewport between old and new states
 */
export function interpolateViewport(
    oldViewport: Viewport,
    newViewport: Viewport,
    t: number
): Viewport {
    const eased = ease(t)
    return {
        x: lerp(oldViewport.x, newViewport.x, eased),
        y: lerp(oldViewport.y, newViewport.y, eased),
        zoom: lerp(oldViewport.zoom, newViewport.zoom, eased)
    }
}

/**
 * Generate animation frames for a transition
 */
export function generateTransitionFrames(
    oldState: TransitionState,
    newState: TransitionState,
    frameCount: number = 30
): AnimationFrame[] {
    const matchedEdges = matchEdgesById(oldState.edges, newState.edges)
    const matchedNodes = matchNodesById(oldState.nodes, newState.nodes)

    // Pre-normalize all edge points
    const normalizedEdges = matchedEdges.map(edge => {
        const { oldNormalized, newNormalized } = normalizePoints(edge.oldPoints, edge.newPoints)
        return { ...edge, oldNormalized, newNormalized }
    })

    const frames: AnimationFrame[] = []

    for (let i = 0; i <= frameCount; i++) {
        const t = i / frameCount

        // Interpolate edges
        const edges: TransitionEdge[] = normalizedEdges.map(edge => {
            const eased = ease(t)
            const points = edge.oldNormalized.map((old, idx) =>
                interpolatePoint(old, edge.newNormalized[idx], eased)
            )
            const opacity = lerp(edge.oldOpacity, edge.newOpacity, eased)
            const strokeDashoffset = edge.appearing ? (1 - eased) * 20 : undefined

            return {
                id: edge.id,
                points,
                opacity,
                strokeDashoffset
            }
        })

        // Interpolate nodes
        const nodes = new Map<string, TransitionNode>()
        for (const matched of matchedNodes) {
            nodes.set(matched.id, interpolateNode(matched.old, matched.new, t))
        }

        // Interpolate viewport
        const viewport = interpolateViewport(oldState.viewport, newState.viewport, t)

        frames.push({ nodes, edges, viewport, t })
    }

    return frames
}

/**
 * Create a transition animation (returns a function to get frame at time t)
 */
export function createTransition(
    oldState: TransitionState,
    newState: TransitionState
): (t: number) => AnimationFrame {
    const matchedEdges = matchEdgesById(oldState.edges, newState.edges)
    const matchedNodes = matchNodesById(oldState.nodes, newState.nodes)

    // Pre-normalize all edge points
    const normalizedEdges = matchedEdges.map(edge => {
        const { oldNormalized, newNormalized } = normalizePoints(edge.oldPoints, edge.newPoints)
        return { ...edge, oldNormalized, newNormalized }
    })

    return (t: number): AnimationFrame => {
        const clampedT = Math.max(0, Math.min(1, t))

        // Interpolate edges
        const edges: TransitionEdge[] = normalizedEdges.map(edge => {
            const eased = ease(clampedT)
            const points = edge.oldNormalized.map((old, idx) =>
                interpolatePoint(old, edge.newNormalized[idx], eased)
            )
            const opacity = lerp(edge.oldOpacity, edge.newOpacity, eased)
            const strokeDashoffset = edge.appearing ? (1 - eased) * 20 : undefined

            return { id: edge.id, points, opacity, strokeDashoffset }
        })

        // Interpolate nodes
        const nodes = new Map<string, TransitionNode>()
        for (const matched of matchedNodes) {
            nodes.set(matched.id, interpolateNode(matched.old, matched.new, clampedT))
        }

        // Interpolate viewport
        const viewport = interpolateViewport(oldState.viewport, newState.viewport, clampedT)

        return { nodes, edges, viewport, t: clampedT }
    }
}
