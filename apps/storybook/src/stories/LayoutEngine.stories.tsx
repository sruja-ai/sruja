import type { Meta, StoryObj } from '@storybook/react'
import { expect, within } from 'storybook/test'
import React from 'react'
import { LayoutSVG } from '../components/LayoutSVG'
import { createC4Id, createC4Graph, layout, SystemContextView, ContainerView, InteractivePreset, PublicationPreset } from '@sruja/layout'

const meta: Meta<typeof LayoutSVG> = {
  title: 'Examples/Layout Engine',
  component: LayoutSVG,
}

export default meta
type Story = StoryObj<typeof LayoutSVG>

function buildSimpleContext() {
  const sys = { id: createC4Id('Sys'), label: 'Sys', kind: 'SoftwareSystem', level: 'context', tags: new Set<string>() }
  const user = { id: createC4Id('User'), label: 'User', kind: 'Person', level: 'context', tags: new Set<string>() }
  const ext = { id: createC4Id('Payments'), label: 'Payments', kind: 'ExternalSystem', level: 'context', tags: new Set<string>() }
  const rels = [{ id: 'User->Sys', from: user.id, to: sys.id }, { id: 'Sys->Payments', from: sys.id, to: ext.id }]
  return createC4Graph([sys as any, user as any, ext as any], rels as any)
}

function buildContainers() {
  const sys = { id: createC4Id('Sys'), label: 'Sys', kind: 'SoftwareSystem', level: 'context', tags: new Set<string>() }
  const api = { id: createC4Id('API'), label: 'API', kind: 'Container', level: 'container', parentId: sys.id, tags: new Set<string>() }
  const web = { id: createC4Id('Web'), label: 'Web', kind: 'Container', level: 'container', parentId: sys.id, tags: new Set<string>() }
  const db = { id: createC4Id('DB'), label: 'DB', kind: 'Database', level: 'container', parentId: sys.id, tags: new Set<string>() }
  const rels = [{ id: 'Web->API', from: web.id, to: api.id }, { id: 'API->DB', from: api.id, to: db.id }]
  return createC4Graph([sys as any, api as any, web as any, db as any], rels as any)
}

export const SystemContext: Story = {
  render: () => {
    const graph = buildSimpleContext()
    const result = layout(graph, SystemContextView(createC4Id('Sys')), InteractivePreset)
    return <LayoutSVG result={result} />
  }
  ,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    const svg = canvasElement.querySelector('svg') as SVGSVGElement | null
    expect(svg).toBeTruthy()
    expect(svg?.querySelectorAll('rect').length || 0).toBeGreaterThan(0)
  }
}

export const ContainerViewStory: Story = {
  render: () => {
    const graph = buildContainers()
    const result = layout(graph, ContainerView(createC4Id('Sys')), PublicationPreset)
    return <LayoutSVG result={result} />
  }
}

export const CurvedEdges: Story = {
  render: () => {
    const graph = buildSimpleContext()
    const edges = [{ id: 'User->Sys', from: createC4Id('User'), to: createC4Id('Sys'), preferredRoute: 'splines' as const }, { id: 'Sys->Payments', from: createC4Id('Sys'), to: createC4Id('Payments'), preferredRoute: 'splines' as const }]
    const result = layout(graph, SystemContextView(createC4Id('Sys')), { ...InteractivePreset, edgeRouting: { ...InteractivePreset.edgeRouting, algorithm: 'splines' } } as any)
    return <LayoutSVG result={result} />
  }
}
