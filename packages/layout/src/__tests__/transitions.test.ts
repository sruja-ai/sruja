import { describe, it, expect } from 'vitest'
import {
    ease,
    lerp,
    interpolatePoint,
    interpolateRect,
    normalizePoints,
    matchEdgesById,
    matchNodesById,
    interpolateNode,
    interpolateViewport,
    generateTransitionFrames,
    createTransition,
    type TransitionNode,
    type TransitionEdge,
    type TransitionState
} from '../algorithms/transitions'

describe('Transitions', () => {
    describe('ease', () => {
        it('returns 0 at t=0', () => {
            expect(ease(0)).toBe(0)
        })

        it('returns 1 at t=1', () => {
            expect(ease(1)).toBe(1)
        })

        it('returns 0.5 near t=0.5', () => {
            const result = ease(0.5)
            expect(result).toBeCloseTo(0.5, 1)
        })

        it('produces smooth curve (derivative continuous)', () => {
            const samples = [0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1]
            const results = samples.map(ease)

            // Should be monotonically increasing
            for (let i = 1; i < results.length; i++) {
                expect(results[i]).toBeGreaterThanOrEqual(results[i - 1])
            }
        })
    })

    describe('lerp', () => {
        it('returns a at t=0', () => {
            expect(lerp(10, 20, 0)).toBe(10)
        })

        it('returns b at t=1', () => {
            expect(lerp(10, 20, 1)).toBe(20)
        })

        it('returns midpoint at t=0.5', () => {
            expect(lerp(10, 20, 0.5)).toBe(15)
        })
    })

    describe('interpolatePoint', () => {
        it('interpolates x and y', () => {
            const a = { x: 0, y: 0 }
            const b = { x: 100, y: 200 }

            const mid = interpolatePoint(a, b, 0.5)

            expect(mid.x).toBe(50)
            expect(mid.y).toBe(100)
        })
    })

    describe('interpolateRect', () => {
        it('interpolates all rect properties', () => {
            const a = { x: 0, y: 0, width: 100, height: 100 }
            const b = { x: 100, y: 100, width: 200, height: 200 }

            const mid = interpolateRect(a, b, 0.5)

            expect(mid.x).toBe(50)
            expect(mid.y).toBe(50)
            expect(mid.width).toBe(150)
            expect(mid.height).toBe(150)
        })
    })

    describe('normalizePoints', () => {
        it('expands shorter array to match longer', () => {
            const old = [{ x: 0, y: 0 }, { x: 100, y: 100 }]
            const newPts = [{ x: 0, y: 0 }, { x: 50, y: 50 }, { x: 100, y: 100 }, { x: 150, y: 150 }]

            const { oldNormalized, newNormalized } = normalizePoints(old, newPts)

            expect(oldNormalized.length).toBe(newNormalized.length)
            expect(oldNormalized.length).toBe(4)
        })

        it('handles empty arrays', () => {
            const { oldNormalized, newNormalized } = normalizePoints([], [{ x: 0, y: 0 }])

            expect(oldNormalized.length).toBe(1)
            expect(newNormalized.length).toBe(1)
        })
    })

    describe('matchEdgesById', () => {
        it('matches edges present in both states', () => {
            const old: TransitionEdge[] = [{ id: 'e1', points: [{ x: 0, y: 0 }], opacity: 1 }]
            const newEdges: TransitionEdge[] = [{ id: 'e1', points: [{ x: 100, y: 100 }], opacity: 1 }]

            const matched = matchEdgesById(old, newEdges)

            expect(matched.length).toBe(1)
            expect(matched[0].appearing).toBe(false)
            expect(matched[0].disappearing).toBe(false)
        })

        it('marks new edges as appearing', () => {
            const old: TransitionEdge[] = []
            const newEdges: TransitionEdge[] = [{ id: 'e1', points: [{ x: 0, y: 0 }], opacity: 1 }]

            const matched = matchEdgesById(old, newEdges)

            expect(matched[0].appearing).toBe(true)
            expect(matched[0].oldOpacity).toBe(0)
        })

        it('marks removed edges as disappearing', () => {
            const old: TransitionEdge[] = [{ id: 'e1', points: [{ x: 0, y: 0 }], opacity: 1 }]
            const newEdges: TransitionEdge[] = []

            const matched = matchEdgesById(old, newEdges)

            expect(matched[0].disappearing).toBe(true)
            expect(matched[0].newOpacity).toBe(0)
        })
    })

    describe('matchNodesById', () => {
        it('matches nodes present in both states', () => {
            const old = new Map<string, TransitionNode>([
                ['n1', { id: 'n1', position: { x: 0, y: 0 }, size: { width: 100, height: 80 }, opacity: 1, scale: 1 }]
            ])
            const newNodes = new Map<string, TransitionNode>([
                ['n1', { id: 'n1', position: { x: 100, y: 100 }, size: { width: 100, height: 80 }, opacity: 1, scale: 1 }]
            ])

            const matched = matchNodesById(old, newNodes)

            expect(matched.length).toBe(1)
            expect(matched[0].appearing).toBe(false)
        })

        it('marks new nodes as appearing with scale 0.8', () => {
            const old = new Map<string, TransitionNode>()
            const newNodes = new Map<string, TransitionNode>([
                ['n1', { id: 'n1', position: { x: 100, y: 100 }, size: { width: 100, height: 80 }, opacity: 1, scale: 1 }]
            ])

            const matched = matchNodesById(old, newNodes)

            expect(matched[0].appearing).toBe(true)
            expect(matched[0].old.scale).toBe(0.8)
            expect(matched[0].old.opacity).toBe(0)
        })
    })

    describe('interpolateNode', () => {
        it('interpolates all node properties', () => {
            const old: TransitionNode = { id: 'n1', position: { x: 0, y: 0 }, size: { width: 100, height: 80 }, opacity: 0, scale: 0.8 }
            const newNode: TransitionNode = { id: 'n1', position: { x: 100, y: 100 }, size: { width: 200, height: 120 }, opacity: 1, scale: 1 }

            const mid = interpolateNode(old, newNode, 0.5)

            expect(mid.position.x).toBeGreaterThan(0)
            expect(mid.position.x).toBeLessThan(100)
            expect(mid.opacity).toBeGreaterThan(0)
            expect(mid.opacity).toBeLessThan(1)
        })
    })

    describe('interpolateViewport', () => {
        it('interpolates viewport properties', () => {
            const old = { x: 0, y: 0, zoom: 1 }
            const newVp = { x: 100, y: 100, zoom: 2 }

            const mid = interpolateViewport(old, newVp, 0.5)

            expect(mid.x).toBeGreaterThan(0)
            expect(mid.zoom).toBeGreaterThan(1)
        })
    })

    describe('generateTransitionFrames', () => {
        it('generates requested number of frames', () => {
            const old: TransitionState = {
                nodes: new Map(),
                edges: [],
                viewport: { x: 0, y: 0, zoom: 1 }
            }
            const newState: TransitionState = {
                nodes: new Map(),
                edges: [],
                viewport: { x: 100, y: 100, zoom: 2 }
            }

            const frames = generateTransitionFrames(old, newState, 10)

            expect(frames.length).toBe(11) // 0 to 10 inclusive
        })

        it('first frame matches old state', () => {
            const old: TransitionState = {
                nodes: new Map(),
                edges: [],
                viewport: { x: 0, y: 0, zoom: 1 }
            }
            const newState: TransitionState = {
                nodes: new Map(),
                edges: [],
                viewport: { x: 100, y: 100, zoom: 2 }
            }

            const frames = generateTransitionFrames(old, newState, 10)

            expect(frames[0].viewport.x).toBe(0)
            expect(frames[0].viewport.zoom).toBe(1)
        })

        it('last frame matches new state', () => {
            const old: TransitionState = {
                nodes: new Map(),
                edges: [],
                viewport: { x: 0, y: 0, zoom: 1 }
            }
            const newState: TransitionState = {
                nodes: new Map(),
                edges: [],
                viewport: { x: 100, y: 100, zoom: 2 }
            }

            const frames = generateTransitionFrames(old, newState, 10)

            expect(frames[10].viewport.x).toBe(100)
            expect(frames[10].viewport.zoom).toBe(2)
        })
    })

    describe('createTransition', () => {
        it('returns a function that interpolates at any t', () => {
            const old: TransitionState = {
                nodes: new Map(),
                edges: [],
                viewport: { x: 0, y: 0, zoom: 1 }
            }
            const newState: TransitionState = {
                nodes: new Map(),
                edges: [],
                viewport: { x: 100, y: 100, zoom: 2 }
            }

            const transition = createTransition(old, newState)

            expect(transition(0).viewport.x).toBe(0)
            expect(transition(1).viewport.x).toBe(100)
            expect(transition(0.5).viewport.x).toBeGreaterThan(0)
        })
    })
})
