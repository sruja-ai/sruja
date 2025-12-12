import { describe, it, expect } from 'vitest'
import {
    removeOverlapsBottomUp,
    distributeSpaceTopDown,
    optimizeForEdges,
    applyMultiPassOptimization
} from '../optimizer'
import type { PositionedNode } from '../coordinates'
import type { HierarchyNode, HierarchyTree } from '../hierarchy'
import type { OptimizationOptions } from '../optimizer'
import type { C4Relationship } from '../../c4-model'
import type { C4Id } from '../../brand'

function createHierarchyNode(id: string, depth: number = 0, children: HierarchyNode[] = []): HierarchyNode {
    return {
        id: id as C4Id,
        node: {
            id: id as C4Id,
            kind: 'SoftwareSystem',
            label: id,
            level: 'landscape',
            tags: new Set()
        },
        children,
        depth,
        subtreeSize: 1 + children.length,
        subtreeDepth: children.length > 0 ? Math.max(...children.map(c => c.subtreeDepth)) + 1 : 0,
        parent: undefined
    }
}

function createPositionedNode(id: string, x: number, y: number, width: number = 100, height: number = 100): PositionedNode {
    return {
        id: id as C4Id,
        node: {
            id: id as C4Id,
            kind: 'Component',
            label: id,
            level: 'component',
            tags: new Set()
        },
        x,
        y,
        bbox: { x, y, width, height },
        size: { width, height },
        contentSize: { width: 80, height: 80 },
        labelLines: [id],
        depth: 1,
        subtreeSize: 1,
        subtreeDepth: 0,
        children: [],
        parent: undefined
    }
}

describe('Optimizer', () => {
    describe('removeOverlapsBottomUp', () => {
        it('detects and fixes overlapping siblings', () => {
            const parent = createHierarchyNode('parent', 0)
            const child1 = createHierarchyNode('child1', 1)
            const child2 = createHierarchyNode('child2', 1)
            parent.children = [child1, child2]

            const tree: HierarchyTree = {
                roots: [parent],
                nodeMap: new Map([[parent.id, parent], [child1.id, child1], [child2.id, child2]]) as any,
                maxDepth: 1
            }

            const positioned = new Map<C4Id, PositionedNode>([
                ['parent' as C4Id, createPositionedNode('parent', 0, 0, 300, 300)],
                ['child1' as C4Id, createPositionedNode('child1', 10, 10)],  // Overlapping
                ['child2' as C4Id, createPositionedNode('child2', 20, 20)]   // with child1
            ])

            const options: OptimizationOptions = {
                enabled: true,
                overlapRemoval: { iterations: 5, padding: 16 }
            }

            const result = removeOverlapsBottomUp(positioned, tree, options)

            const c1 = result.get('child1' as C4Id)!
            const c2 = result.get('child2' as C4Id)!

            // After overlap removal, nodes should not overlap
            const noOverlap =
                c1.bbox.x + c1.bbox.width + 16 <= c2.bbox.x ||
                c2.bbox.x + c2.bbox.width + 16 <= c1.bbox.x ||
                c1.bbox.y + c1.bbox.height + 16 <= c2.bbox.y ||
                c2.bbox.y + c2.bbox.height + 16 <= c1.bbox.y

            expect(noOverlap).toBe(true)
        })

        it('resizes parent to fit children', () => {
            const parent = createHierarchyNode('parent', 0)
            const child = createHierarchyNode('child', 1)
            parent.children = [child]

            const tree: HierarchyTree = {
                roots: [parent],
                nodeMap: new Map([[parent.id, parent], [child.id, child]]) as any,
                maxDepth: 1
            }

            const positioned = new Map<C4Id, PositionedNode>([
                ['parent' as C4Id, createPositionedNode('parent', 0, 0, 150, 150)],  // Too small
                ['child' as C4Id, createPositionedNode('child', 10, 50, 100, 100)]   // Child needs more space
            ])

            const options: OptimizationOptions = {
                enabled: true,
                overlapRemoval: { iterations: 5, padding: 16 }
            }

            const result = removeOverlapsBottomUp(positioned, tree, options)

            const p = result.get('parent' as C4Id)!
            const c = result.get('child' as C4Id)!

            // Parent should have expanded to fit child
            expect(p.bbox.width).toBeGreaterThanOrEqual(c.bbox.width + 32) // padding * 2
            expect(p.bbox.height).toBeGreaterThanOrEqual(c.bbox.height + 32)
        })
    })

    describe('distributeSpaceTopDown', () => {
        it('redistributes extra space among children', () => {
            const parent = createHierarchyNode('parent', 0)
            const child1 = createHierarchyNode('child1', 1)
            const child2 = createHierarchyNode('child2', 1)
            parent.children = [child1, child2]

            const tree: HierarchyTree = {
                roots: [parent],
                nodeMap: new Map([[parent.id, parent], [child1.id, child1], [child2.id, child2]]) as any,
                maxDepth: 1
            }

            const positioned = new Map<C4Id, PositionedNode>([
                ['parent' as C4Id, createPositionedNode('parent', 0, 0, 500, 500)],  // Lots of space
                ['child1' as C4Id, createPositionedNode('child1', 10, 10, 50, 50)],
                ['child2' as C4Id, createPositionedNode('child2', 10, 70, 50, 50)]
            ])

            const options: OptimizationOptions = {
                enabled: true,
                spaceDistribution: { enabled: true, minThreshold: 50 }
            }

            const originalC1 = positioned.get('child1' as C4Id)!
            const originalC2 = positioned.get('child2' as C4Id)!

            const result = distributeSpaceTopDown(positioned, tree, options)

            const c1 = result.get('child1' as C4Id)!
            const c2 = result.get('child2' as C4Id)!

            // Children should have more space between them after distribution
            const originalGap = Math.abs(originalC2.y - (originalC1.y + originalC1.bbox.height))
            const newGap = Math.abs(c2.y - (c1.y + c1.bbox.height))

            expect(newGap).toBeGreaterThanOrEqual(originalGap)
        })
    })

    describe('optimizeForEdges', () => {
        it('groups nodes into layers by depth', () => {
            const root = createHierarchyNode('root', 0)
            const child1 = createHierarchyNode('child1', 1)
            const child2 = createHierarchyNode('child2', 1)
            const grandchild = createHierarchyNode('grandchild', 2)

            child1.children = [grandchild]
            root.children = [child1, child2]

            const tree: HierarchyTree = {
                roots: [root],
                nodeMap: new Map([
                    [root.id, root],
                    [child1.id, child1],
                    [child2.id, child2],
                    [grandchild.id, grandchild]
                ]) as any,
                maxDepth: 2
            }

            const positioned = new Map<C4Id, PositionedNode>([
                ['root' as C4Id, createPositionedNode('root', 0, 0, 300, 300)],
                ['child1' as C4Id, createPositionedNode('child1', 10, 10)],
                ['child2' as C4Id, createPositionedNode('child2', 120, 10)],
                ['grandchild' as C4Id, createPositionedNode('grandchild', 20, 120)]
            ])

            const relationships: C4Relationship[] = [
                { id: 'e1', from: 'child1' as C4Id, to: 'child2' as C4Id }
            ]

            const options: OptimizationOptions = {
                enabled: true,
                edgeOptimization: { enabled: true, minimizeCrossings: true }
            }

            // Should not throw
            const result = optimizeForEdges(positioned, relationships, tree, options)
            expect(result).toBeDefined()
            expect(result.size).toBeGreaterThan(0)
        })
    })

    describe('applyMultiPassOptimization', () => {
        it('applies all optimization passes in sequence', () => {
            const parent = createHierarchyNode('parent', 0)
            const child1 = createHierarchyNode('child1', 1)
            const child2 = createHierarchyNode('child2', 1)
            parent.children = [child1, child2]

            const tree: HierarchyTree = {
                roots: [parent],
                nodeMap: new Map([[parent.id, parent], [child1.id, child1], [child2.id, child2]]) as any,
                maxDepth: 1
            }

            const positioned = new Map<C4Id, PositionedNode>([
                ['parent' as C4Id, createPositionedNode('parent', 0, 0, 300, 300)],
                ['child1' as C4Id, createPositionedNode('child1', 10, 10)],
                ['child2' as C4Id, createPositionedNode('child2', 15, 15)]  // Slightly overlapping
            ])

            const relationships: C4Relationship[] = []

            const options: OptimizationOptions = {
                enabled: true,
                overlapRemoval: { iterations: 5, padding: 16 },
                spaceDistribution: { enabled: true, minThreshold: 50 },
                edgeOptimization: { enabled: true, minimizeCrossings: true }
            }

            const result = applyMultiPassOptimization(positioned, relationships, tree, options)

            expect(result).toBeDefined()
            expect(result.size).toBe(positioned.size)

            // All nodes should still exist
            expect(result.has('parent' as C4Id)).toBe(true)
            expect(result.has('child1' as C4Id)).toBe(true)
            expect(result.has('child2' as C4Id)).toBe(true)
        })

        it('skips optimization when disabled', () => {
            const positioned = new Map<C4Id, PositionedNode>([
                ['node1' as C4Id, createPositionedNode('node1', 0, 0)]
            ])

            const tree: HierarchyTree = {
                roots: [],
                nodeMap: new Map(),
                maxDepth: 0
            }

            const options: OptimizationOptions = {
                enabled: false
            }

            const result = applyMultiPassOptimization(positioned, [], tree, options)

            // Should return unmodified
            expect(result).toBe(positioned)
        })
    })
})
