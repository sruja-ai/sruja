// apps/storybook/src/stories/SrujaLoader.stories.tsx
import { SrujaLoader } from '../../../../packages/ui/src/components/SrujaLoader';
import type { Meta, StoryObj } from '@storybook/react';

const meta: Meta<typeof SrujaLoader> = {
  title: 'Components/SrujaLoader',
  component: SrujaLoader,
  tags: ['autodocs'],
  parameters: {
    docs: {
      description: {
        component: 'An animated loading spinner component featuring the Sruja logo. Used throughout the application to indicate loading states.',
      },
    },
  },
  argTypes: {
    size: {
      control: { type: 'number', min: 16, max: 128, step: 8 },
      description: 'Size of the loader in pixels',
      table: {
        type: { summary: 'number' },
        defaultValue: { summary: '48' },
      },
    },
    className: {
      control: { type: 'text' },
      description: 'Additional CSS classes',
      table: {
        type: { summary: 'string' },
      },
    },
  },
};

export default meta;
type Story = StoryObj<typeof SrujaLoader>;

export const Default: Story = {
  args: {
    size: 48,
  },
};

export const Small: Story = {
  args: {
    size: 24,
  },
};

export const Medium: Story = {
  args: {
    size: 48,
  },
};

export const Large: Story = {
  args: {
    size: 64,
  },
};

export const ExtraLarge: Story = {
  args: {
    size: 96,
  },
};

export const WithCustomClass: Story = {
  args: {
    size: 48,
    className: 'border-2 border-primary rounded-lg p-4',
  },
};

export const InContext: Story = {
  render: () => (
    <div style={{ padding: '2rem', textAlign: 'center' }}>
      <div style={{ marginBottom: '1rem', color: '#64748b' }}>
        Loading your architecture...
      </div>
      <SrujaLoader size={48} />
    </div>
  ),
};

export const Inline: Story = {
  render: () => (
    <div style={{ padding: '1rem' }}>
      <p style={{ display: 'inline-block', marginRight: '0.5rem' }}>
        Processing
      </p>
      <SrujaLoader size={16} />
    </div>
  ),
};

