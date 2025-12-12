import { describe, it, expect } from 'vitest'
import {
    layoutL1SystemContext,
    layoutL2Containers,
    layoutL3Components,
    type C4LayoutNode
} from '../algorithms/c4-level-layouts'
import { C4Id } from '../brand'
import { NODE_WIDTH, NODE_HEIGHT } from '../constants'

function makeNode(id: string, laneHint?: C4LayoutNode['laneHint']): C4LayoutNode {
    return {
        id: id as C4Id,
        size: { width: NODE_WIDTH, height: NODE_HEIGHT },
        laneHint
    }
}

function makeEdge(from: string, to: string) {
    return { source: from, target: to }
}

describe('C4 Level Layouts', () => {
    describe('layoutL1SystemContext', () => {
        it('places central system at canvas center', () => {
            const central = makeNode('main-system')
            const result = layoutL1SystemContext(central, [])

            const pos = result.positions.get('main-system' as C4Id)!
            expect(pos).toBeDefined()
            // Should be centered at default canvas center (600, 400)
            expect(pos.x).toBe(600 - NODE_WIDTH / 2)
            expect(pos.y).toBe(400 - NODE_HEIGHT / 2)
        })

        it('distributes externals on 4 sides', () => {
            const central = makeNode('main-system')
            const externals = [
                makeNode('ext-1'),
                makeNode('ext-2'),
                makeNode('ext-3'),
                makeNode('ext-4')
            ]

            const result = layoutL1SystemContext(central, externals)

            // All 5 nodes should be positioned
            expect(result.positions.size).toBe(5)

            // Each external should be at different position
            const positions = [...result.positions.values()]
            const uniquePositions = new Set(positions.map(p => `${p.x},${p.y}`))
            expect(uniquePositions.size).toBe(5)
        })

        it('handles many externals', () => {
            const central = makeNode('main-system')
            const externals = Array.from({ length: 12 }, (_, i) => makeNode(`ext-${i}`))

            const result = layoutL1SystemContext(central, externals)

            expect(result.positions.size).toBe(13)
        })
    })

    describe('layoutL2Containers', () => {
        it('places containers in grid inside boundary', () => {
            const containers = [
                makeNode('container-1'),
                makeNode('container-2'),
                makeNode('container-3')
            ]

            const result = layoutL2Containers(containers, [])

            expect(result.positions.size).toBe(3)
            expect(result.boundarySize).toBeDefined()

            // 3 containers should be in 1 layer if no edges
            const pos1 = result.positions.get('container-1' as C4Id)!
            const pos2 = result.positions.get('container-2' as C4Id)!
            const pos3 = result.positions.get('container-3' as C4Id)!

            // All same Y (same layer, same height)
            expect(pos1.y).toBe(pos2.y)
            expect(pos2.y).toBe(pos3.y)

            // Different X (different slots)
            expect(pos1.x).not.toBe(pos2.x)
            expect(pos2.x).not.toBe(pos3.x)
        })

        it('calculates correct boundary size', () => {
            const containers = [
                makeNode('c1'),
                makeNode('c2'),
                makeNode('c3'),
                makeNode('c4')
            ]

            const result = layoutL2Containers(containers, [], [], { x: 0, y: 0 })

            // 4 containers → 3 columns, 2 rows (based on grid logic)
            expect(result.boundarySize!.width).toBeGreaterThan(0)
            expect(result.boundarySize!.height).toBeGreaterThan(0)
        })

        it('positions externals around the boundary', () => {
            const containers = [makeNode('c1'), makeNode('c2')]
            const externals = [makeNode('ext-db'), makeNode('ext-api')]

            const result = layoutL2Containers(containers, externals)

            expect(result.positions.size).toBe(4)
        })
    })

    describe('layoutL3Components', () => {
        it('orders components based on dependencies (flow)', () => {
            const components = [
                makeNode('UserController'),
                makeNode('UserService'),
                makeNode('UserRepository')
            ]

            // Controller -> Service -> Repository
            const edges = [
                makeEdge('UserController', 'UserService'),
                makeEdge('UserService', 'UserRepository')
            ]

            const result = layoutL3Components(components, [], edges)

            expect(result.positions.size).toBe(3)

            const pos1 = result.positions.get('UserController' as C4Id)!
            const pos2 = result.positions.get('UserService' as C4Id)!
            const pos3 = result.positions.get('UserRepository' as C4Id)!

            // Flow should be Top -> Down
            // pos2 below pos1
            expect(pos2.y).toBeGreaterThan(pos1.y)
            // pos3 below pos2
            expect(pos3.y).toBeGreaterThan(pos2.y)
        })

        it('places unconnected components in same layer', () => {
            const components = [
                makeNode('Comp1'),
                makeNode('Comp2')
            ]

            const result = layoutL3Components(components, [])

            const pos1 = result.positions.get('Comp1' as C4Id)!
            const pos2 = result.positions.get('Comp2' as C4Id)!

            // Same Y
            expect(pos1.y).toBe(pos2.y)
        })

        it('handles empty components array', () => {
            const result = layoutL3Components([], [])

            expect(result.positions.size).toBe(0)
            expect(result.boundarySize).toEqual({ width: 0, height: 0 })
        })
    })

    describe('layout determinism', () => {
        it('produces identical results for same input', () => {
            const central = makeNode('system')
            const externals = [makeNode('ext-1'), makeNode('ext-2')]

            const result1 = layoutL1SystemContext(central, externals)
            const result2 = layoutL1SystemContext(central, externals)

            expect([...result1.positions.entries()]).toEqual([...result2.positions.entries()])
        })

        it('produces identical results for same input (L3)', () => {
            const components = [makeNode('comp-1'), makeNode('comp-2')]
            const result1 = layoutL3Components(components, [])
            const result2 = layoutL3Components(components, [])
            expect(result1).toStrictEqual(result2)
        })

        it('handles variable sized nodes with center alignment in layer', () => {
            const largeComponents: C4LayoutNode[] = [
                { id: 'c1' as C4Id, size: { width: 100, height: 100 } },
                { id: 'c2' as C4Id, size: { width: 100, height: 300 } }, // Tall node
                { id: 'c3' as C4Id, size: { width: 100, height: 100 } }
            ]

            // No edges, so single layer.
            // Sugiyama aligns centers vertically in the layer.
            // Max height = 300.
            // C2 (300) y = 0 relative to layer top.
            // C1 (100) y = (300 - 100) / 2 = 100.

            const result = layoutL2Containers(largeComponents, [])

            const c1 = result.positions.get('c1' as C4Id)!
            const c2 = result.positions.get('c2' as C4Id)!
            const c3 = result.positions.get('c3' as C4Id)!

            // C1 should be centered relative to C2
            expect(c1.y).toBeGreaterThan(c2.y)
            expect(c3.y).toBeGreaterThan(c2.y)

            // C1 and C3 should be at same Y
            expect(c1.y).toBe(c3.y)
        })
    })
})
