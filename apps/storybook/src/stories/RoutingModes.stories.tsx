import type { Meta, StoryObj } from '@storybook/react'
import React from 'react'
import { LayoutSVG } from '../components/LayoutSVG'
import { createC4Id, createC4Graph, layout, SystemContextView, InteractivePreset } from '@sruja/layout'

const meta: Meta<typeof LayoutSVG> = { title: 'Examples/Routing Modes', component: LayoutSVG }
export default meta
type Story = StoryObj<typeof LayoutSVG>

function build() {
  const sys = { id: createC4Id('Sys'), label: 'Sys', kind: 'SoftwareSystem', level: 'context', tags: new Set<string>() }
  const user = { id: createC4Id('User'), label: 'User', kind: 'Person', level: 'context', tags: new Set<string>() }
  const ext = { id: createC4Id('Payments'), label: 'Payments', kind: 'ExternalSystem', level: 'context', tags: new Set<string>() }
  const rels = [{ id: 'User->Sys', from: user.id, to: sys.id }, { id: 'Sys->Payments', from: sys.id, to: ext.id }]
  return createC4Graph([sys as any, user as any, ext as any], rels as any)
}

export const Orthogonal: Story = { render: () => <LayoutSVG result={layout(build(), SystemContextView(createC4Id('Sys')), InteractivePreset)} /> }
export const Splines: Story = { render: () => <LayoutSVG result={layout(build(), SystemContextView(createC4Id('Sys')), { ...InteractivePreset, edgeRouting: { ...InteractivePreset.edgeRouting, algorithm: 'splines' } } as any)} /> }
