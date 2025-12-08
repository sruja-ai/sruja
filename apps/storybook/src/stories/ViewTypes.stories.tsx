import type { Meta, StoryObj } from '@storybook/react'
import React from 'react'
import { LayoutSVG } from '../components/LayoutSVG'
import { createC4Id, createC4Graph, layout, SystemContextView, ContainerView, ComponentView, DeploymentView, InteractivePreset } from '@sruja/layout'

const meta: Meta<typeof LayoutSVG> = { title: 'Examples/View Types', component: LayoutSVG }
export default meta
type Story = StoryObj<typeof LayoutSVG>

function buildSystem() {
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

function buildComponents() {
  const api = { id: createC4Id('API'), label: 'API', kind: 'Container', level: 'container', tags: new Set<string>() }
  const auth = { id: createC4Id('Auth'), label: 'Auth', kind: 'Component', level: 'component', parentId: api.id, tags: new Set<string>() }
  const gateway = { id: createC4Id('Gateway'), label: 'Gateway', kind: 'Component', level: 'component', parentId: api.id, tags: new Set<string>() }
  const rels = [{ id: 'Gateway->Auth', from: gateway.id, to: auth.id }]
  return createC4Graph([api as any, auth as any, gateway as any], rels as any)
}

function buildDeployment() {
  const prod = 'production'
  const node = { id: createC4Id('EC2'), label: 'EC2', kind: 'DeploymentNode', level: 'deployment', tags: new Set([prod, 'infrastructure']) }
  const container = { id: createC4Id('API'), label: 'API', kind: 'Container', level: 'deployment', tags: new Set([prod]) }
  const db = { id: createC4Id('RDS'), label: 'RDS', kind: 'Database', level: 'deployment', tags: new Set([prod]) }
  const rels = [{ id: 'API->RDS', from: container.id, to: db.id }]
  return createC4Graph([node as any, container as any, db as any], rels as any)
}

export const SystemContext: Story = { render: () => <LayoutSVG result={layout(buildSystem(), SystemContextView(createC4Id('Sys')), InteractivePreset)} /> }
export const Container: Story = { render: () => <LayoutSVG result={layout(buildContainers(), ContainerView(createC4Id('Sys')), InteractivePreset)} /> }
export const Component: Story = { render: () => <LayoutSVG result={layout(buildComponents(), ComponentView(createC4Id('API')), InteractivePreset)} /> }
export const Deployment: Story = { render: () => <LayoutSVG result={layout(buildDeployment(), DeploymentView('production'), InteractivePreset)} /> }
