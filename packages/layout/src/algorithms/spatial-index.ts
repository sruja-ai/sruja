/**
 * Spatial Index for efficient collision detection
 * 
 * Uses a grid-based approach to achieve O(1) neighbor lookups
 * instead of O(n) full scans for each edge routing check.
 */

import type { Rect } from '../geometry/rect'

export interface SpatialGridOptions {
    /** Size of each grid cell in pixels */
    cellSize: number
}

/**
 * A grid-based spatial index for efficient collision detection
 */
export class SpatialGrid {
    private readonly cellSize: number
    private readonly cells: Map<string, Set<string>>
    private readonly nodeRects: Map<string, Rect>

    constructor(options: SpatialGridOptions = { cellSize: 100 }) {
        this.cellSize = options.cellSize
        this.cells = new Map()
        this.nodeRects = new Map()
    }

    /**
     * Clear all entries from the index
     */
    clear(): void {
        this.cells.clear()
        this.nodeRects.clear()
    }

    /**
     * Get all cell keys that a rect overlaps
     */
    private getCellKeysForRect(rect: Rect): string[] {
        const keys: string[] = []
        const minCellX = Math.floor(rect.x / this.cellSize)
        const maxCellX = Math.floor((rect.x + rect.width) / this.cellSize)
        const minCellY = Math.floor(rect.y / this.cellSize)
        const maxCellY = Math.floor((rect.y + rect.height) / this.cellSize)

        for (let cx = minCellX; cx <= maxCellX; cx++) {
            for (let cy = minCellY; cy <= maxCellY; cy++) {
                keys.push(`${cx},${cy}`)
            }
        }
        return keys
    }

    /**
     * Insert a node into the spatial index
     */
    insert(nodeId: string, bbox: Rect): void {
        this.nodeRects.set(nodeId, bbox)

        const keys = this.getCellKeysForRect(bbox)
        for (const key of keys) {
            if (!this.cells.has(key)) {
                this.cells.set(key, new Set())
            }
            this.cells.get(key)!.add(nodeId)
        }
    }

    /**
     * Remove a node from the spatial index
     */
    remove(nodeId: string): void {
        const rect = this.nodeRects.get(nodeId)
        if (!rect) return

        const keys = this.getCellKeysForRect(rect)
        for (const key of keys) {
            this.cells.get(key)?.delete(nodeId)
        }
        this.nodeRects.delete(nodeId)
    }

    /**
     * Get candidate node IDs that might intersect with the given rect.
     * These are nodes in cells that the query rect overlaps.
     */
    getCandidates(queryRect: Rect): string[] {
        const candidates = new Set<string>()
        const keys = this.getCellKeysForRect(queryRect)

        for (const key of keys) {
            const cell = this.cells.get(key)
            if (cell) {
                for (const nodeId of cell) {
                    candidates.add(nodeId)
                }
            }
        }

        return Array.from(candidates)
    }

    /**
     * Get candidate node IDs that might intersect with a line segment
     */
    getCandidatesForSegment(p1: { x: number; y: number }, p2: { x: number; y: number }): string[] {
        // Create a bounding box for the segment
        const minX = Math.min(p1.x, p2.x)
        const maxX = Math.max(p1.x, p2.x)
        const minY = Math.min(p1.y, p2.y)
        const maxY = Math.max(p1.y, p2.y)

        const segmentRect: Rect = {
            x: minX,
            y: minY,
            width: maxX - minX || 1, // At least 1 for vertical/horizontal lines
            height: maxY - minY || 1
        }

        return this.getCandidates(segmentRect)
    }

    /**
     * Get the stored rect for a node
     */
    getRect(nodeId: string): Rect | undefined {
        return this.nodeRects.get(nodeId)
    }

    /**
     * Get all node IDs in the index
     */
    getAllNodeIds(): string[] {
        return Array.from(this.nodeRects.keys())
    }

    /**
     * Get statistics about the index
     */
    getStats(): { nodeCount: number; cellCount: number; avgNodesPerCell: number } {
        const nodeCount = this.nodeRects.size
        const cellCount = this.cells.size
        let totalNodeRefs = 0
        for (const cell of this.cells.values()) {
            totalNodeRefs += cell.size
        }
        const avgNodesPerCell = cellCount > 0 ? totalNodeRefs / cellCount : 0

        return { nodeCount, cellCount, avgNodesPerCell }
    }
}

/**
 * Create a padded bounding box for no-entry zone
 */
export function getPaddedRect(rect: Rect, padding: number): Rect {
    return {
        x: rect.x - padding,
        y: rect.y - padding,
        width: rect.width + padding * 2,
        height: rect.height + padding * 2
    }
}

/**
 * Check if a point is inside a rect
 */
export function isPointInRect(point: { x: number; y: number }, rect: Rect): boolean {
    return (
        point.x >= rect.x &&
        point.x <= rect.x + rect.width &&
        point.y >= rect.y &&
        point.y <= rect.y + rect.height
    )
}

/**
 * Check if two rects overlap
 */
export function rectsOverlap(a: Rect, b: Rect): boolean {
    return !(
        a.x + a.width < b.x ||
        b.x + b.width < a.x ||
        a.y + a.height < b.y ||
        b.y + b.height < a.y
    )
}
