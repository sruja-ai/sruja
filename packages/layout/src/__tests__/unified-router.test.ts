import { describe, it, expect } from 'vitest'
import {
    routeL0,
    routeL1,
    routeL2,
    routeL3,
    routeEdge,
    type RouteNode,
    type RouteLane
} from '../algorithms/unified-router'

function makeNode(id: string, x: number, y: number, width = 200, height = 120): RouteNode {
    return { id, bbox: { x, y, width, height } }
}

describe('Unified Router', () => {
    describe('routeL0', () => {
        it('creates simple Manhattan route', () => {
            const sourcePort = { side: 'east' as const, position: { x: 200, y: 60 } }
            const targetPort = { side: 'west' as const, position: { x: 400, y: 260 } }

            const result = routeL0(sourcePort, targetPort, [], new Map())

            expect(result.points.length).toBeGreaterThanOrEqual(2)
            expect(result.points[0]).toEqual({ x: 200, y: 60 })
            expect(result.points[result.points.length - 1]).toEqual({ x: 400, y: 260 })
        })

        it('produces 1-2 bends max', () => {
            const sourcePort = { side: 'east' as const, position: { x: 200, y: 60 } }
            const targetPort = { side: 'west' as const, position: { x: 400, y: 260 } }

            const result = routeL0(sourcePort, targetPort, [], new Map())

            expect(result.bendCount).toBeLessThanOrEqual(2)
        })
    })

    describe('routeL1', () => {
        it('routes around boundary when present', () => {
            const sourcePort = { side: 'east' as const, position: { x: 200, y: 200 } }
            const targetPort = { side: 'west' as const, position: { x: 600, y: 200 } }
            const boundary = { bbox: { x: 300, y: 100, width: 200, height: 200 } }

            const result = routeL1(sourcePort, targetPort, [], new Map(), boundary)

            expect(result.points.length).toBeGreaterThan(2)
        })
    })

    describe('routeL2', () => {
        it('avoids obstacles', () => {
            const sourcePort = { side: 'east' as const, position: { x: 200, y: 200 } }
            const targetPort = { side: 'west' as const, position: { x: 600, y: 200 } }
            const obstacles = [makeNode('obstacle', 350, 150)]

            const result = routeL2(sourcePort, targetPort, obstacles)

            expect(result.points.length).toBeGreaterThan(2)
        })
    })

    describe('routeL3', () => {
        it('routes through lane corridors', () => {
            const sourcePort = { side: 'east' as const, position: { x: 200, y: 100 } }
            const targetPort = { side: 'west' as const, position: { x: 400, y: 300 } }
            const lanes: RouteLane[] = [
                { index: 0, y: 0, height: 200 },
                { index: 1, y: 200, height: 200 }
            ]

            const result = routeL3(sourcePort, targetPort, 0, 1, [], lanes)

            expect(result.points.length).toBeGreaterThan(2)
            expect(result.bendCount).toBeLessThanOrEqual(4)
        })
    })

    describe('routeEdge', () => {
        it('dispatches to correct level router', () => {
            const nodes = new Map([
                ['a', makeNode('a', 0, 0)],
                ['b', makeNode('b', 400, 0)]
            ])
            const edge = { id: 'e1', sourceId: 'a', targetId: 'b' }

            const resultL0 = routeEdge(edge, 'L0', nodes)
            const resultL2 = routeEdge(edge, 'L2', nodes)

            expect(resultL0).not.toBeNull()
            expect(resultL2).not.toBeNull()
            // L2 may have different routing due to obstacle avoidance logic
        })

        it('returns null for missing nodes', () => {
            const nodes = new Map([['a', makeNode('a', 0, 0)]])
            const edge = { id: 'e1', sourceId: 'a', targetId: 'missing' }

            const result = routeEdge(edge, 'L0', nodes)

            expect(result).toBeNull()
        })
    })
})
