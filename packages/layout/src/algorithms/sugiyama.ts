import { C4Id } from '../brand'
import { Size } from '../types'
import { C4LayoutOptions } from '../c4-options'

export interface SugiyamaNode {
    id: C4Id
    size: Size
    layer: number
    order: number
    x: number
    y: number
}

export interface SugiyamaResult {
    width: number
    height: number
    nodes: Map<C4Id, SugiyamaNode>
}

/**
 * Performs a localized Sugiyama layout for a set of nodes and relationships.
 */
export function layoutSugiyama(
    nodes: { id: C4Id; size: Size }[],
    relationships: { from: C4Id; to: C4Id }[],
    options: C4LayoutOptions
): SugiyamaResult {
    const resultNodes = new Map<C4Id, SugiyamaNode>()
    const nodeMap = new Map<C4Id, { id: C4Id; size: Size }>()
    nodes.forEach(n => nodeMap.set(n.id, n))

    // 1. Assign Layers (Longest Path)
    const layerMap = assignLayers(nodes, relationships)
    const layers: C4Id[][] = []

    layerMap.forEach((layer, id) => {
        if (!layers[layer]) layers[layer] = []
        layers[layer].push(id)
    })

    // 2. Minimize Crossings (Order Layers)
    // Simple heuristic: Barycenter or Median.
    // We'll use a simplified barycenter-ish approach (iterative).
    for (let i = 0; i < 4; i++) { // Few iterations
        orderLayers(layers, relationships, true)  // Down
        orderLayers(layers, relationships, false) // Up
    }

    // 3. Assign Coordinates
    // X: Compact packing within layer
    // Y: Layer heights
    const xMap = new Map<C4Id, number>()
    const yMap = new Map<C4Id, number>()

    const rankSpacing = options.spacing.rank.Container ?? 60
    const nodeSpacing = options.spacing.node.Container ?? 40 // Default spacing

    let maxWidth = 0

    for (let l = 0; l < layers.length; l++) {
        const layerIds = layers[l] || []
        if (layerIds.length === 0) continue

        const layerNodes = layerIds.map(id => nodeMap.get(id)!)

        const totalWidth = layerNodes.reduce((sum, n) => sum + n.size.width, 0) + nodeSpacing * (layerNodes.length - 1)
        maxWidth = Math.max(maxWidth, totalWidth)
    }

    // Re-iterate to calculate exact coords
    let currentY = 0
    for (let l = 0; l < layers.length; l++) {
        const layerIds = layers[l] || []
        if (layerIds.length === 0) continue

        const layerNodes = layerIds.map(id => nodeMap.get(id)!)
        const maxHeight = Math.max(...layerNodes.map(n => n.size.height))

        const totalWidth = layerNodes.reduce((sum, n) => sum + n.size.width, 0) + nodeSpacing * (layerNodes.length - 1)

        let startX = (maxWidth - totalWidth) / 2 // Center alignment

        let currentX = startX
        layerNodes.forEach((n, i) => {
            xMap.set(n.id, currentX)
            const offY = (maxHeight - n.size.height) / 2
            yMap.set(n.id, currentY + offY)

            resultNodes.set(n.id, {
                id: n.id,
                size: n.size,
                layer: l,
                order: i,
                x: currentX,
                y: currentY + offY // Store top-left
            })

            currentX += n.size.width + nodeSpacing
        })

        currentY += maxHeight + rankSpacing
    }

    return {
        width: maxWidth,
        height: Math.max(0, currentY - rankSpacing), // Remove last spacing
        nodes: resultNodes
    }
}

function assignLayers(nodes: { id: C4Id }[], relationships: { from: C4Id; to: C4Id }[]): Map<C4Id, number> {
    const layerMap = new Map<C4Id, number>()

    // Build adjacency
    const outEdges = new Map<C4Id, C4Id[]>()
    relationships.forEach(r => {
        // Only internal relationships
        if (!outEdges.has(r.from)) outEdges.set(r.from, [])
        outEdges.get(r.from)!.push(r.to)
    })

    // This is actually "Height" (distance to leaf).
    // Standard Sugiyama uses "Rank" (distance from root).
    // "Root" = node with no incoming edges.
    // Let's compute In-Degree.

    const inDegree = new Map<C4Id, number>()
    nodes.forEach(n => inDegree.set(n.id, 0))
    relationships.forEach(r => {
        inDegree.set(r.to, (inDegree.get(r.to) || 0) + 1)
    })

    // Assign Rank 0 to sources
    // Topological sort / Longest path from sources.
    // Or: recursive "Height" from sinks?
    // Let's use recursive rank from sources.

    const rank = new Map<C4Id, number>()
    const visiting = new Set<C4Id>()

    function getRank(id: C4Id): number {
        if (rank.has(id)) return rank.get(id)!
        if (visiting.has(id)) return 0 // Cycle
        visiting.add(id)

        let maxPrev = -1
        // Find incoming edges
        // Inefficient O(E) per node?
        // Better to pre-build inEdges.
        const incoming = relationships.filter(r => r.to === id)
        for (const r of incoming) {
            const rk = getRank(r.from)
            if (rk > maxPrev) maxPrev = rk
        }

        visiting.delete(id)
        const result = maxPrev + 1
        rank.set(id, result)
        return result
    }

    nodes.forEach(n => {
        layerMap.set(n.id, getRank(n.id))
    })

    return layerMap
}


function orderLayers(layers: C4Id[][], relationships: { from: C4Id; to: C4Id }[], down: boolean) {
    // Implementation of barycenter crossing minimization
    // ...
    // For now, just stable sort by ID or keep order to ensure determinism if no implementation details provided yet?
    // I'll implement a basic barycenter.

    const start = down ? 1 : layers.length - 2
    const end = down ? layers.length : -1
    const step = down ? 1 : -1

    for (let i = start; i !== end; i += step) {
        const currentLayer = layers[i]
        const prevLayer = layers[i - step]
        // Sort currentLayer based on neighbors in prevLayer

        const nodePos = new Map<C4Id, number>()
        prevLayer.forEach((id, idx) => nodePos.set(id, idx))

        const scores = currentLayer.map(id => {
            const neighbors = down
                ? relationships.filter(r => r.to === id && nodePos.has(r.from)).map(r => r.from)
                : relationships.filter(r => r.from === id && nodePos.has(r.to)).map(r => r.to)

            if (neighbors.length === 0) return { id, score: -1 } // Keep current pos or use index

            const sum = neighbors.reduce((a, b) => a + nodePos.get(b)!, 0)
            return { id, score: sum / neighbors.length }
        })

        scores.sort((a, b) => {
            if (a.score === -1 && b.score === -1) return 0
            if (a.score === -1) return 1
            if (b.score === -1) return -1
            return a.score - b.score
        })

        layers[i] = scores.map(s => s.id)
    }
}
