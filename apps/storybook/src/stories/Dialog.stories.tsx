// apps/storybook/src/stories/Dialog.stories.tsx
import { useState } from 'react'
import { Dialog } from '../../../../packages/ui/src/components/Dialog'
import { Button } from '../../../../packages/ui/src/components/Button'
import { Input } from '../../../../packages/ui/src/components/Input'
import type { Meta, StoryObj } from '@storybook/react'

const meta: Meta<typeof Dialog> = {
  title: 'Components/Dialog',
  component: Dialog,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: 'A modal dialog component for displaying important information, forms, or confirmations. Used throughout Sruja Studio for user interactions like adding elements, exporting diagrams, and confirming actions.',
      },
    },
  },
  argTypes: {
    isOpen: {
      control: { type: 'boolean' },
      description: 'Controls dialog visibility',
    },
    title: {
      control: { type: 'text' },
      description: 'Dialog title',
    },
    onClose: {
      action: 'closed',
      description: 'Callback fired when dialog should close',
    },
  },
}

export default meta
type Story = StoryObj<typeof Dialog>

export const Playground: Story = {
  render: () => {
    const [open, setOpen] = useState(false)
    return (
      <div style={{ padding: 16 }}>
        <Button onClick={() => setOpen(true)}>Open Dialog</Button>
        <Dialog
          isOpen={open}
          onClose={() => setOpen(false)}
          title="Dialog Title"
          footer={
            <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
              <Button variant="ghost" onClick={() => setOpen(false)}>Cancel</Button>
              <Button onClick={() => setOpen(false)}>Confirm</Button>
            </div>
          }
        >
          Dialog content goes here.
        </Dialog>
      </div>
    )
  },
}

export const AddElement: Story = {
  render: () => {
    const [open, setOpen] = useState(false)
    const [name, setName] = useState('')
    return (
      <div style={{ padding: 16 }}>
        <Button onClick={() => setOpen(true)}>Add Container</Button>
        <Dialog
          isOpen={open}
          onClose={() => {
            setOpen(false)
            setName('')
          }}
          title="Add Container"
          footer={
            <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
              <Button variant="ghost" onClick={() => setOpen(false)}>Cancel</Button>
              <Button 
                onClick={() => {
                  alert(`Added container: ${name}`)
                  setOpen(false)
                  setName('')
                }}
                disabled={!name.trim()}
              >
                Add
              </Button>
            </div>
          }
        >
          <div style={{ padding: '8px 0' }}>
            <Input
              label="Container Name"
              placeholder="e.g., API Service"
              value={name}
              onChange={(e) => setName(e.target.value)}
              helperText="Enter a descriptive name for the container"
            />
          </div>
        </Dialog>
      </div>
    )
  },
  parameters: {
    docs: {
      description: {
        story: 'Dialog for adding new architecture elements with input validation.',
      },
    },
  },
}

export const Confirmation: Story = {
  render: () => {
    const [open, setOpen] = useState(false)
    return (
      <div style={{ padding: 16 }}>
        <Button variant="danger" onClick={() => setOpen(true)}>Delete System</Button>
        <Dialog
          isOpen={open}
          onClose={() => setOpen(false)}
          title="Delete System"
          footer={
            <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
              <Button variant="ghost" onClick={() => setOpen(false)}>Cancel</Button>
              <Button 
                variant="danger" 
                onClick={() => {
                  alert('System deleted')
                  setOpen(false)
                }}
              >
                Delete
              </Button>
            </div>
          }
        >
          <p style={{ margin: 0, color: 'var(--color-text-secondary)' }}>
            Are you sure you want to delete "Web Application"? This action cannot be undone and will remove all containers and relations.
          </p>
        </Dialog>
      </div>
    )
  },
  parameters: {
    docs: {
      description: {
        story: 'Confirmation dialog for destructive actions like deleting architecture elements.',
      },
    },
  },
}

export const ExportOptions: Story = {
  render: () => {
    const [open, setOpen] = useState(false)
    const [format, setFormat] = useState<'png' | 'svg' | 'json'>('png')
    return (
      <div style={{ padding: 16 }}>
        <Button onClick={() => setOpen(true)}>Export Diagram</Button>
        <Dialog
          isOpen={open}
          onClose={() => setOpen(false)}
          title="Export Diagram"
          footer={
            <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
              <Button variant="ghost" onClick={() => setOpen(false)}>Cancel</Button>
              <Button onClick={() => {
                alert(`Exporting as ${format.toUpperCase()}`)
                setOpen(false)
              }}>
                Export
              </Button>
            </div>
          }
        >
          <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            <div>
              <label style={{ display: 'block', marginBottom: 8, fontSize: 14, fontWeight: 500 }}>
                Format
              </label>
              <div style={{ display: 'flex', gap: 8 }}>
                {(['png', 'svg', 'json'] as const).map((fmt) => (
                  <button
                    key={fmt}
                    onClick={() => setFormat(fmt)}
                    style={{
                      flex: 1,
                      padding: '8px 16px',
                      borderRadius: 6,
                      border: `2px solid ${format === fmt ? '#3b82f6' : '#e2e8f0'}`,
                      backgroundColor: format === fmt ? '#eff6ff' : '#fff',
                      color: format === fmt ? '#1d4ed8' : '#475569',
                      cursor: 'pointer',
                      fontSize: 14,
                      fontWeight: format === fmt ? 600 : 500,
                      textTransform: 'uppercase',
                    }}
                  >
                    {fmt}
                  </button>
                ))}
              </div>
            </div>
            <div>
              <label style={{ display: 'flex', alignItems: 'center', gap: 8, cursor: 'pointer' }}>
                <input type="checkbox" defaultChecked />
                <span style={{ fontSize: 14 }}>Include metadata</span>
              </label>
            </div>
          </div>
        </Dialog>
      </div>
    )
  },
  parameters: {
    docs: {
      description: {
        story: 'Dialog for exporting architecture diagrams with format selection and options.',
      },
    },
  },
}

export const Showcase: Story = {
  render: () => {
    const [open, setOpen] = useState(false)
    return (
      <div>
        <Button onClick={() => setOpen(true)}>Open Dialog</Button>
        <Dialog isOpen={open} onClose={() => setOpen(false)} title="Modern Dialog" footer={<div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}><Button variant="ghost" onClick={() => setOpen(false)}>Cancel</Button><Button onClick={() => setOpen(false)}>Confirm</Button></div>}>
          <div className="text-sm text-[var(--color-text-secondary)]">Dialog content aligned with design-system spacing, borders, and typography.</div>
        </Dialog>
      </div>
    )
  },
}
