import { describe, it, expect } from 'vitest'
import { layoutL0 } from '../l0-layout'
import { DefaultTheme } from '../../theme'
import { CachedTextMeasurer } from '../../utils/cached-text-measurer'
import { CanvasTextMeasurer } from '../../utils/canvas-text-measurer'
import type { C4LayoutOptions } from '../../c4-options'
import type { HierarchyNode } from '../hierarchy'
import type { SizedNode } from '../sizing'

describe('L0 Layout', () => {
    const dummyOptions: C4LayoutOptions = {
        strategy: 'grid',
        direction: 'TB',
        alignment: 'center',
        spacingMode: 'fixed',
        spacing: { node: {}, rank: {}, padding: {}, port: 10 },
        minSize: { width: 100, height: 100 },
        maxSize: { width: 1000, height: 1000 },
        aspectRatioLimits: { min: 1, max: 2 },
        edgeRouting: { algorithm: 'orthogonal', bendPenalty: 1, crossingPenalty: 10, segmentLength: 20, avoidNodes: false, preferOrthogonal: true },
        overlapRemoval: { enabled: false, algorithm: 'shift', iterations: 0, tolerance: 1, padding: 0 },
        beautify: { alignNodes: false, straightenEdges: true, balanceTree: false, compactGroups: false, removeOverlaps: false },
        maxIterations: 1,
        tolerance: 0.01,
        useGPU: false,
        measurer: new CachedTextMeasurer(new CanvasTextMeasurer()),
        theme: DefaultTheme
    }

    const mockHierarchyNode = (id: string, children: HierarchyNode[] = [], kind = 'SoftwareSystem'): HierarchyNode => ({
        id: id as any,
        node: { id: id as any, label: id, kind: kind as any, level: 'landscape', tags: new Set() },
        children,
        depth: 0,
        subtreeSize: 1,
        subtreeDepth: 0
    })

    const mockSizedNode = (id: string): SizedNode => ({
        id: id as any,
        node: { id: id as any, label: id, kind: 'SoftwareSystem', level: 'landscape', tags: new Set() },
        size: { width: 100, height: 100 },
        contentSize: { width: 100, height: 100 },
        labelLines: [id],
        depth: 0,
        subtreeSize: 1,
        subtreeDepth: 0,
        parent: undefined,
        children: []
    })

    it('compiles and runs layoutL0 for 4 systems', () => {
        const roots = [
            { id: 'SysA', size: { width: 300, height: 100 } },
            { id: 'SysB', size: { width: 300, height: 100 } },
            { id: 'SysC', size: { width: 300, height: 100 } },
            { id: 'SysD', size: { width: 300, height: 100 } },
        ] as any

        const hierarchy = new Map<string, HierarchyNode>()
        roots.forEach((r: any) => hierarchy.set(r.id, mockHierarchyNode(r.id)))
        const sized = new Map<string, SizedNode>()
        roots.forEach((r: any) => sized.set(r.id, mockSizedNode(r.id)))

        const result = layoutL0(roots, [], dummyOptions, (id) => hierarchy.get(id), (id) => sized.get(id))

        expect(result.nodes.length).toBe(4)
        // 4 items -> 2x2 grid
        // Row 0
        expect(result.nodes[0].y).toBe(0)
        expect(result.nodes[1].y).toBe(0)
        // Row 1
        expect(result.nodes[2].y).toBeGreaterThan(0)
        expect(result.nodes[3].y).toBeGreaterThan(0)
        expect(result.nodes[2].y).toEqual(result.nodes[3].y)
    })

    it('places container badges inside system', () => {
        const sysA = mockHierarchyNode('SysA')
        const container1 = mockHierarchyNode('Container1', [], 'Container')
        sysA.children.push(container1)

        const roots = [{ id: 'SysA', size: { width: 100, height: 100 } }] as any
        const hierarchy = new Map<string, HierarchyNode>()
        hierarchy.set('SysA', sysA)

        const sized = new Map<string, SizedNode>()
        sized.set('SysA', mockSizedNode('SysA'))
        sized.set('Container1', { ...mockSizedNode('Container1'), node: container1.node })

        const result = layoutL0(roots, [], dummyOptions, (id) => hierarchy.get(id), (id) => sized.get(id))

        expect(result.nodes.length).toBe(2) // SysA + Container1
        const sysNode = result.nodes.find(n => n.id === 'SysA')!
        const badgeNode = result.nodes.find(n => n.id === 'Container1')!

        expect(badgeNode.x).toBeGreaterThan(sysNode.x)
        expect(badgeNode.y).toBeGreaterThan(sysNode.y)
        // Check contained
        expect(badgeNode.x).toBeLessThan(sysNode.x + sysNode.size.width)
        expect(badgeNode.y).toBeLessThan(sysNode.y + sysNode.size.height)
    })
})
