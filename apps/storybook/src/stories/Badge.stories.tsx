// apps/storybook/src/stories/Badge.stories.tsx
import { Badge } from '../../../../packages/ui/src/components/Badge'
import { Card } from '../../../../packages/ui/src/components/Card'
import type { Meta, StoryObj } from '@storybook/react'

const meta: Meta<typeof Badge> = {
  title: 'Components/Badge',
  component: Badge,
  tags: ['autodocs'],
  parameters: {
    docs: {
      description: {
        component: 'Badge component for displaying status, labels, and metadata. Used throughout Sruja to indicate element types, validation status, and system states.',
      },
    },
  },
  argTypes: {
    color: {
      control: { type: 'select' },
      options: ['default', 'brand', 'success', 'error', 'warning', 'info'],
      description: 'Color variant of the badge',
    },
    children: {
      control: { type: 'text' },
      description: 'Badge content',
    },
  },
}

export default meta
type Story = StoryObj<typeof Badge>

export const Playground: Story = {
  args: {
    children: 'Badge',
    color: 'brand',
  },
}

export const Variants: Story = {
  render: () => (
    <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8 }}>
      <Badge>Neutral</Badge>
      <Badge color="brand">Brand</Badge>
      <Badge color="success">Success</Badge>
      <Badge color="error">Error</Badge>
      <Badge color="warning">Warning</Badge>
      <Badge color="info">Info</Badge>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'All available badge color variants.',
      },
    },
  },
}

export const ElementTypes: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
      <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
        <span style={{ fontSize: 14, color: 'var(--color-text-secondary)', minWidth: 100 }}>Person:</span>
        <Badge color="info">User</Badge>
        <Badge color="info">Admin</Badge>
      </div>
      <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
        <span style={{ fontSize: 14, color: 'var(--color-text-secondary)', minWidth: 100 }}>System:</span>
        <Badge color="brand">Web Application</Badge>
        <Badge color="brand">API Gateway</Badge>
      </div>
      <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
        <span style={{ fontSize: 14, color: 'var(--color-text-secondary)', minWidth: 100 }}>Container:</span>
        <Badge>API Service</Badge>
        <Badge>Web Server</Badge>
      </div>
      <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
        <span style={{ fontSize: 14, color: 'var(--color-text-secondary)', minWidth: 100 }}>Datastore:</span>
        <Badge color="warning">PostgreSQL</Badge>
        <Badge color="warning">Redis</Badge>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Badges used to label different architecture element types.',
      },
    },
  },
}

export const StatusIndicators: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
      <Card
        title="Validation Status"
        subtitle="Architecture validation results"
      >
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          <Badge color="success">Valid</Badge>
          <Badge color="error">2 Errors</Badge>
          <Badge color="warning">1 Warning</Badge>
        </div>
      </Card>
      <Card
        title="System Health"
        subtitle="Component status indicators"
      >
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          <Badge color="success">Healthy</Badge>
          <Badge color="warning">Degraded</Badge>
          <Badge color="error">Down</Badge>
        </div>
      </Card>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Badges used for status indicators in architecture validation and system health.',
      },
    },
  },
}

export const InCards: Story = {
  render: () => (
    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))', gap: 16 }}>
      <Card
        title="E-commerce Platform"
        subtitle="Production system"
        footer={<Badge color="success">Active</Badge>}
      >
        <p style={{ fontSize: 14, color: 'var(--color-text-secondary)', margin: 0 }}>
          Main e-commerce application with payment processing.
        </p>
      </Card>
      <Card
        title="API Gateway"
        subtitle="Infrastructure component"
        footer={<Badge color="info">Beta</Badge>}
      >
        <p style={{ fontSize: 14, color: 'var(--color-text-secondary)', margin: 0 }}>
          API routing and load balancing service.
        </p>
      </Card>
      <Card
        title="User Database"
        subtitle="Data storage"
        footer={<Badge color="warning">Maintenance</Badge>}
      >
        <p style={{ fontSize: 14, color: 'var(--color-text-secondary)', margin: 0 }}>
          PostgreSQL database for user data.
        </p>
      </Card>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Badges used in card footers to show status or metadata.',
      },
    },
  },
}

