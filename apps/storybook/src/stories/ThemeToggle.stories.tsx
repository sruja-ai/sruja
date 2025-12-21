import { ThemeToggle } from '../../../../packages/ui/src/components/ThemeToggle'
import type { Meta, StoryObj } from '@storybook/react'

const meta: Meta<typeof ThemeToggle> = {
  title: 'Components/ThemeToggle',
  component: ThemeToggle,
  tags: ['autodocs'],
  parameters: { layout: 'centered' },
  argTypes: {
    iconOnly: { control: { type: 'boolean' } },
    size: {
      control: { type: 'select' },
      options: ['sm', 'md', 'lg'],
    },
  },
}

export default meta
type Story = StoryObj<typeof ThemeToggle>

export const Default: Story = {
  args: {
    iconOnly: true,
    size: 'sm',
  },
}

export const Showcase: Story = {
  render: () => (
    <div className="grid grid-cols-3 gap-4">
      <ThemeToggle iconOnly size="sm" />
      <ThemeToggle iconOnly size="md" />
      <ThemeToggle iconOnly size="lg" />
    </div>
  ),
}
