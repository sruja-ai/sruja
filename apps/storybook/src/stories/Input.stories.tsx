// apps/storybook/src/stories/Input.stories.tsx
import { Input } from '../../../../packages/ui/src/components/Input'
import type { Meta, StoryObj } from '@storybook/react'

const meta: Meta<typeof Input> = {
  title: 'Components/Input',
  component: Input,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component:
          'Enterprise-grade input with accessible label, helper and error messaging, and consistent focus styling aligned to the design system.',
      },
    },
  },
  argTypes: {
    label: {
      control: { type: 'text' },
      description: 'Label text displayed above the input',
    },
    placeholder: {
      control: { type: 'text' },
      description: 'Placeholder text shown when input is empty',
    },
    helperText: {
      control: { type: 'text' },
      description: 'Helper text displayed below the input',
    },
    error: {
      control: { type: 'text' },
      description: 'Error message displayed below the input (overrides helperText)',
    },
    disabled: {
      control: { type: 'boolean' },
      description: 'Disables the input',
    },
    type: {
      control: { type: 'select' },
      options: ['text', 'email', 'password', 'number', 'tel', 'url'],
      description: 'Input type',
    },
  },
}

export default meta
type Story = StoryObj<typeof Input>

export const Playground: Story = {
  args: {
    label: 'Email',
    placeholder: 'you@example.com',
    helperText: 'We will never share your email.',
  },
}

export const Basic: Story = {
  args: {
    label: 'Project Name',
    placeholder: 'Enter project name',
    helperText: 'Choose a unique name for your project',
  },
  parameters: {
    docs: {
      description: {
        story: 'Basic input with label and helper text.',
      },
    },
  },
}

export const WithError: Story = {
  args: {
    label: 'Email Address',
    placeholder: 'you@example.com',
    error: 'Please enter a valid email address',
    defaultValue: 'invalid-email',
  },
  parameters: {
    docs: {
      description: {
        story: 'Input in error state with validation message.',
      },
    },
  },
}

export const InputTypes: Story = {
  render: () => (
    <div className="space-y-4 max-w-md">
      <Input
        type="text"
        label="Text Input"
        placeholder="Enter text"
        helperText="Standard text input"
      />
      <Input
        type="email"
        label="Email Input"
        placeholder="user@example.com"
        helperText="Email address with validation"
      />
      <Input
        type="password"
        label="Password"
        placeholder="Enter password"
        helperText="At least 8 characters"
      />
      <Input
        type="number"
        label="Number Input"
        placeholder="0"
        helperText="Numeric value only"
      />
      <Input
        type="url"
        label="Website URL"
        placeholder="https://example.com"
        helperText="Full URL including protocol"
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Different input types for various data formats.',
      },
    },
  },
}

export const States: Story = {
  render: () => (
    <div className="space-y-4 max-w-md">
      <Input
        label="Default State"
        placeholder="Type here..."
        helperText="Normal input state"
      />
      <Input
        label="With Value"
        defaultValue="Architecture as Code"
        helperText="Input with pre-filled value"
      />
      <Input
        label="Error State"
        defaultValue="invalid"
        error="This field is required"
      />
      <Input
        label="Disabled State"
        defaultValue="Cannot edit"
        disabled
        helperText="This input is disabled"
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'All input states: default, with value, error, and disabled.',
      },
    },
  },
}

export const InForm: Story = {
  render: () => (
    <form className="space-y-4 max-w-md p-6 border border-[var(--color-border)] rounded-lg">
      <h3 className="text-lg font-semibold text-[var(--color-text-primary)] mb-4">
        Create New Architecture
      </h3>
      <Input
        label="Architecture Name"
        placeholder="e.g., E-commerce Platform"
        helperText="Choose a descriptive name"
        required
      />
      <Input
        type="email"
        label="Contact Email"
        placeholder="architect@example.com"
        helperText="For notifications and updates"
      />
      <Input
        label="Description"
        placeholder="Brief description of the architecture"
        helperText="Optional description"
      />
      <div className="flex gap-3 pt-2">
        <button
          type="submit"
          className="px-5 py-2.5 bg-[var(--color-primary)] text-white rounded-md hover:opacity-90"
        >
          Create
        </button>
        <button
          type="button"
          className="px-5 py-2.5 bg-transparent border border-[var(--color-border)] rounded-md"
        >
          Cancel
        </button>
      </div>
    </form>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Input components used in a complete form context.',
      },
    },
  },
}

export const Showcase: Story = {
  render: () => (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6 max-w-2xl">
      <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5">
        <h4 className="text-base font-semibold mb-3">States</h4>
        <div className="space-y-3">
          <Input label="Default" placeholder="Type here" helperText="Helper" />
          <Input label="Error" defaultValue="invalid" error="Required" />
          <Input label="Disabled" defaultValue="Cannot edit" disabled />
        </div>
      </div>
      <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5">
        <h4 className="text-base font-semibold mb-3">Types</h4>
        <div className="space-y-3">
          <Input type="email" label="Email" placeholder="you@example.com" />
          <Input type="password" label="Password" placeholder="••••••••" />
          <Input type="number" label="Number" placeholder="0" />
        </div>
      </div>
    </div>
  ),
}
