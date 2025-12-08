// apps/storybook/src/stories/Editor.stories.tsx
import { useState } from 'react'
import { MonacoEditor } from '../../../../packages/ui/src/components/MonacoEditor'
import { SrujaMonacoEditor } from '../../../../packages/ui/src/components/SrujaMonacoEditor'
import type { Meta, StoryObj } from '@storybook/react'

const meta: Meta<typeof SrujaMonacoEditor> = {
  title: 'Components/Editor',
  component: SrujaMonacoEditor,
  tags: ['autodocs'],
  parameters: {
    docs: {
      description: {
        component: 'Monaco-based code editor with Sruja language support. Provides syntax highlighting, autocomplete, and validation for architecture-as-code DSL. Used in Sruja Studio for editing architecture definitions.',
      },
    },
  },
}

export default meta
type Story = StoryObj<typeof SrujaMonacoEditor>

const SIMPLE_EXAMPLE = `system App "My App" {
  container Web "Web Server"
  datastore DB "Database"
}
person User "User"
User -> App.Web "Visits"
App.Web -> App.DB "Reads/Writes"`

const ECOMMERCE_EXAMPLE = `system EcommerceSystem "E-commerce Platform" {
  container WebApp "Web Application"
  container API "API Gateway"
  container Payment "Payment Service"
  container Inventory "Inventory Service"
  
  datastore UserDB "User Database"
  datastore ProductDB "Product Database"
  datastore OrderDB "Order Database"
}

person Customer "Customer"
person Admin "Administrator"

Customer -> EcommerceSystem.WebApp "Browses"
Customer -> EcommerceSystem.WebApp "Purchases"
EcommerceSystem.WebApp -> EcommerceSystem.API "Requests"
EcommerceSystem.API -> EcommerceSystem.Payment "Processes"
EcommerceSystem.API -> EcommerceSystem.Inventory "Checks"
EcommerceSystem.Payment -> EcommerceSystem.OrderDB "Stores"
EcommerceSystem.Inventory -> EcommerceSystem.ProductDB "Queries"
Admin -> EcommerceSystem.API "Manages"`

export const Basic: Story = {
  render: () => {
    const [value, setValue] = useState(SIMPLE_EXAMPLE)
    return (
      <div style={{ height: '400px', border: '1px solid var(--color-border)', borderRadius: 8, overflow: 'hidden' }}>
        <SrujaMonacoEditor value={value} onChange={setValue} height="100%" />
      </div>
    )
  },
  parameters: {
    docs: {
      description: {
        story: 'Basic Sruja editor with simple architecture definition.',
      },
    },
  },
}

export const ComplexArchitecture: Story = {
  render: () => {
    const [value, setValue] = useState(ECOMMERCE_EXAMPLE)
    return (
      <div style={{ height: '500px', border: '1px solid var(--color-border)', borderRadius: 8, overflow: 'hidden' }}>
        <SrujaMonacoEditor value={value} onChange={setValue} height="100%" />
      </div>
    )
  },
  parameters: {
    docs: {
      description: {
        story: 'Editor with complex e-commerce platform architecture definition.',
      },
    },
  },
}

export const SplitView: Story = {
  render: () => {
    const [value, setValue] = useState(SIMPLE_EXAMPLE)
    return (
      <div style={{ display: 'flex', height: '500px', border: '1px solid var(--color-border)', borderRadius: 8, overflow: 'hidden' }}>
        <div style={{ width: '50%', borderRight: '1px solid var(--color-border)' }}>
          <div style={{ padding: 8, borderBottom: '1px solid var(--color-border)', fontSize: 12, fontWeight: 600, backgroundColor: 'var(--color-surface)' }}>
            Editor
          </div>
          <div style={{ height: 'calc(100% - 40px)' }}>
            <SrujaMonacoEditor value={value} onChange={setValue} height="100%" />
          </div>
        </div>
        <div style={{ width: '50%', backgroundColor: 'var(--color-surface)', display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--color-text-secondary)' }}>
          <div style={{ textAlign: 'center' }}>
            <div style={{ fontSize: 18, fontWeight: 600, marginBottom: 8 }}>Diagram Preview</div>
            <div style={{ fontSize: 14 }}>Interactive viewer would appear here</div>
          </div>
        </div>
      </div>
    )
  },
  parameters: {
    docs: {
      description: {
        story: 'Split view layout with editor on left and diagram preview on right, as used in Studio.',
      },
    },
  },
}
