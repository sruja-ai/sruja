import { describe, it, expect } from 'vitest'
import {
    computeClutterScore,
    shouldReoptimize,
    getOptimizationSuggestions,
    type NodeBox,
    type EdgePath
} from '../algorithms/clutter-detection'
import {
    optimizeLayout,
    expandGrid,
    type OptimizableLayout
} from '../algorithms/self-optimizer'

function makeBox(id: string, x: number, y: number, width = 200, height = 120): NodeBox {
    return { id, bbox: { x, y, width, height } }
}

function makeEdge(id: string, points: { x: number; y: number }[]): EdgePath {
    return { id, points }
}

describe('Clutter Detection', () => {
    describe('computeClutterScore', () => {
        it('returns low score for well-spaced nodes', () => {
            const nodes = [
                makeBox('a', 0, 0),
                makeBox('b', 400, 0),
                makeBox('c', 0, 300),
                makeBox('d', 400, 300)
            ]
            const edges: EdgePath[] = []

            const score = computeClutterScore(nodes, edges)

            expect(score.nodeProximity).toBeLessThan(0.3)
            expect(score.total).toBeLessThan(0.5)
            expect(score.needsOptimization).toBe(false)
        })

        it('returns high score for overlapping nodes', () => {
            const nodes = [
                makeBox('a', 0, 0),
                makeBox('b', 50, 50),  // Overlapping with 'a'
                makeBox('c', 100, 100) // Overlapping chain
            ]
            const edges: EdgePath[] = []

            const score = computeClutterScore(nodes, edges)

            expect(score.nodeProximity).toBeGreaterThan(0.5)
        })

        it('detects edge crossings', () => {
            const nodes = [
                makeBox('a', 0, 0),
                makeBox('b', 200, 0),
                makeBox('c', 0, 200),
                makeBox('d', 200, 200)
            ]
            // These edges cross: (a→d) crosses (b→c)
            const edges = [
                makeEdge('e1', [{ x: 100, y: 60 }, { x: 300, y: 260 }]),
                makeEdge('e2', [{ x: 300, y: 60 }, { x: 100, y: 260 }])
            ]

            const score = computeClutterScore(nodes, edges)

            expect(score.edgeCrossings).toBeGreaterThan(0)
        })

        it('calculates routing complexity', () => {
            const nodes = [makeBox('a', 0, 0), makeBox('b', 400, 400)]
            const edges = [
                makeEdge('e1', [
                    { x: 200, y: 60 },
                    { x: 300, y: 60 },
                    { x: 300, y: 200 },
                    { x: 400, y: 200 },
                    { x: 400, y: 460 }
                ])
            ]

            const score = computeClutterScore(nodes, edges)

            expect(score.routingComplexity).toBeGreaterThan(0.5)
        })
    })

    describe('shouldReoptimize', () => {
        it('returns true for high clutter score', () => {
            const highScore = {
                nodeProximity: 0.8,
                edgeCrossings: 0.5,
                routingComplexity: 0.6,
                density: 0.7,
                total: 0.75,
                needsOptimization: true
            }

            expect(shouldReoptimize(highScore)).toBe(true)
        })

        it('returns false for low clutter score', () => {
            const lowScore = {
                nodeProximity: 0.1,
                edgeCrossings: 0.1,
                routingComplexity: 0.1,
                density: 0.2,
                total: 0.12,
                needsOptimization: false
            }

            expect(shouldReoptimize(lowScore)).toBe(false)
        })
    })

    describe('getOptimizationSuggestions', () => {
        it('suggests expanding grid for high node proximity', () => {
            const score = {
                nodeProximity: 0.6,
                edgeCrossings: 0.1,
                routingComplexity: 0.2,
                density: 0.3,
                total: 0.4,
                needsOptimization: false
            }

            const suggestions = getOptimizationSuggestions(score)

            expect(suggestions.some(s => s.includes('spacing') || s.includes('grid'))).toBe(true)
        })
    })
})

describe('Self-Optimizer', () => {
    describe('expandGrid', () => {
        it('expands node positions relative to center', () => {
            const layout: OptimizableLayout = {
                nodes: new Map([
                    ['a', { position: { x: 0, y: 0 }, size: { width: 100, height: 80 } }],
                    ['b', { position: { x: 200, y: 0 }, size: { width: 100, height: 80 } }]
                ]),
                edges: []
            }

            const expanded = expandGrid(layout, 2.0)

            const posA = expanded.nodes.get('a')!.position
            const posB = expanded.nodes.get('b')!.position

            // Distance between nodes should be larger
            const originalDist = 200
            const newDist = posB.x - posA.x
            expect(newDist).toBeGreaterThan(originalDist * 1.5)
        })
    })

    describe('optimizeLayout', () => {
        it('applies transformations when clutter is high', () => {
            // Create a cluttered layout with overlapping nodes
            const layout: OptimizableLayout = {
                nodes: new Map([
                    ['a', { position: { x: 0, y: 0 }, size: { width: 200, height: 120 } }],
                    ['b', { position: { x: 10, y: 10 }, size: { width: 200, height: 120 } }],
                    ['c', { position: { x: 20, y: 20 }, size: { width: 200, height: 120 } }],
                    ['d', { position: { x: 30, y: 30 }, size: { width: 200, height: 120 } }]
                ]),
                edges: []
            }

            // Use extremely low threshold to guarantee optimization triggers
            const result = optimizeLayout(layout, { maxPasses: 3, threshold: 0.01 })

            expect(result.passesApplied).toBeGreaterThan(0)
            expect(result.transformations.length).toBeGreaterThan(0)
        })

        it('does not modify well-spaced layouts', () => {
            const layout: OptimizableLayout = {
                nodes: new Map([
                    ['a', { position: { x: 0, y: 0 }, size: { width: 100, height: 80 } }],
                    ['b', { position: { x: 500, y: 0 }, size: { width: 100, height: 80 } }],
                    ['c', { position: { x: 0, y: 400 }, size: { width: 100, height: 80 } }],
                    ['d', { position: { x: 500, y: 400 }, size: { width: 100, height: 80 } }]
                ]),
                edges: []
            }

            const result = optimizeLayout(layout, { threshold: 0.5 })

            expect(result.passesApplied).toBe(0)
        })
    })
})
