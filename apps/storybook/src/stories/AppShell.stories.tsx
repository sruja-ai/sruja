// apps/storybook/src/stories/AppShell.stories.tsx
import { AppShell } from '../../../../packages/ui/src/components/AppShell'
import { Header } from '../../../../packages/ui/src/components/Header'
import { Footer } from '../../../../packages/ui/src/components/Footer'
import { ThemeToggle } from '../../../../packages/ui/src/components/ThemeToggle'
import { Button } from '../../../../packages/ui/src/components/Button'
import { Card } from '../../../../packages/ui/src/components/Card'
import { Badge } from '../../../../packages/ui/src/components/Badge'
import { Download } from 'lucide-react'
import type { Meta, StoryObj } from '@storybook/react'

const meta: Meta<typeof AppShell> = {
  title: 'Components/AppShell',
  component: AppShell,
  tags: ['autodocs'],
  parameters: {
    docs: {
      description: {
        component: 'Complete application shell layout used in Sruja Studio. Provides header, sidebar, main content area, and footer. This is the foundation for the full Studio application interface.',
      },
    },
    layout: 'fullscreen',
  },
}

export default meta
type Story = StoryObj<typeof AppShell>

export const Playground: Story = {
  render: () => (
    <div style={{ height: '600px', border: '1px solid var(--color-border)', borderRadius: 8, overflow: 'hidden' }}>
      <AppShell
        header={
          <Header
            title="Sruja Studio"
            subtitle="Architecture Visualization Tool"
            version="0.1.0"
            leftContent={<div />}
            rightContent={<ThemeToggle iconOnly size="sm" />}
          />
        }
        sidebar={
          <div style={{ padding: 16 }}>
            <h3 style={{ fontSize: 12, fontWeight: 600, textTransform: 'uppercase', color: 'var(--color-text-secondary)', marginBottom: 12 }}>
              Explorer
            </h3>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
              <Button variant="ghost" size="sm" style={{ justifyContent: 'flex-start' }}>Web Application</Button>
              <Button variant="ghost" size="sm" style={{ justifyContent: 'flex-start' }}>API Service</Button>
              <Button variant="ghost" size="sm" style={{ justifyContent: 'flex-start' }}>Database</Button>
            </div>
          </div>
        }
        footer={
          <Footer
            leftContent={<span>© {new Date().getFullYear()} Sruja</span>}
            centerContent={<span>Architecture as Code</span>}
            rightContent={<a href="https://sruja.ai" target="_blank" rel="noopener noreferrer">sruja.ai</a>}
          />
        }
      >
        <div style={{ padding: 24 }}>
          <h2 style={{ marginTop: 0, marginBottom: 16 }}>Architecture Overview</h2>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))', gap: 16 }}>
            <Card title="Systems" footer={<Badge color="brand">3</Badge>}>
              <p style={{ fontSize: 14, color: 'var(--color-text-secondary)', margin: 0 }}>
                Total systems defined in architecture
              </p>
            </Card>
            <Card title="Containers" footer={<Badge>12</Badge>}>
              <p style={{ fontSize: 14, color: 'var(--color-text-secondary)', margin: 0 }}>
                Container components across all systems
              </p>
            </Card>
            <Card title="Relations" footer={<Badge color="info">24</Badge>}>
              <p style={{ fontSize: 14, color: 'var(--color-text-secondary)', margin: 0 }}>
                Connections between elements
              </p>
            </Card>
          </div>
        </div>
      </AppShell>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Complete application shell with header, sidebar navigation, main content, and footer.',
      },
    },
  },
}

export const StudioLayout: Story = {
  render: () => (
    <div style={{ height: '700px', border: '1px solid var(--color-border)', borderRadius: 8, overflow: 'hidden' }}>
      <AppShell
        header={
          <Header
            title="Sruja Studio"
            subtitle="Architecture Visualization Tool"
            version="0.1.0"
            leftContent={
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
                  }}
                >
                  <option>Simple Web App</option>
                  <option>E-commerce Platform</option>
                </select>
              </div>
            }
            rightContent={
              <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <Button variant="ghost" size="sm">Preview</Button>
                <Button size="sm" style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                  <Download size={16} />
                  Export
                </Button>
                <ThemeToggle iconOnly size="sm" />
              </div>
            }
          />
        }
        sidebar={
          <div style={{ padding: 16, height: '100%', borderRight: '1px solid var(--color-border)' }}>
            <h3 style={{ fontSize: 12, fontWeight: 600, textTransform: 'uppercase', color: 'var(--color-text-secondary)', marginBottom: 16 }}>
              Model Explorer
            </h3>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
              <div style={{ padding: 8, backgroundColor: 'var(--color-surface)', borderRadius: 6 }}>
                <div style={{ fontWeight: 600, fontSize: 14, marginBottom: 4 }}>Web Application</div>
                <div style={{ fontSize: 12, color: 'var(--color-text-secondary)' }}>System</div>
              </div>
              <div style={{ paddingLeft: 16 }}>
                <div style={{ padding: 6, fontSize: 13 }}>API Service</div>
                <div style={{ padding: 6, fontSize: 13 }}>Database</div>
              </div>
            </div>
          </div>
        }
        footer={
          <Footer
            leftContent={<span>© {new Date().getFullYear()} Sruja</span>}
            centerContent={<span>Architecture as Code</span>}
            rightContent={<a href="https://sruja.ai" target="_blank" rel="noopener noreferrer">sruja.ai</a>}
          />
        }
      >
        <div style={{ padding: 24, height: '100%', display: 'flex', flexDirection: 'column', gap: 16 }}>
          <div style={{ flex: 1, border: '1px dashed var(--color-border)', borderRadius: 8, display: 'flex', alignItems: 'center', justifyContent: 'center', backgroundColor: 'var(--color-surface)' }}>
            <div style={{ textAlign: 'center', color: 'var(--color-text-secondary)' }}>
              <div style={{ fontSize: 18, fontWeight: 600, marginBottom: 8 }}>Architecture Diagram</div>
              <div style={{ fontSize: 14 }}>Interactive viewer would appear here</div>
            </div>
          </div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 12 }}>
            <Card title="Elements" footer={<Badge>15</Badge>}>
              <div style={{ fontSize: 24, fontWeight: 600 }}>15</div>
            </Card>
            <Card title="Relations" footer={<Badge color="info">8</Badge>}>
              <div style={{ fontSize: 24, fontWeight: 600 }}>8</div>
            </Card>
            <Card title="Status" footer={<Badge color="success">Valid</Badge>}>
              <div style={{ fontSize: 14, color: 'var(--color-text-secondary)' }}>No errors</div>
            </Card>
          </div>
        </div>
      </AppShell>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Complete Studio layout with model explorer sidebar and architecture viewer.',
      },
    },
  },
}
