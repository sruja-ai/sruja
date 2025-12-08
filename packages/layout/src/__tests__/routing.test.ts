import { describe, it, expect } from 'vitest'
import { createC4Id } from '../brand'
import { createC4Graph } from '../c4-model'
import { layout } from '../c4-layout'
import { createDefaultViewState } from '../c4-view'
import { InteractivePreset } from '../c4-options'

describe('Obstacle-aware routing', () => {
  it('routes orthogonally around obstacles', () => {
    const s = { id: createC4Id('S'), label: 'Source', kind: 'SoftwareSystem', level: 'context', tags: new Set<string>() }
    const t = { id: createC4Id('T'), label: 'Target', kind: 'SoftwareSystem', level: 'context', tags: new Set<string>() }
    const o = { id: createC4Id('O'), label: 'Obstacle', kind: 'SoftwareSystem', level: 'context', tags: new Set<string>() }
    const rel = { id: 'S->T', from: s.id, to: t.id }
    const graph = createC4Graph([s as any, t as any, o as any], [rel as any])
    const res = layout(graph, createDefaultViewState(), InteractivePreset)
    const edge = res.relationships[0]
    const obstacle = [...res.nodes.values()].find(n => n.nodeId === 'O')!
    const passesThrough = edge.points.some(p => p.x >= obstacle.bbox.x && p.x <= obstacle.bbox.x + obstacle.bbox.width && p.y >= obstacle.bbox.y && p.y <= obstacle.bbox.y + obstacle.bbox.height)
    expect(passesThrough).toBe(false)
  })
})
