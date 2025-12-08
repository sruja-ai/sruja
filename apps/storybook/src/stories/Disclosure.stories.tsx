// apps/storybook/src/stories/Disclosure.stories.tsx
import { Disclosure } from '../../../../packages/ui/src/components/Disclosure';
import type { Meta, StoryObj } from '@storybook/react';

const meta: Meta<typeof Disclosure> = {
  title: 'Components/Disclosure',
  component: Disclosure,
  tags: ['autodocs'],
  parameters: {
    docs: {
      description: {
        component: 'A collapsible disclosure component for showing and hiding content. Built on Headless UI.',
      },
    },
  },
  argTypes: {
    title: {
      control: { type: 'text' },
      description: 'Title text for the disclosure button',
    },
    defaultOpen: {
      control: { type: 'boolean' },
      description: 'Whether the disclosure is open by default',
      table: {
        defaultValue: { summary: 'false' },
      },
    },
  },
};

export default meta;
type Story = StoryObj<typeof Disclosure>;

export const Default: Story = {
  args: {
    title: 'Click to expand',
    children: 'This is the hidden content that appears when you click the disclosure button.',
  },
};

export const DefaultOpen: Story = {
  args: {
    title: 'Already expanded',
    defaultOpen: true,
    children: 'This content is visible by default.',
  },
};

export const WithLongContent: Story = {
  args: {
    title: 'Long content example',
    children: (
      <div>
        <p>This disclosure contains multiple paragraphs of content.</p>
        <p>You can put any React content here, including lists, code blocks, and more.</p>
        <ul>
          <li>Item one</li>
          <li>Item two</li>
          <li>Item three</li>
        </ul>
      </div>
    ),
  },
};

export const Multiple: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
      <Disclosure title="First section">
        Content for the first section
      </Disclosure>
      <Disclosure title="Second section" defaultOpen>
        Content for the second section (open by default)
      </Disclosure>
      <Disclosure title="Third section">
        Content for the third section
      </Disclosure>
    </div>
  ),
};

