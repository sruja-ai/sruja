// apps/storybook/src/stories/Breadcrumb.stories.tsx
import { Breadcrumb } from '../../../../packages/ui/src/components/Breadcrumb'
import type { Meta, StoryObj } from '@storybook/react'

const meta: Meta<typeof Breadcrumb> = {
  title: 'Components/Breadcrumb',
  component: Breadcrumb,
  tags: ['autodocs'],
  parameters: {
    docs: {
      description: {
        component: 'Breadcrumb navigation component for showing hierarchical navigation paths. Used in Sruja Viewer to show the current location in the architecture hierarchy (e.g., System > Container > Component).',
      },
    },
  },
  argTypes: {
    items: {
      description: 'Array of breadcrumb items to display',
    },
    onItemClick: {
      action: 'item clicked',
      description: 'Callback fired when a breadcrumb item is clicked',
    },
    onHomeClick: {
      action: 'home clicked',
      description: 'Callback fired when home button is clicked',
    },
    showHome: {
      control: { type: 'boolean' },
      description: 'Whether to show the home button',
    },
  },
}

export default meta
type Story = StoryObj<typeof Breadcrumb>

export const Playground: Story = {
  args: {
    items: [
      { id: 'system1', label: 'E-commerce Platform' },
      { id: 'container1', label: 'API Gateway' },
    ],
    onItemClick: (id) => console.log('Clicked:', id),
    showHome: true,
  },
}

export const SystemNavigation: Story = {
  args: {
    items: [
      { id: 'ecommerce', label: 'E-commerce Platform' },
    ],
    onItemClick: (id) => alert(`Navigate to: ${id}`),
    showHome: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Breadcrumb showing navigation to a system level.',
      },
    },
  },
}

export const ContainerNavigation: Story = {
  args: {
    items: [
      { id: 'ecommerce', label: 'E-commerce Platform' },
      { id: 'api', label: 'API Gateway' },
    ],
    onItemClick: (id) => alert(`Navigate to: ${id}`),
    showHome: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Breadcrumb showing navigation to a container within a system.',
      },
    },
  },
}

export const ComponentNavigation: Story = {
  args: {
    items: [
      { id: 'ecommerce', label: 'E-commerce Platform' },
      { id: 'api', label: 'API Gateway' },
      { id: 'auth', label: 'Authentication Service' },
    ],
    onItemClick: (id) => alert(`Navigate to: ${id}`),
    showHome: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Breadcrumb showing deep navigation to a component level.',
      },
    },
  },
}

export const WithoutHome: Story = {
  args: {
    items: [
      { id: 'system1', label: 'Web Application' },
      { id: 'container1', label: 'API Service' },
    ],
    onItemClick: (id) => alert(`Navigate to: ${id}`),
    showHome: false,
  },
  parameters: {
    docs: {
      description: {
        story: 'Breadcrumb without home button for simpler navigation.',
      },
    },
  },
}

export const LongPath: Story = {
  args: {
    items: [
      { id: 'system1', label: 'E-commerce Platform' },
      { id: 'container1', label: 'Payment Service' },
      { id: 'component1', label: 'Payment Processor' },
      { id: 'subcomponent1', label: 'Credit Card Handler' },
    ],
    onItemClick: (id) => alert(`Navigate to: ${id}`),
    showHome: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Breadcrumb with a long navigation path showing multiple hierarchy levels.',
      },
    },
  },
}

export const InTopBar: Story = {
  render: () => (
    <div style={{ 
      display: 'flex', 
      alignItems: 'center', 
      gap: 16, 
      padding: '12px 16px',
      backgroundColor: 'var(--color-background)',
      borderBottom: '1px solid var(--color-border)',
    }}>
      <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--color-text-primary)' }}>
        Sruja Architecture
      </div>
      <Breadcrumb
        items={[
          { id: 'ecommerce', label: 'E-commerce Platform' },
          { id: 'api', label: 'API Gateway' },
        ]}
        onItemClick={(id) => alert(`Navigate to: ${id}`)}
        showHome={true}
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Breadcrumb integrated into a top bar navigation, as used in the Viewer app.',
      },
    },
  },
}

