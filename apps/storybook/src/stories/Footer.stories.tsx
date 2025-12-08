// apps/storybook/src/stories/Footer.stories.tsx
import { Footer } from '../../../../packages/ui/src/components/Footer'
import { Badge } from '../../../../packages/ui/src/components/Badge'
import type { Meta, StoryObj } from '@storybook/react'

const meta: Meta<typeof Footer> = {
  title: 'Components/Footer',
  component: Footer,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: 'Application footer component used in Sruja Studio. Displays copyright, branding, and links. Supports custom left, center, and right content areas for flexible layouts.',
      },
    },
  },
  argTypes: {
    leftContent: {
      description: 'Content displayed on the left side',
    },
    centerContent: {
      description: 'Content displayed in the center',
    },
    rightContent: {
      description: 'Content displayed on the right side',
    },
  },
}

export default meta
type Story = StoryObj<typeof Footer>

export const Playground: Story = {
  args: {
    leftContent: <span>© {new Date().getFullYear()} Sruja</span>,
    centerContent: <span>Architecture as Code</span>,
    rightContent: <a href="https://sruja.ai" target="_blank" rel="noopener noreferrer" style={{ color: 'inherit', textDecoration: 'none' }}>sruja.ai</a>,
  },
}

export const StudioFooter: Story = {
  args: {
    leftContent: <span>© {new Date().getFullYear()} Sruja</span>,
    centerContent: <span>Architecture as Code</span>,
    rightContent: (
      <a
        href="https://sruja.ai"
        target="_blank"
        rel="noopener noreferrer"
        style={{ color: 'inherit', textDecoration: 'none', display: 'flex', alignItems: 'center', gap: 8 }}
      >
        sruja.ai
      </a>
    ),
  },
  parameters: {
    docs: {
      description: {
        story: 'Footer layout as used in Sruja Studio with branding and links.',
      },
    },
  },
}

export const WithStatus: Story = {
  args: {
    leftContent: (
      <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
        <span>© {new Date().getFullYear()} Sruja</span>
        <Badge color="success" style={{ fontSize: 11 }}>v0.1.0</Badge>
      </div>
    ),
    centerContent: <span>Architecture as Code</span>,
    rightContent: (
      <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
        <a href="https://sruja.ai" target="_blank" rel="noopener noreferrer" style={{ color: 'inherit', textDecoration: 'none' }}>
          Documentation
        </a>
        <a href="https://sruja.ai" target="_blank" rel="noopener noreferrer" style={{ color: 'inherit', textDecoration: 'none' }}>
          sruja.ai
        </a>
      </div>
    ),
  },
  parameters: {
    docs: {
      description: {
        story: 'Footer with version badge and additional links.',
      },
    },
  },
}

export const Minimal: Story = {
  args: {
    leftContent: <span>© {new Date().getFullYear()} Sruja</span>,
    centerContent: <div />,
    rightContent: <div />,
  },
  parameters: {
    docs: {
      description: {
        story: 'Minimal footer with just copyright information.',
      },
    },
  },
}

export const Showcase: Story = {
  render: () => (
    <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5 max-w-3xl">
      <Footer
        leftContent={<span>© {new Date().getFullYear()} Sruja</span>}
        centerContent={<span>Architecture as Code</span>}
        rightContent={<a href="https://sruja.ai" target="_blank" rel="noopener noreferrer" className="text-[var(--color-primary)]">sruja.ai</a>}
      />
    </div>
  ),
}
