import type { Meta, StoryObj } from '@storybook/react'
import React from 'react'
import { TopBar } from '@sruja/viewer-core/components/TopBar'
import { ThemeProvider } from '@sruja/ui'

const meta: Meta<typeof TopBar> = {
  title: 'Viewer/TopBar',
  component: TopBar,
}

export default meta
type Story = StoryObj<typeof TopBar>

const data = {
  architecture: {
    persons: [{ id: 'User', label: 'User' }],
    systems: [{ id: 'Sys', label: 'System', containers: [{ id: 'API', label: 'API' }] }],
    requirements: [{ id: 'R1', title: 'Login' }],
    adrs: [{ id: 'ADR-001', title: 'Use React' }],
  },
  metadata: { name: 'Demo' }
} as any

export const Basic: Story = {
  render: () => (
    <ThemeProvider defaultMode="system">
      <TopBar
        data={data}
        currentLevel={1}
        onSetLevel={() => {}}
        onSearch={() => {}}
        onSelectNode={() => {}}
        breadcrumbs={[{ id: 'root', label: 'Root' }, { id: 'Sys', label: 'System' }]}
        onBreadcrumbClick={() => {}}
        onLayoutChange={() => {}}
        dragEnabled={false}
        onToggleDrag={() => {}}
        onToggleSidebar={() => {}}
        onExport={() => {}}
        onEditInStudio={() => {}}
        onPreviewMarkdown={() => {}}
      />
    </ThemeProvider>
  )
}
