import { describe, it, expect } from 'vitest'
import { createC4Id } from '../brand'
import { createC4Graph } from '../c4-model'
import { layout } from '../c4-layout'
import { createDefaultViewState } from '../c4-view'
import { CompactPreset } from '../c4-options'

describe('Overlap removal', () => {
  it('reduces overlaps when spacing is small', () => {
    const a = { id: createC4Id('A'), label: 'A', kind: 'SoftwareSystem', level: 'context', tags: new Set<string>() }
    const b = { id: createC4Id('B'), label: 'B', kind: 'SoftwareSystem', level: 'context', tags: new Set<string>() }
    const graph = createC4Graph([a as any, b as any], [])
    const options = { ...CompactPreset, overlapRemoval: { ...CompactPreset.overlapRemoval, enabled: true, padding: 10, iterations: 5 } }
    const res = layout(graph, createDefaultViewState(), options as any)
    const nodes = [...res.nodes.values()]
    const r1 = nodes[0].bbox
    const r2 = nodes[1].bbox
    const overlap = !(r1.x + r1.width + 0 < r2.x || r2.x + r2.width + 0 < r1.x || r1.y + r1.height + 0 < r2.y || r2.y + r2.height + 0 < r1.y)
    expect(overlap).toBe(false)
  })
})
