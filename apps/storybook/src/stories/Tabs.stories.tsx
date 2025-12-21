// apps/storybook/src/stories/Tabs.stories.tsx
import { Tabs } from '../../../../packages/ui/src/components/Tabs'
import { Card } from '../../../../packages/ui/src/components/Card'
import { Badge } from '../../../../packages/ui/src/components/Badge'
import type { Meta, StoryObj } from '@storybook/react'

const meta: Meta<typeof Tabs> = {
  title: 'Components/Tabs',
  component: Tabs,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: 'Tab navigation component for organizing content into sections. Used in Sruja Studio for switching between editor, viewer, and documentation views.',
      },
    },
  },
  argTypes: {
    tabs: {
      description: 'Array of tab definitions with id, label, and content',
    },
  },
}

export default meta
type Story = StoryObj<typeof Tabs>

export const Playground: Story = {
  args: {
    tabs: [
      { id: 'editor', label: 'Editor', content: <div style={{ padding: 16 }}>Editor content</div> },
      { id: 'viewer', label: 'Viewer', content: <div style={{ padding: 16 }}>Viewer content</div> },
      { id: 'docs', label: 'Documentation', content: <div style={{ padding: 16 }}>Documentation content</div> },
    ],
  },
}

export const ViewSwitcher: Story = {
  render: () => (
    <Tabs
      tabs={[
        {
          id: 'editor',
          label: 'Editor',
          content: (
            <div style={{ padding: 24, backgroundColor: 'var(--color-surface)', minHeight: 200, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--color-text-secondary)' }}>
              <div style={{ textAlign: 'center' }}>
                <div style={{ fontSize: 18, fontWeight: 600, marginBottom: 8 }}>DSL Editor</div>
                <div style={{ fontSize: 14 }}>Monaco editor with syntax highlighting</div>
              </div>
            </div>
          ),
        },
        {
          id: 'split',
          label: 'Split View',
          content: (
            <div style={{ padding: 24, backgroundColor: 'var(--color-surface)', minHeight: 200, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--color-text-secondary)' }}>
              <div style={{ textAlign: 'center' }}>
                <div style={{ fontSize: 18, fontWeight: 600, marginBottom: 8 }}>Split View</div>
                <div style={{ fontSize: 14 }}>Editor and diagram side by side</div>
              </div>
            </div>
          ),
        },
        {
          id: 'viewer',
          label: 'Viewer',
          content: (
            <div style={{ padding: 24, backgroundColor: 'var(--color-surface)', minHeight: 200, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--color-text-secondary)' }}>
              <div style={{ textAlign: 'center' }}>
                <div style={{ fontSize: 18, fontWeight: 600, marginBottom: 8 }}>Diagram Viewer</div>
                <div style={{ fontSize: 14 }}>Interactive architecture diagram</div>
              </div>
            </div>
          ),
        },
      ]}
    />
  ),
  parameters: {
    docs: {
      description: {
        story: 'Tabs for switching between different view modes in Studio.',
      },
    },
  },
}

export const WithContent: Story = {
  render: () => (
    <Tabs
      tabs={[
        {
          id: 'overview',
          label: 'Overview',
          content: (
            <div style={{ padding: 24 }}>
              <h3 style={{ marginTop: 0 }}>Architecture Overview</h3>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: 16 }}>
                <Card title="Systems" footer={<Badge color="brand">3</Badge>}>
                  <p style={{ fontSize: 14, color: 'var(--color-text-secondary)', margin: 0 }}>Total systems</p>
                </Card>
                <Card title="Containers" footer={<Badge>12</Badge>}>
                  <p style={{ fontSize: 14, color: 'var(--color-text-secondary)', margin: 0 }}>Container components</p>
                </Card>
                <Card title="Relations" footer={<Badge color="info">24</Badge>}>
                  <p style={{ fontSize: 14, color: 'var(--color-text-secondary)', margin: 0 }}>Connections</p>
                </Card>
              </div>
            </div>
          ),
        },
        {
          id: 'validation',
          label: 'Validation',
          content: (
            <div style={{ padding: 24 }}>
              <h3 style={{ marginTop: 0 }}>Validation Results</h3>
              <Card title="Status" footer={<Badge color="success">Valid</Badge>}>
                <p style={{ fontSize: 14, color: 'var(--color-text-secondary)', margin: 0 }}>
                  No errors or warnings found in the architecture definition.
                </p>
              </Card>
            </div>
          ),
        },
        {
          id: 'export',
          label: 'Export',
          content: (
            <div style={{ padding: 24 }}>
              <h3 style={{ marginTop: 0 }}>Export Options</h3>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                <button style={{ padding: '12px 16px', borderRadius: 6, border: '1px solid var(--color-border)', backgroundColor: 'var(--color-background)', cursor: 'pointer', textAlign: 'left' }}>
                  Export as PNG
                </button>
                <button style={{ padding: '12px 16px', borderRadius: 6, border: '1px solid var(--color-border)', backgroundColor: 'var(--color-background)', cursor: 'pointer', textAlign: 'left' }}>
                  Export as SVG
                </button>
                <button style={{ padding: '12px 16px', borderRadius: 6, border: '1px solid var(--color-border)', backgroundColor: 'var(--color-background)', cursor: 'pointer', textAlign: 'left' }}>
                  Export as JSON
                </button>
              </div>
            </div>
          ),
        },
      ]}
    />
  ),
  parameters: {
    docs: {
      description: {
        story: 'Tabs with rich content sections for architecture management.',
      },
    },
  },
}

export const Showcase: Story = {
  render: () => (
    <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5 max-w-2xl">
      <Tabs
        tabs={[
          { id: 'overview', label: 'Overview', content: <div className="p-4 text-[var(--color-text-secondary)]">Overview content</div> },
          { id: 'diagram', label: 'Diagram', content: <div className="p-4 text-[var(--color-text-secondary)]">Diagram content</div> },
          { id: 'export', label: 'Export', content: <div className="p-4 text-[var(--color-text-secondary)]">Export options</div> },
        ]}
      />
    </div>
  ),
}
