import { describe, it, expect } from 'vitest'
import { createC4Id } from '../brand'
import { createC4Graph } from '../c4-model'
import { layout } from '../c4-layout'
import { createDefaultViewState } from '../c4-view'

describe('Pipeline layout', () => {
  it('produces positioned nodes and edges for a simple graph', () => {
    const person = { id: createC4Id('PersonA'), label: 'User', kind: 'Person', level: 'context', tags: new Set<string>() }
    const system = { id: createC4Id('SystemA'), label: 'Ecommerce', kind: 'SoftwareSystem', level: 'context', tags: new Set<string>() }
    const rel = { id: 'PersonA->SystemA', from: person.id, to: system.id, interaction: 'sync' as const }
    const graph = createC4Graph([person as any, system as any], [rel as any])
    const result = layout(graph, createDefaultViewState())
    expect(result.nodes.size).toBe(2)
    expect(result.relationships.length).toBe(1)
    const edge = result.relationships[0]
    expect(edge.points.length).toBeGreaterThan(1)
    expect(result.bbox.width).toBeGreaterThan(0)
    expect(result.metrics.totalEdgeLength).toBeGreaterThan(0)
  })
})
