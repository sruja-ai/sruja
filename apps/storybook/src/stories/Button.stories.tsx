// apps/storybook/src/stories/Button.stories.tsx
import { Button } from '../../../../packages/ui/src/components/Button'
import type { Meta, StoryObj } from '@storybook/react'

const meta: Meta<typeof Button> = {
  title: 'Components/Button',
  component: Button,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component:
          'Enterprise-grade button with clear hierarchy, robust focus states, and accessible variants. Designed to align with modern product UI patterns.',
      },
    },
  },
  argTypes: {
    variant: {
      control: { type: 'select' },
      options: ['primary', 'secondary', 'outline', 'ghost', 'danger'],
      description: 'Visual style variant of the button',
      table: {
        type: { summary: 'primary | secondary | outline | ghost | danger' },
        defaultValue: { summary: 'primary' },
      },
    },
    size: {
      control: { type: 'radio' },
      options: ['sm', 'md', 'lg'],
      description: 'Size of the button',
      table: {
        type: { summary: 'sm | md | lg' },
        defaultValue: { summary: 'md' },
      },
    },
    isLoading: {
      control: { type: 'boolean' },
      description: 'Shows loading spinner and disables interaction',
      table: {
        type: { summary: 'boolean' },
        defaultValue: { summary: 'false' },
      },
    },
    disabled: {
      control: { type: 'boolean' },
      description: 'Disables the button',
      table: {
        type: { summary: 'boolean' },
        defaultValue: { summary: 'false' },
      },
    },
    children: {
      control: { type: 'text' },
      description: 'Button label or content',
    },
  },
}

export default meta
type Story = StoryObj<typeof Button>

export const Playground: Story = {
  args: {
    children: 'Button',
    variant: 'primary',
    size: 'md',
    isLoading: false,
  },
}

export const Variants: Story = {
  render: () => (
    <div className="flex flex-wrap gap-4 items-center">
      <Button variant="primary">Primary Action</Button>
      <Button variant="secondary">Secondary Action</Button>
      <Button variant="outline">Outline Button</Button>
      <Button variant="ghost">Ghost Button</Button>
      <Button variant="danger">Delete</Button>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'All available button variants for different use cases and visual hierarchy.',
      },
    },
  },
}

export const Sizes: Story = {
  render: () => (
    <div className="flex flex-wrap gap-4 items-center">
      <Button size="sm">Small Button</Button>
      <Button size="md">Medium Button</Button>
      <Button size="lg">Large Button</Button>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Three size options to match different UI contexts and importance levels.',
      },
    },
  },
}

export const States: Story = {
  render: () => (
    <div className="flex flex-wrap gap-4 items-center">
      <Button>Default</Button>
      <Button isLoading>Loading</Button>
      <Button disabled>Disabled</Button>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Button states including default, loading, and disabled states.',
      },
    },
  },
}

export const PrimaryActions: Story = {
  render: () => (
    <div className="space-y-4">
      <div className="flex gap-3">
        <Button variant="primary" size="lg">Create Architecture</Button>
        <Button variant="primary" size="lg" isLoading>Exporting...</Button>
      </div>
      <p className="text-sm text-[var(--color-text-secondary)]">
        Use primary buttons for the main action on a page or in a section.
      </p>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Primary buttons are used for the most important actions like creating, saving, or submitting.',
      },
    },
  },
}

export const SecondaryActions: Story = {
  render: () => (
    <div className="space-y-4">
      <div className="flex gap-3">
        <Button variant="secondary">Cancel</Button>
        <Button variant="outline">Learn More</Button>
        <Button variant="ghost">View Details</Button>
      </div>
      <p className="text-sm text-[var(--color-text-secondary)]">
        Secondary buttons are used for less prominent actions or alternatives to primary actions.
      </p>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Secondary button variants for supporting actions and navigation.',
      },
    },
  },
}

export const DestructiveActions: Story = {
  render: () => (
    <div className="space-y-4">
      <div className="flex gap-3">
        <Button variant="danger">Delete Architecture</Button>
        <Button variant="danger" disabled>Cannot Delete</Button>
      </div>
      <p className="text-sm text-[var(--color-text-secondary)]">
        Use danger variant for destructive actions that require user confirmation.
      </p>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Danger buttons for destructive actions like deletion or removal.',
      },
    },
  },
}

export const InForms: Story = {
  render: () => (
    <div className="max-w-md space-y-4 p-6 border border-[var(--color-border)] rounded-lg">
      <h3 className="text-lg font-semibold text-[var(--color-text-primary)]">Create New Project</h3>
      <div className="space-y-3">
        <input
          type="text"
          placeholder="Project name"
          className="w-full px-3.5 py-2.5 rounded-md border border-[var(--color-border)] bg-[var(--color-background)]"
        />
        <textarea
          placeholder="Description"
          rows={3}
          className="w-full px-3.5 py-2.5 rounded-md border border-[var(--color-border)] bg-[var(--color-background)]"
        />
      </div>
      <div className="flex gap-3 pt-2">
        <Button variant="primary">Create Project</Button>
        <Button variant="ghost">Cancel</Button>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Button usage in forms with primary action and cancel option.',
      },
    },
  },
}

export const Showcase: Story = {
  render: () => (
    <div className="space-y-6">
      <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
        <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5">
          <h4 className="text-base font-semibold mb-3">Primary & Secondary</h4>
          <div className="flex flex-wrap gap-3 items-center">
            <Button variant="primary">Primary</Button>
            <Button variant="primary" isLoading>Loading</Button>
            <Button variant="secondary">Secondary</Button>
            <Button variant="outline">Outline</Button>
            <Button variant="ghost">Ghost</Button>
          </div>
        </div>
        <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5">
          <h4 className="text-base font-semibold mb-3">Sizes</h4>
          <div className="flex flex-wrap gap-3 items-center">
            <Button size="sm">Small</Button>
            <Button size="md">Medium</Button>
            <Button size="lg">Large</Button>
          </div>
        </div>
        <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5">
          <h4 className="text-base font-semibold mb-3">States</h4>
          <div className="flex flex-wrap gap-3 items-center">
            <Button>Default</Button>
            <Button disabled>Disabled</Button>
          </div>
        </div>
        <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5">
          <h4 className="text-base font-semibold mb-3">Destructive</h4>
          <div className="flex flex-wrap gap-3 items-center">
            <Button variant="danger">Delete</Button>
            <Button variant="danger" disabled>Delete (Disabled)</Button>
          </div>
        </div>
      </div>
      <p className="text-sm text-[var(--color-text-secondary)]">
        This showcase demonstrates consistent spacing, typography, and elevations aligned with a modern product system.
      </p>
    </div>
  ),
}
