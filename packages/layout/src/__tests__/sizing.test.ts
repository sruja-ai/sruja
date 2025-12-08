import { describe, it, expect } from 'vitest'
import { MockTextMeasurer2 } from '../utils/text-measurer'
import { calculateSizes } from '../algorithms/sizing'
import { InteractivePreset } from '../c4-options'
import { createC4Id } from '../brand'
import { HierarchyTree } from '../algorithms/hierarchy'

describe('Sizing', () => {
  it('computes size for hierarchy nodes', () => {
    const measurer = new MockTextMeasurer2()
    const id = createC4Id('node1')
    const tree: HierarchyTree = {
      roots: [{ id, node: { kind: 'SoftwareSystem', level: 'context', label: 'hello world' } as any, children: [] } as any],
      nodeMap: new Map(),
      maxDepth: 0
    }

    // Add to nodeMap as calculateSizes might iterate it if implementation changes (currently iterates roots)
    // Actually current calculateSizes iterates roots.

    const sizes = calculateSizes(tree, [], measurer, InteractivePreset)
    const size = sizes.get(id)?.size

    expect(size).toBeDefined()
    expect(size!.width).toBeGreaterThan(0)
    expect(size!.height).toBeGreaterThan(0)
  })
})
