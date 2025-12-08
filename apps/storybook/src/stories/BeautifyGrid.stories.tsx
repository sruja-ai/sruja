import type { Meta, StoryObj } from '@storybook/react'
import React from 'react'
import { LayoutSVG } from '../components/LayoutSVG'
import { createC4Id, createC4Graph, layout, SystemContextView, InteractivePreset } from '@sruja/layout'

const meta: Meta<typeof LayoutSVG> = { title: 'Examples/Beautify & Grid', component: LayoutSVG }
export default meta
type Story = StoryObj<typeof LayoutSVG>

function build() {
  const sys = { id: createC4Id('Sys'), label: 'Sys', kind: 'SoftwareSystem', level: 'context', tags: new Set<string>() }
  const a = { id: createC4Id('A'), label: 'A', kind: 'SoftwareSystem', level: 'context', tags: new Set<string>() }
  const b = { id: createC4Id('B'), label: 'B', kind: 'SoftwareSystem', level: 'context', tags: new Set<string>() }
  const rels = [{ id: 'A->Sys', from: a.id, to: sys.id }, { id: 'B->Sys', from: b.id, to: sys.id }]
  return createC4Graph([sys as any, a as any, b as any], rels as any)
}

export const NoSnap: Story = { render: () => <LayoutSVG result={layout(build(), { ...SystemContextView(createC4Id('Sys')), snapToGrid: false }, InteractivePreset)} /> }
export const SnapToGrid: Story = { render: () => <LayoutSVG result={layout(build(), { ...SystemContextView(createC4Id('Sys')), snapToGrid: true, gridSize: 20 }, InteractivePreset)} /> }
