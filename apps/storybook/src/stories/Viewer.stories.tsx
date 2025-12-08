// apps/storybook/src/stories/Viewer.stories.tsx
import { SrujaViewerView, type ArchitectureJSON } from '../../../../packages/viewer/src/index.ts'
import type { Meta, StoryObj } from '@storybook/react'

const meta: Meta<typeof SrujaViewerView> = {
  title: 'Components/Viewer',
  component: SrujaViewerView,
  tags: ['autodocs'],
  parameters: {
    docs: {
      description: {
        component: 'Interactive architecture diagram viewer. Renders C4 model diagrams from architecture JSON data. Supports zoom, pan, selection, and level navigation. This is the core visualization component in Sruja Studio.',
      },
    },
  },
}

export default meta
type Story = StoryObj<typeof SrujaViewerView>

const SIMPLE_WEB_APP: ArchitectureJSON = {
  metadata: { name: 'Simple Web Application', version: '1.0.0' },
  architecture: {
    systems: [
      {
        id: 'WebApp',
        label: 'Web Application',
        containers: [
          { id: 'API', label: 'API Service' },
          { id: 'DB', label: 'Database' },
        ],
      },
    ],
    persons: [{ id: 'User', label: 'End User' }],
    relations: [
      { from: 'User', to: 'WebApp.API', verb: 'Visits' },
      { from: 'WebApp.API', to: 'WebApp.DB', verb: 'Reads/Writes' },
    ],
  },
}

const ECOMMERCE_PLATFORM: ArchitectureJSON = {
  metadata: { name: 'E-commerce Platform', version: '1.0.0' },
  architecture: {
    systems: [
      {
        id: 'EcommerceSystem',
        label: 'E-commerce Platform',
        containers: [
          { id: 'WebApp', label: 'Web Application' },
          { id: 'API', label: 'API Gateway' },
          { id: 'Payment', label: 'Payment Service' },
          { id: 'Inventory', label: 'Inventory Service' },
        ],
        datastores: [
          { id: 'UserDB', label: 'User Database' },
          { id: 'ProductDB', label: 'Product Database' },
          { id: 'OrderDB', label: 'Order Database' },
        ],
      },
    ],
    persons: [
      { id: 'Customer', label: 'Customer' },
      { id: 'Admin', label: 'Administrator' },
    ],
    relations: [
      { from: 'Customer', to: 'EcommerceSystem.WebApp', verb: 'Browses' },
      { from: 'Customer', to: 'EcommerceSystem.WebApp', verb: 'Purchases' },
      { from: 'EcommerceSystem.WebApp', to: 'EcommerceSystem.API', verb: 'Requests' },
      { from: 'EcommerceSystem.API', to: 'EcommerceSystem.Payment', verb: 'Processes' },
      { from: 'EcommerceSystem.API', to: 'EcommerceSystem.Inventory', verb: 'Checks' },
      { from: 'EcommerceSystem.Payment', to: 'EcommerceSystem.OrderDB', verb: 'Stores' },
      { from: 'EcommerceSystem.Inventory', to: 'EcommerceSystem.ProductDB', verb: 'Queries' },
      { from: 'Admin', to: 'EcommerceSystem.API', verb: 'Manages' },
    ],
  },
}

export const SimpleWebApp: Story = {
  render: () => (
    <div style={{ height: '500px', border: '1px solid var(--color-border)', borderRadius: 8, overflow: 'hidden' }}>
      <SrujaViewerView data={SIMPLE_WEB_APP} />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Simple web application architecture with user, web app, API, and database.',
      },
    },
  },
}

export const EcommercePlatform: Story = {
  render: () => (
    <div style={{ height: '600px', border: '1px solid var(--color-border)', borderRadius: 8, overflow: 'hidden' }}>
      <SrujaViewerView data={ECOMMERCE_PLATFORM} />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Complex e-commerce platform architecture with multiple services, databases, and user types.',
      },
    },
  },
}

export const WithControls: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', height: '600px', border: '1px solid var(--color-border)', borderRadius: 8, overflow: 'hidden' }}>
      <div style={{ padding: 12, borderBottom: '1px solid var(--color-border)', display: 'flex', gap: 8, alignItems: 'center', backgroundColor: 'var(--color-surface)' }}>
        <button style={{ padding: '6px 12px', borderRadius: 6, border: '1px solid var(--color-border)', fontSize: 14, cursor: 'pointer' }}>
          Level 1
        </button>
        <button style={{ padding: '6px 12px', borderRadius: 6, border: '1px solid var(--color-border)', fontSize: 14, cursor: 'pointer' }}>
          Level 2
        </button>
        <button style={{ padding: '6px 12px', borderRadius: 6, border: '1px solid var(--color-border)', fontSize: 14, cursor: 'pointer' }}>
          Level 3
        </button>
        <div style={{ flex: 1 }} />
        <span style={{ fontSize: 12, color: 'var(--color-text-secondary)' }}>Zoom: 100%</span>
      </div>
      <div style={{ flex: 1, position: 'relative' }}>
        <SrujaViewerView data={ECOMMERCE_PLATFORM} />
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Viewer with level navigation controls and zoom indicator, as used in Studio.',
      },
    },
  },
}
