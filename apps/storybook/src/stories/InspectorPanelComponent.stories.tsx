import type { Meta, StoryObj } from '@storybook/react'
import React from 'react'
import { InspectorPanel } from '@sruja/viewer-core/components/InspectorPanel'

const meta: Meta<typeof InspectorPanel> = {
  title: 'Viewer/InspectorPanel',
  component: InspectorPanel,
}

export default meta
type Story = StoryObj<typeof InspectorPanel>

const data = {
  architecture: {
    persons: [{ id: 'User', label: 'User' }],
    systems: [{ id: 'Sys', label: 'System', containers: [] }],
    relations: []
  },
  metadata: { name: 'Demo' }
} as any

export const Basic: Story = {
  render: () => (
    <InspectorPanel
      nodeId={'User'}
      data={data}
      onClose={() => {}}
      onSelectNode={() => {}}
      onSelectRequirement={() => {}}
      onSelectADR={() => {}}
    />
  )
}
