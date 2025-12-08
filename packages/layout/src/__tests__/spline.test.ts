import { describe, it, expect } from 'vitest'
import { createC4Id } from '../brand'
import { createC4Graph } from '../c4-model'
import { layout } from '../c4-layout'
import { createDefaultViewState } from '../c4-view'
import { InteractivePreset } from '../c4-options'

describe('Spline routing', () => {
  it('produces control points for curved edges', () => {
    const a = { id: createC4Id('A'), label: 'A', kind: 'SoftwareSystem', level: 'context', tags: new Set<string>() }
    const b = { id: createC4Id('B'), label: 'B', kind: 'SoftwareSystem', level: 'context', tags: new Set<string>() }
    const rel = { id: 'A->B', from: a.id, to: b.id, preferredRoute: 'splines' as const }
    const graph = createC4Graph([a as any, b as any], [rel as any])
    const res = layout(graph, createDefaultViewState(), InteractivePreset)
    const e = res.relationships[0]
    expect(e.controlPoints && e.controlPoints.length).toBeGreaterThan(0)
    expect(e.segmentTypes[0]).toBe('arc')
  })
})
