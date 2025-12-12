import { describe, it, expect } from 'vitest'
import { layoutL1SystemContext } from '../l1-layout'
import type { HierarchyNode, HierarchyTree } from '../hierarchy'

import type { C4Node } from '../../c4-model'
import { InteractivePreset } from '../../c4-options'

// Helper to create a HierarchyNode
function createNode(id: string, kind: string): HierarchyNode {
    return {
        id: id as any,
        node: { id: id as any, kind: kind as any } as C4Node,
        children: [], // L1 ignores children details for layout usually
        depth: 0,
        subtreeSize: 1,
        subtreeDepth: 0
    }
}

describe('layoutL1SystemContext', () => {
    it('places a single system in the center', () => {
        const system = createNode('sys', 'SoftwareSystem')
        const tree: HierarchyTree = {
            roots: [system],
            nodeMap: new Map([['sys', system]] as any),
            maxDepth: 0
        }
        const sizes = new Map([['sys', { width: 200, height: 100 }]])

        const result = layoutL1SystemContext(tree, sizes, [], InteractivePreset)

        expect(result.size).toBe(1)
        const pos = result.get('sys')!
        // Center cluster logic centers the group around 0,0
        expect(pos.bbox.x).toBe(-100) // -width/2
        expect(pos.bbox.y).toBe(-50)  // -height/2
    })

    it('places satellites around the central system', () => {
        const system = createNode('sys', 'SoftwareSystem')
        const user = createNode('user', 'Person')
        const extSys = createNode('ext', 'ExternalSystem')

        const tree: HierarchyTree = {
            roots: [system, user, extSys],
            nodeMap: new Map(), // not strictly needed for the algo logic used so far
            maxDepth: 0
        }

        const sizes = new Map([
            ['sys', { width: 200, height: 100 }],
            ['user', { width: 100, height: 100 }],
            ['ext', { width: 150, height: 100 }]
        ])

        const result = layoutL1SystemContext(tree, sizes, [], InteractivePreset)

        expect(result.size).toBe(3)

        const pSys = result.get('sys')!
        const pUser = result.get('user')!
        const pExt = result.get('ext')!

        // System should be central
        expect(pSys.bbox.x).toBe(-100)
        expect(pSys.bbox.y).toBe(-50)

        // External nodes should be further out
        // Distance from center 0,0
        const distUser = Math.hypot(pUser.bbox.x + 50, pUser.bbox.y + 50)
        const distExt = Math.hypot(pExt.bbox.x + 75, pExt.bbox.y + 50)

        expect(distUser).toBeGreaterThan(150)
        expect(distExt).toBeGreaterThan(150)

        // They should be at different angles/positions
        expect(pUser.bbox.x).not.toBe(pExt.bbox.x)
        expect(pUser.bbox.y).not.toBe(pExt.bbox.y)
    })

    it('falls back to circle layout if no clear center', () => {
        const n1 = createNode('n1', 'Person')
        const n2 = createNode('n2', 'Person')

        const tree: HierarchyTree = {
            roots: [n1, n2],
            nodeMap: new Map(),
            maxDepth: 0
        }
        const sizes = new Map([
            ['n1', { width: 100, height: 100 }],
            ['n2', { width: 100, height: 100 }]
        ])

        const result = layoutL1SystemContext(tree, sizes, [], InteractivePreset)

        expect(result.size).toBe(2)
        // Should be symmetric
        const p1 = result.get('n1')!
        const p2 = result.get('n2')!

        const d1 = Math.hypot(p1.bbox.x + 50, p1.bbox.y + 50)
        const d2 = Math.hypot(p2.bbox.x + 50, p2.bbox.y + 50)

        expect(Math.abs(d1 - d2)).toBeLessThan(1) // Should be equidistant from center
    })

    it('maintains deterministic satellite angles when center expands', () => {
        const system = createNode('sys', 'SoftwareSystem')
        const ext1 = createNode('ext1', 'ExternalSystem')
        const ext2 = createNode('ext2', 'ExternalSystem')
        const person = createNode('person', 'Person')

        const roots = [system, ext1, ext2, person]
        const tree: HierarchyTree = { roots, nodeMap: new Map(), maxDepth: 0 }

        // Initial small size
        const sizesSmall = new Map([
            ['sys', { width: 200, height: 100 }],
            ['ext1', { width: 100, height: 100 }],
            ['ext2', { width: 100, height: 100 }],
            ['person', { width: 100, height: 100 }]
        ])

        const resSmall = layoutL1SystemContext(tree, sizesSmall, [], InteractivePreset)

        // Expanded large size
        const sizesBig = new Map([
            ['sys', { width: 800, height: 600 }], // Expanded
            ['ext1', { width: 100, height: 100 }],
            ['ext2', { width: 100, height: 100 }],
            ['person', { width: 100, height: 100 }]
        ])

        const resBig = layoutL1SystemContext(tree, sizesBig, [], InteractivePreset)

        // Verify angles are identical (relative order preserved)
        const getAngle = (res: Map<string, any>, id: string) => {
            const p = res.get(id)!
            return Math.atan2(p.bbox.y + p.bbox.height / 2, p.bbox.x + p.bbox.width / 2)
        }

        const angleS1 = getAngle(resSmall, 'ext1')
        const angleS2 = getAngle(resSmall, 'ext2')
        const angleSP = getAngle(resSmall, 'person')

        const angleB1 = getAngle(resBig, 'ext1')
        const angleB2 = getAngle(resBig, 'ext2')
        const angleBP = getAngle(resBig, 'person')

        // Angles should be EXACTLY the same because sort is deterministic and count is same
        expect(angleB1).toBeCloseTo(angleS1, 4)
        expect(angleB2).toBeCloseTo(angleS2, 4)
        expect(angleBP).toBeCloseTo(angleSP, 4)
    })
})
