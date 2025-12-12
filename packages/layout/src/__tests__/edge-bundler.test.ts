import { describe, it, expect } from 'vitest'
import {
    identifyBundles,
    applyBundleOffsets,
    bundleEdges,
    type EdgeForBundling
} from '../algorithms/edge-bundler'

describe('Edge Bundler', () => {
    describe('identifyBundles', () => {
        it('bundles edges with same source and target', () => {
            const edges: EdgeForBundling[] = [
                { id: 'e1', sourceId: 'A', targetId: 'B', points: [{ x: 0, y: 0 }, { x: 100, y: 0 }] },
                { id: 'e2', sourceId: 'A', targetId: 'B', points: [{ x: 0, y: 0 }, { x: 100, y: 0 }] }
            ]

            const bundles = identifyBundles(edges)

            expect(bundles).toHaveLength(1)
            expect(bundles[0].edgeIds).toContain('e1')
            expect(bundles[0].edgeIds).toContain('e2')
        })

        it('creates bidirectional bundle for opposite direction edges', () => {
            const edges: EdgeForBundling[] = [
                { id: 'e1', sourceId: 'A', targetId: 'B', points: [{ x: 0, y: 0 }, { x: 100, y: 0 }] },
                { id: 'e2', sourceId: 'B', targetId: 'A', points: [{ x: 100, y: 0 }, { x: 0, y: 0 }] }
            ]

            const bundles = identifyBundles(edges)

            expect(bundles).toHaveLength(1)
            expect(bundles[0].isBidirectional).toBe(true)
        })

        it('does not bundle single edges', () => {
            const edges: EdgeForBundling[] = [
                { id: 'e1', sourceId: 'A', targetId: 'B', points: [{ x: 0, y: 0 }, { x: 100, y: 0 }] }
            ]

            const bundles = identifyBundles(edges)

            expect(bundles).toHaveLength(0)
        })

        it('respects minBundleSize option', () => {
            const edges: EdgeForBundling[] = [
                { id: 'e1', sourceId: 'A', targetId: 'B', points: [] },
                { id: 'e2', sourceId: 'A', targetId: 'B', points: [] }
            ]

            const bundles = identifyBundles(edges, { minBundleSize: 3 })

            expect(bundles).toHaveLength(0)
        })

        it('calculates spread offsets for bundled edges', () => {
            const edges: EdgeForBundling[] = [
                { id: 'e1', sourceId: 'A', targetId: 'B', points: [] },
                { id: 'e2', sourceId: 'A', targetId: 'B', points: [] },
                { id: 'e3', sourceId: 'A', targetId: 'B', points: [] }
            ]

            const bundles = identifyBundles(edges, { fanOutSpacing: 10 })

            expect(bundles).toHaveLength(1)
            const offsets = bundles[0].spreadOffsets

            // Three edges should be offset at -10, 0, +10
            const offsetValues = [...offsets.values()].sort((a, b) => a - b)
            expect(offsetValues).toEqual([-10, 0, 10])
        })
    })

    describe('applyBundleOffsets', () => {
        it('adjusts edge endpoints based on offsets', () => {
            const edges: EdgeForBundling[] = [
                { id: 'e1', sourceId: 'A', targetId: 'B', points: [{ x: 0, y: 0 }, { x: 100, y: 0 }] },
                { id: 'e2', sourceId: 'A', targetId: 'B', points: [{ x: 0, y: 0 }, { x: 100, y: 0 }] }
            ]

            const bundles = identifyBundles(edges, { fanOutSpacing: 10 })
            const adjusted = applyBundleOffsets(edges, bundles)

            expect(adjusted.has('e1')).toBe(true)
            expect(adjusted.has('e2')).toBe(true)

            // Edges should have different Y coordinates after offset
            const e1Start = adjusted.get('e1')![0]
            const e2Start = adjusted.get('e2')![0]

            expect(e1Start.y).not.toEqual(e2Start.y)
        })

        it('leaves non-bundled edges unchanged', () => {
            const edges: EdgeForBundling[] = [
                { id: 'e1', sourceId: 'A', targetId: 'B', points: [{ x: 0, y: 0 }, { x: 100, y: 0 }] },
                { id: 'e2', sourceId: 'C', targetId: 'D', points: [{ x: 50, y: 50 }, { x: 150, y: 50 }] }
            ]

            const bundles = identifyBundles(edges) // No bundles (different pairs)
            const adjusted = applyBundleOffsets(edges, bundles)

            expect(adjusted.get('e1')).toEqual(edges[0].points)
            expect(adjusted.get('e2')).toEqual(edges[1].points)
        })
    })

    describe('bundleEdges', () => {
        it('returns both bundles and adjusted points', () => {
            const edges: EdgeForBundling[] = [
                { id: 'e1', sourceId: 'A', targetId: 'B', points: [{ x: 0, y: 0 }, { x: 100, y: 0 }] },
                { id: 'e2', sourceId: 'A', targetId: 'B', points: [{ x: 0, y: 0 }, { x: 100, y: 0 }] }
            ]

            const result = bundleEdges(edges)

            expect(result.bundles).toHaveLength(1)
            expect(result.adjustedPoints.size).toBe(2)
        })
    })
})
