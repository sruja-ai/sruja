// apps/storybook/src/stories/Logo.stories.tsx
import { Logo } from '../../../../packages/ui/src/components/Logo';
import type { Meta, StoryObj } from '@storybook/react';

const meta: Meta<typeof Logo> = {
  title: 'Components/Logo',
  component: Logo,
  tags: ['autodocs'],
  parameters: {
    docs: {
      description: {
        component: 'The Sruja logo component. Can be displayed with or without a rotation animation for loading states.',
      },
    },
  },
  argTypes: {
    size: {
      control: { type: 'number', min: 16, max: 128, step: 8 },
      description: 'Size of the logo in pixels',
      table: {
        type: { summary: 'number' },
        defaultValue: { summary: '32' },
      },
    },
    isLoading: {
      control: { type: 'boolean' },
      description: 'Whether to show the logo with rotation animation',
      table: {
        defaultValue: { summary: 'false' },
      },
    },
    className: {
      control: { type: 'text' },
      description: 'Additional CSS classes',
    },
    alt: {
      control: { type: 'text' },
      description: 'Alt text for accessibility',
      table: {
        defaultValue: { summary: 'Sruja Logo' },
      },
    },
  },
};

export default meta;
type Story = StoryObj<typeof Logo>;

export const Default: Story = {
  args: {
    size: 32,
  },
};

export const Small: Story = {
  args: {
    size: 16,
  },
};

export const Medium: Story = {
  args: {
    size: 32,
  },
};

export const Large: Story = {
  args: {
    size: 64,
  },
};

export const WithAnimation: Story = {
  args: {
    size: 48,
    isLoading: true,
  },
};

export const InContext: Story = {
  render: () => (
    <div style={{ padding: '2rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
      <Logo size={32} />
      <span style={{ fontSize: '1.5rem', fontWeight: 'bold' }}>Sruja</span>
    </div>
  ),
};

export const LoadingState: Story = {
  render: () => (
    <div style={{ padding: '2rem', textAlign: 'center' }}>
      <Logo size={48} isLoading />
      <p style={{ marginTop: '1rem', color: '#64748b' }}>Loading...</p>
    </div>
  ),
};

