// apps/storybook/src/stories/Card.stories.tsx
import { Card } from '../../../../packages/ui/src/components/Card'
import { Badge } from '../../../../packages/ui/src/components/Badge'
import { Button } from '../../../../packages/ui/src/components/Button'
import type { Meta, StoryObj } from '@storybook/react'

const meta: Meta<typeof Card> = {
  title: 'Components/Card',
  component: Card,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component:
          'Versatile card with header, footer, and interactive states. Designed for dashboards and rich content layouts with consistent spacing and elevation.',
      },
    },
  },
  argTypes: {
    title: {
      control: { type: 'text' },
      description: 'Main title displayed in the card header',
    },
    subtitle: {
      control: { type: 'text' },
      description: 'Secondary text displayed below the title',
    },
    interactive: {
      control: { type: 'boolean' },
      description: 'Enables hover and focus states for clickable cards',
    },
    onClick: {
      action: 'clicked',
      description: 'Callback fired when card is clicked',
    },
  },
}

export default meta
type Story = StoryObj<typeof Card>

export const Playground: Story = {
  args: {
    title: 'Card Title',
    subtitle: 'Optional subtitle',
    children: 'Content inside the card.',
    footer: <Badge color="brand">Footer Info</Badge>,
    interactive: true,
  },
}

export const Basic: Story = {
  args: {
    title: 'Architecture Overview',
    subtitle: 'System components and relationships',
    children: (
      <div className="space-y-2">
        <p className="text-sm text-[var(--color-text-secondary)]">
          This card displays a basic architecture overview with title, subtitle, and content.
        </p>
        <ul className="text-sm text-[var(--color-text-secondary)] list-disc list-inside space-y-1">
          <li>Web Application</li>
          <li>API Service</li>
          <li>Database</li>
        </ul>
      </div>
    ),
  },
  parameters: {
    docs: {
      description: {
        story: 'Basic card with title, subtitle, and content area.',
      },
    },
  },
}

export const WithFooter: Story = {
  args: {
    title: 'Project Status',
    subtitle: 'Last updated 2 hours ago',
    children: (
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <span className="text-sm text-[var(--color-text-secondary)]">Completion</span>
          <span className="text-sm font-medium text-[var(--color-text-primary)]">75%</span>
        </div>
        <div className="w-full bg-[var(--color-neutral-200)] rounded-full h-2">
          <div className="bg-[var(--color-primary)] h-2 rounded-full" style={{ width: '75%' }} />
        </div>
      </div>
    ),
    footer: (
      <div className="flex items-center justify-between">
        <Badge color="success">Active</Badge>
        <Button variant="ghost" size="sm">View Details</Button>
      </div>
    ),
  },
  parameters: {
    docs: {
      description: {
        story: 'Card with footer section containing badges and actions.',
      },
    },
  },
}

export const Interactive: Story = {
  args: {
    title: 'E-commerce Platform',
    subtitle: 'Click to view architecture details',
    children: (
      <div className="space-y-2">
        <p className="text-sm text-[var(--color-text-secondary)]">
          A comprehensive e-commerce solution with payment processing, inventory management, and order fulfillment.
        </p>
        <div className="flex gap-2 flex-wrap">
          <Badge color="info">Web App</Badge>
          <Badge color="info">API</Badge>
          <Badge color="info">Database</Badge>
        </div>
      </div>
    ),
    footer: <Badge color="brand">12 Components</Badge>,
    interactive: true,
    onClick: () => alert('Card clicked!'),
  },
  parameters: {
    docs: {
      description: {
        story: 'Interactive card with hover and focus states. Click to trigger action.',
      },
    },
  },
}

export const FeatureShowcase: Story = {
  render: () => (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
      <Card
        title="Architecture as Code"
        subtitle="Define with DSL"
        footer={<Badge color="brand">New</Badge>}
      >
        <p className="text-sm text-[var(--color-text-secondary)]">
          Write architecture definitions using a simple, readable domain-specific language.
        </p>
      </Card>
      <Card
        title="Visual Diagrams"
        subtitle="Auto-generated"
        footer={<Badge color="success">Active</Badge>}
      >
        <p className="text-sm text-[var(--color-text-secondary)]">
          Automatically generate beautiful diagrams from your architecture code.
        </p>
      </Card>
      <Card
        title="Validation Engine"
        subtitle="Catch errors early"
        footer={<Badge color="info">Beta</Badge>}
      >
        <p className="text-sm text-[var(--color-text-secondary)]">
          Built-in validation ensures your architecture is consistent and error-free.
        </p>
      </Card>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Card grid layout for showcasing features or content collections.',
      },
    },
  },
}

export const DashboardCard: Story = {
  args: {
    title: 'System Health',
    subtitle: 'Real-time monitoring',
    children: (
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <span className="text-2xl font-bold text-[var(--color-text-primary)]">98.5%</span>
          <Badge color="success">Healthy</Badge>
        </div>
        <div className="space-y-2">
          <div className="flex justify-between text-sm">
            <span className="text-[var(--color-text-secondary)]">Uptime</span>
            <span className="text-[var(--color-text-primary)] font-medium">99.9%</span>
          </div>
          <div className="flex justify-between text-sm">
            <span className="text-[var(--color-text-secondary)]">Response Time</span>
            <span className="text-[var(--color-text-primary)] font-medium">120ms</span>
          </div>
          <div className="flex justify-between text-sm">
            <span className="text-[var(--color-text-secondary)]">Active Users</span>
            <span className="text-[var(--color-text-primary)] font-medium">1,234</span>
          </div>
        </div>
      </div>
    ),
    footer: (
      <Button variant="ghost" size="sm" className="w-full">
        View Full Report
      </Button>
    ),
  },
  parameters: {
    docs: {
      description: {
        story: 'Dashboard-style card with metrics and statistics.',
      },
    },
  },
}
