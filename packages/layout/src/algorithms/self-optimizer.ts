/**
 * Self-Optimization Algorithm
 * 
 * Takes a layout with clutter score and applies transformations
 * to reduce clutter and improve readability.
 */

import type { Point } from '../geometry/point'
import type { ClutterScore, NodeBox, EdgePath } from './clutter-detection'
import { computeClutterScore } from './clutter-detection'
import { MAX_OPTIMIZATION_PASSES, V_SPACING } from '../constants'

export interface OptimizableLayout {
    nodes: Map<string, { position: Point; size: { width: number; height: number } }>
    edges: EdgePath[]
    boundarySize?: { width: number; height: number }
}

export interface OptimizationResult {
    layout: OptimizableLayout
    finalScore: ClutterScore
    passesApplied: number
    transformations: string[]
}

/**
 * Expand grid spacing by a factor
 */
export function expandGrid(
    layout: OptimizableLayout,
    factor: number = 1.2
): OptimizableLayout {
    const nodes = new Map<string, { position: Point; size: { width: number; height: number } }>()

    // Find center of layout
    let centerX = 0, centerY = 0, count = 0
    for (const [, node] of layout.nodes) {
        centerX += node.position.x + node.size.width / 2
        centerY += node.position.y + node.size.height / 2
        count++
    }

    if (count > 0) {
        centerX /= count
        centerY /= count
    }

    // Expand positions relative to center
    for (const [id, node] of layout.nodes) {
        const dx = (node.position.x + node.size.width / 2) - centerX
        const dy = (node.position.y + node.size.height / 2) - centerY

        const newX = centerX + dx * factor - node.size.width / 2
        const newY = centerY + dy * factor - node.size.height / 2

        nodes.set(id, { position: { x: newX, y: newY }, size: node.size })
    }

    // Edges need re-routing (not handled here, caller should re-route)
    return {
        nodes,
        edges: layout.edges,
        boundarySize: layout.boundarySize
            ? {
                width: layout.boundarySize.width * factor,
                height: layout.boundarySize.height * factor
            }
            : undefined
    }
}

/**
 * Shift nodes horizontally to reduce vertical edge crossings
 */
export function rebalanceHorizontal(
    layout: OptimizableLayout
): OptimizableLayout {
    const nodes = new Map(layout.nodes)

    // Group nodes by approximate Y position (rows)
    const rows = new Map<number, string[]>()
    const rowThreshold = V_SPACING * 0.5

    for (const [id, node] of nodes) {
        const rowKey = Math.round(node.position.y / rowThreshold) * rowThreshold
        if (!rows.has(rowKey)) rows.set(rowKey, [])
        rows.get(rowKey)!.push(id)
    }

    // For each row, distribute nodes evenly
    for (const [, rowNodes] of rows) {
        if (rowNodes.length <= 1) continue

        // Sort by x position
        rowNodes.sort((a, b) => nodes.get(a)!.position.x - nodes.get(b)!.position.x)

        const first = nodes.get(rowNodes[0])!
        // const last = nodes.get(rowNodes[rowNodes.length - 1])!
        // const totalWidth = (last.position.x + last.size.width) - first.position.x
        // const spacing = totalWidth / (rowNodes.length - 1)

        // Redistribute ensuring no overlap
        let currentX = first.position.x
        const minGap = 50

        // If naive spacing causes overlap, enforce min spacing
        // Check if spacing < max width in row? Better to just layout sequentially
        // However, we want to maintain the "center" of gravity or the span if possible
        // But preventing overlap is priority.

        // Strategy: Place sequentially starting from currentX
        rowNodes.forEach((id) => {
            const node = nodes.get(id)!
            if (currentX < node.position.x) {
                // If there's natural gap, keep it? 
                // No, we want to 'rebalance' i.e. distribute.
                // But blindly distributing caused overlap.
            }

            nodes.set(id, {
                ...node,
                position: { x: currentX, y: node.position.y }
            })

            currentX += node.size.width + minGap
        })
    }

    return { nodes, edges: layout.edges, boundarySize: layout.boundarySize }
}

/**
 * Snap nodes to grid (8px)
 */
export function snapToGrid(
    layout: OptimizableLayout,
    gridSize: number = 8
): OptimizableLayout {
    const nodes = new Map<string, { position: Point; size: { width: number; height: number } }>()
    for (const [id, node] of layout.nodes) {
        nodes.set(id, {
            position: {
                x: Math.round(node.position.x / gridSize) * gridSize,
                y: Math.round(node.position.y / gridSize) * gridSize
            },
            size: node.size
        })
    }
    return { nodes, edges: layout.edges, boundarySize: layout.boundarySize }
}

/**
 * Convert layout to NodeBox format for clutter scoring
 */
function toNodeBoxes(layout: OptimizableLayout): NodeBox[] {
    const boxes: NodeBox[] = []
    for (const [id, node] of layout.nodes) {
        boxes.push({
            id,
            bbox: {
                x: node.position.x,
                y: node.position.y,
                width: node.size.width,
                height: node.size.height
            }
        })
    }
    return boxes
}

/**
 * Run self-optimization loop
 */
export function optimizeLayout(
    layout: OptimizableLayout,
    options: {
        maxPasses?: number
        threshold?: number
    } = {}
): OptimizationResult {
    const { maxPasses = MAX_OPTIMIZATION_PASSES, threshold = 0.7 } = options

    let currentLayout = layout
    let currentScore = computeClutterScore(toNodeBoxes(currentLayout), currentLayout.edges, { threshold })
    const transformations: string[] = []
    let passesApplied = 0

    while (currentScore.needsOptimization && passesApplied < maxPasses) {
        passesApplied++

        // Apply transformations based on highest clutter sources
        if (currentScore.nodeProximity > 0.4) {
            currentLayout = expandGrid(currentLayout, 1.15)
            transformations.push(`Pass ${passesApplied}: Expanded grid by 15%`)
        } else if (currentScore.edgeCrossings > 0.3) {
            currentLayout = rebalanceHorizontal(currentLayout)
            transformations.push(`Pass ${passesApplied}: Rebalanced horizontal positions`)
        } else {
            // Minor expansion as fallback
            currentLayout = expandGrid(currentLayout, 1.1)
            transformations.push(`Pass ${passesApplied}: Minor grid expansion`)
        }

        // Recalculate score
        currentScore = computeClutterScore(toNodeBoxes(currentLayout), currentLayout.edges, { threshold })
    }

    // FINAL PASS: Snap to grid (Top-Down Refinement rule)
    currentLayout = snapToGrid(currentLayout, 8)
    transformations.push('Final Pass: Snapped to 8px grid')

    return {
        layout: currentLayout,
        finalScore: currentScore,
        passesApplied,
        transformations
    }
}
