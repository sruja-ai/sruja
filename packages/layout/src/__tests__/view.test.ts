import { describe, it, expect } from 'vitest'
import { createC4Id } from '../brand'
import { createC4Graph } from '../c4-model'
import { layout } from '../c4-layout'
import { SystemContextView, ContainerView } from '../c4-view'

describe('View presets', () => {
  it('SystemContextView focuses system and includes people/external systems', () => {
    const sys = { id: createC4Id('Sys'), label: 'Sys', kind: 'SoftwareSystem', level: 'context', tags: new Set<string>() }
    const person = { id: createC4Id('User'), label: 'User', kind: 'Person', level: 'context', tags: new Set<string>() }
    const container = { id: createC4Id('Api'), label: 'API', kind: 'Container', level: 'container', parentId: sys.id, tags: new Set<string>() }
    const rel = { id: 'User->Sys', from: person.id, to: sys.id }
    const graph = createC4Graph([sys as any, person as any, container as any], [rel as any])
    const result = layout(graph, SystemContextView(sys.id))
    const ids = new Set([...result.nodes.keys()])
    expect(ids.has('Sys')).toBe(true)
    expect(ids.has('User')).toBe(true)
    expect(ids.has('Api')).toBe(false)
  })

  it('ContainerView expands containers of system', () => {
    const sys = { id: createC4Id('Sys'), label: 'Sys', kind: 'SoftwareSystem', level: 'context', tags: new Set<string>() }
    const container = { id: createC4Id('Api'), label: 'API', kind: 'Container', level: 'container', parentId: sys.id, tags: new Set<string>() }
    const db = { id: createC4Id('DB'), label: 'DB', kind: 'Database', level: 'container', parentId: sys.id, tags: new Set<string>() }
    const graph = createC4Graph([sys as any, container as any, db as any], [])
    const result = layout(graph, ContainerView(sys.id))
    const ids = new Set([...result.nodes.keys()])
    expect(ids.has('Sys')).toBe(true)
    expect(ids.has('Api')).toBe(true)
    expect(ids.has('DB')).toBe(true)
  })
})
