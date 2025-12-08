import { describe, it, expect } from 'vitest'
import { layoutSugiyama } from '../algorithms/sugiyama'
import { InteractivePreset } from '../c4-options'
import { createC4Id } from '../brand'

describe('Sugiyama Layout', () => {
    const options = InteractivePreset

    it('layouts single node', () => {
        const nodes = [{ id: createC4Id('a'), size: { width: 100, height: 50 } }]
        const rels: any[] = []
        const result = layoutSugiyama(nodes, rels, options)

        expect(result.width).toBe(100)
        expect(result.height).toBe(50)
        expect(result.nodes.get(createC4Id('a'))!.x).toBe(0)
        expect(result.nodes.get(createC4Id('a'))!.y).toBe(0)
    })

    it('layouts two connected nodes (vertical)', () => {
        const a = createC4Id('a')
        const b = createC4Id('b')
        const nodes = [
            { id: a, size: { width: 100, height: 50 } },
            { id: b, size: { width: 100, height: 50 } }
        ]
        const rels = [{ from: a, to: b }]

        const result = layoutSugiyama(nodes, rels, options)

        const nodeA = result.nodes.get(a)!
        const nodeB = result.nodes.get(b)!

        // A should be above B
        expect(nodeB.y).toBeGreaterThan(nodeA.y + nodeA.size.height)
        expect(nodeA.layer).toBe(0)
        expect(nodeB.layer).toBe(1)

        // Width should be max width (100)
        expect(result.width).toBe(100)
    })

    it('layouts complex hierarchy', () => {
        // A -> B -> C
        // A -> D
        const [a, b, c, d] = ['a', 'b', 'c', 'd'].map(createC4Id)
        const nodes = [a, b, c, d].map(id => ({ id, size: { width: 100, height: 50 } }))
        const rels = [
            { from: a, to: b },
            { from: b, to: c },
            { from: a, to: d }
        ]

        const result = layoutSugiyama(nodes, rels, options)

        expect(result.nodes.get(a)!.layer).toBe(0)
        expect(result.nodes.get(b)!.layer).toBe(1)
        expect(result.nodes.get(d)!.layer).toBe(1)
        expect(result.nodes.get(c)!.layer).toBe(2)

        // B and D should be on same layer (Y usually same/close depending on heights)
        // Heights are same (50). So Y should be exact.
        const yB = result.nodes.get(b)!.y
        const yD = result.nodes.get(d)!.y
        expect(yB).toBe(yD)
    })
})
