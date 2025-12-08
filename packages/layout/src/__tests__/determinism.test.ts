import { describe, it, expect } from 'vitest'
import { createC4Id } from '../brand'
import { createC4Graph } from '../c4-model'
import { layout } from '../c4-layout'
import { createDefaultViewState } from '../c4-view'

describe('Determinism', () => {
  it('same input yields same layout', () => {
    const a = { id: createC4Id('A'), label: 'A', kind: 'SoftwareSystem', level: 'context', tags: new Set<string>() }
    const b = { id: createC4Id('B'), label: 'B', kind: 'SoftwareSystem', level: 'context', tags: new Set<string>() }
    const c = { id: createC4Id('C'), label: 'C', kind: 'SoftwareSystem', level: 'context', tags: new Set<string>() }
    const rels = [
      { id: 'A->B', from: a.id, to: b.id },
      { id: 'B->C', from: b.id, to: c.id },
      { id: 'A->C', from: a.id, to: c.id }
    ]
    const graph = createC4Graph([a as any, b as any, c as any], rels as any)
    const view = createDefaultViewState()
    const r1 = layout(graph, view)
    const r2 = layout(graph, view)
    expect(JSON.stringify([...r1.nodes.entries()])).toEqual(JSON.stringify([...r2.nodes.entries()]))
    expect(JSON.stringify(r1.relationships)).toEqual(JSON.stringify(r2.relationships))
  })
})
