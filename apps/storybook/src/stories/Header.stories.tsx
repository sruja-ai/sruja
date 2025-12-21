// apps/storybook/src/stories/Header.stories.tsx
import { Header } from '../../../../packages/ui/src/components/Header'
import { ThemeToggle } from '../../../../packages/ui/src/components/ThemeToggle'
import { Button } from '../../../../packages/ui/src/components/Button'
import { Download } from 'lucide-react'
import type { Meta, StoryObj } from '@storybook/react'

const meta: Meta<typeof Header> = {
  title: 'Components/Header',
  component: Header,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: 'Application header component used in Sruja Studio. Displays branding, version information, and action buttons. Supports custom left and right content areas for flexible layouts.',
      },
    },
  },
  argTypes: {
    title: {
      control: { type: 'text' },
      description: 'Main application title',
    },
    subtitle: {
      control: { type: 'text' },
      description: 'Subtitle or tagline',
    },
    version: {
      control: { type: 'text' },
      description: 'Version number displayed in header',
    },
    logoLoading: {
      control: { type: 'boolean' },
      description: 'Shows loading state for logo',
    },
  },
}

export default meta
type Story = StoryObj<typeof Header>

export const Playground: Story = {
  args: {
    title: 'Sruja Studio',
    subtitle: 'Architecture Visualization Tool',
    version: '0.1.0',
    leftContent: <div />,
    rightContent: <ThemeToggle iconOnly size="sm" />,
  },
}

export const StudioHeader: Story = {
  args: {
    title: 'Sruja Studio',
    subtitle: 'Architecture Visualization Tool',
    version: '0.1.0',
    logoLoading: false,
    logoSize: 32,
    leftContent: (
      <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
        <label style={{ fontSize: 14, fontWeight: 500, color: 'var(--color-text-secondary)' }}>
          Example:
        </label>
        <select
          style={{
            padding: '6px 12px',
            borderRadius: 6,
            border: '1px solid var(--color-border)',
            fontSize: 14,
            backgroundColor: 'var(--color-background)',
            color: 'var(--color-text-primary)',
          }}
        >
          <option>Simple Web App</option>
          <option>E-commerce Platform</option>
          <option>Microservices</option>
        </select>
      </div>
    ),
    rightContent: (
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <Button variant="ghost" size="sm">
          Preview Markdown
        </Button>
        <Button size="sm" style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
          <Download size={16} />
          Export
        </Button>
        <ThemeToggle iconOnly size="sm" />
      </div>
    ),
  },
  parameters: {
    docs: {
      description: {
        story: 'Header layout as used in Sruja Studio with example selector and action buttons.',
      },
    },
  },
}

export const Showcase: Story = {
  render: () => (
    <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5 max-w-3xl">
      <Header
        title="Sruja Studio"
        subtitle="Architecture Visualization Tool"
        version="0.1.0"
        leftContent={<div />}
        rightContent={<ThemeToggle iconOnly size="sm" />}
      />
    </div>
  ),
}

export const Minimal: Story = {
  args: {
    title: 'Sruja',
    subtitle: 'Architecture as Code',
    version: '0.1.0',
    leftContent: <div />,
    rightContent: <ThemeToggle iconOnly size="sm" />,
  },
  parameters: {
    docs: {
      description: {
        story: 'Minimal header with just branding and theme toggle.',
      },
    },
  },
}

export const WithLoading: Story = {
  args: {
    title: 'Sruja Studio',
    subtitle: 'Loading architecture...',
    version: '0.1.0',
    logoLoading: true,
    leftContent: <div />,
    rightContent: <ThemeToggle iconOnly size="sm" />,
  },
  parameters: {
    docs: {
      description: {
        story: 'Header with loading state while initializing WASM or parsing architecture.',
      },
    },
  },
}
