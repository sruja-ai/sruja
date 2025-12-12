import { describe, it, expect, beforeEach } from 'vitest'
import {
    SpatialGrid,
    getPaddedRect,
    isPointInRect,
    rectsOverlap
} from '../algorithms/spatial-index'
import type { Rect } from '../geometry/rect'

describe('SpatialGrid', () => {
    let grid: SpatialGrid

    beforeEach(() => {
        grid = new SpatialGrid({ cellSize: 100 })
    })

    describe('insert and getCandidates', () => {
        it('returns empty array for empty grid', () => {
            const candidates = grid.getCandidates({ x: 0, y: 0, width: 50, height: 50 })
            expect(candidates).toEqual([])
        })

        it('returns inserted nodes that overlap query rect', () => {
            grid.insert('node1', { x: 50, y: 50, width: 100, height: 100 })
            grid.insert('node2', { x: 500, y: 500, width: 100, height: 100 })

            const candidates = grid.getCandidates({ x: 0, y: 0, width: 120, height: 120 })

            expect(candidates).toContain('node1')
            expect(candidates).not.toContain('node2')
        })

        it('handles nodes spanning multiple cells', () => {
            // Node spans cells (0,0), (0,1), (1,0), (1,1)
            grid.insert('largeNode', { x: 50, y: 50, width: 150, height: 150 })

            // Query in cell (1,1) should still find the node
            const candidates = grid.getCandidates({ x: 150, y: 150, width: 50, height: 50 })

            expect(candidates).toContain('largeNode')
        })
    })

    describe('remove', () => {
        it('removes node from all cells', () => {
            grid.insert('node1', { x: 50, y: 50, width: 150, height: 150 })
            grid.remove('node1')

            const candidates = grid.getCandidates({ x: 0, y: 0, width: 300, height: 300 })
            expect(candidates).toEqual([])
        })

        it('does not affect other nodes', () => {
            grid.insert('node1', { x: 0, y: 0, width: 50, height: 50 })
            grid.insert('node2', { x: 100, y: 100, width: 50, height: 50 })
            grid.remove('node1')

            expect(grid.getRect('node1')).toBeUndefined()
            expect(grid.getRect('node2')).toBeDefined()
        })
    })

    describe('getCandidatesForSegment', () => {
        it('finds nodes along a segment', () => {
            grid.insert('nodeA', { x: 0, y: 0, width: 50, height: 50 })
            grid.insert('nodeB', { x: 200, y: 200, width: 50, height: 50 })
            grid.insert('nodeC', { x: 500, y: 500, width: 50, height: 50 })

            // Segment from (0,0) to (250,250)
            const candidates = grid.getCandidatesForSegment({ x: 0, y: 0 }, { x: 250, y: 250 })

            expect(candidates).toContain('nodeA')
            expect(candidates).toContain('nodeB')
            expect(candidates).not.toContain('nodeC')
        })
    })

    describe('getStats', () => {
        it('returns correct statistics', () => {
            grid.insert('node1', { x: 0, y: 0, width: 50, height: 50 })
            grid.insert('node2', { x: 100, y: 100, width: 50, height: 50 })

            const stats = grid.getStats()

            expect(stats.nodeCount).toBe(2)
            expect(stats.cellCount).toBeGreaterThan(0)
        })
    })

    describe('clear', () => {
        it('removes all entries', () => {
            grid.insert('node1', { x: 0, y: 0, width: 50, height: 50 })
            grid.insert('node2', { x: 100, y: 100, width: 50, height: 50 })
            grid.clear()

            expect(grid.getAllNodeIds()).toEqual([])
            expect(grid.getStats().nodeCount).toBe(0)
        })
    })
})

describe('getPaddedRect', () => {
    it('expands rect by padding on all sides', () => {
        const rect: Rect = { x: 100, y: 100, width: 50, height: 50 }
        const padded = getPaddedRect(rect, 10)

        expect(padded.x).toBe(90)
        expect(padded.y).toBe(90)
        expect(padded.width).toBe(70)
        expect(padded.height).toBe(70)
    })
})

describe('isPointInRect', () => {
    it('returns true for point inside rect', () => {
        const rect: Rect = { x: 0, y: 0, width: 100, height: 100 }
        expect(isPointInRect({ x: 50, y: 50 }, rect)).toBe(true)
    })

    it('returns false for point outside rect', () => {
        const rect: Rect = { x: 0, y: 0, width: 100, height: 100 }
        expect(isPointInRect({ x: 150, y: 50 }, rect)).toBe(false)
    })

    it('returns true for point on edge', () => {
        const rect: Rect = { x: 0, y: 0, width: 100, height: 100 }
        expect(isPointInRect({ x: 0, y: 50 }, rect)).toBe(true)
        expect(isPointInRect({ x: 100, y: 100 }, rect)).toBe(true)
    })
})

describe('rectsOverlap', () => {
    it('returns true for overlapping rects', () => {
        const a: Rect = { x: 0, y: 0, width: 100, height: 100 }
        const b: Rect = { x: 50, y: 50, width: 100, height: 100 }
        expect(rectsOverlap(a, b)).toBe(true)
    })

    it('returns false for non-overlapping rects', () => {
        const a: Rect = { x: 0, y: 0, width: 100, height: 100 }
        const b: Rect = { x: 200, y: 200, width: 100, height: 100 }
        expect(rectsOverlap(a, b)).toBe(false)
    })

    it('returns true for touching rects', () => {
        const a: Rect = { x: 0, y: 0, width: 100, height: 100 }
        const b: Rect = { x: 100, y: 0, width: 100, height: 100 }
        expect(rectsOverlap(a, b)).toBe(true) // Touching at edge
    })
})
