// apps/storybook/src/stories/SearchBar.stories.tsx
import { useMemo, useState } from 'react'
import { SearchBar, type SearchItem } from '../../../../packages/ui/src/components/SearchBar'
import { Badge } from '../../../../packages/ui/src/components/Badge'
import type { Meta, StoryObj } from '@storybook/react'

const meta: Meta<typeof SearchBar> = {
  title: 'Components/SearchBar',
  component: SearchBar,
  tags: ['autodocs'],
  parameters: {
    docs: {
      description: {
        component: 'Search bar component for finding architecture elements. Used in Sruja Studio for quickly locating systems, containers, and other elements in large architectures. Supports keyboard navigation and filtering.',
      },
    },
  },
  argTypes: {
    query: {
      control: { type: 'text' },
      description: 'Current search query',
    },
    onQueryChange: {
      action: 'query changed',
      description: 'Callback fired when search query changes',
    },
    onSelect: {
      action: 'selected',
      description: 'Callback fired when a result is selected',
    },
  },
}

export default meta
type Story = StoryObj<typeof SearchBar>

const ARCHITECTURE_ELEMENTS: SearchItem[] = [
  { id: 'customer', label: 'Customer', subLabel: 'Person' },
  { id: 'admin', label: 'Administrator', subLabel: 'Person' },
  { id: 'ecommerce', label: 'E-commerce Platform', subLabel: 'System' },
  { id: 'webapp', label: 'Web Application', subLabel: 'E-commerce Platform > Container' },
  { id: 'api', label: 'API Gateway', subLabel: 'E-commerce Platform > Container' },
  { id: 'payment', label: 'Payment Service', subLabel: 'E-commerce Platform > Container' },
  { id: 'userdb', label: 'User Database', subLabel: 'E-commerce Platform > Datastore' },
  { id: 'productdb', label: 'Product Database', subLabel: 'E-commerce Platform > Datastore' },
]

export const Playground: Story = {
  render: function PlaygroundComponent() {
    const [q, setQ] = useState('')
    const [sel, setSel] = useState<SearchItem | null>(null)
    const results = useMemo(() => {
      const s = q.trim().toLowerCase()
      if (!s) return []
      return ARCHITECTURE_ELEMENTS.filter(
        (i) => i.label.toLowerCase().includes(s) || (i.subLabel || '').toLowerCase().includes(s)
      )
    }, [q])
    return (
      <div style={{ display: 'flex', flexDirection: 'column', gap: 16, maxWidth: 600 }}>
        <SearchBar query={q} onQueryChange={setQ} results={results} onSelect={setSel} />
        {sel && (
          <div style={{ padding: 12, backgroundColor: 'var(--color-surface)', borderRadius: 6, fontSize: 14 }}>
            <strong>Selected:</strong> {sel.label} <Badge color="info" style={{ marginLeft: 8, fontSize: 11 }}>{sel.subLabel}</Badge>
          </div>
        )}
      </div>
    )
  },
}

export const Basic: Story = {
  render: function BasicComponent() {
    const [q, setQ] = useState('')
    const results = useMemo(() => {
      const s = q.trim().toLowerCase()
      if (!s) return []
      return ARCHITECTURE_ELEMENTS.filter(
        (i) => i.label.toLowerCase().includes(s) || (i.subLabel || '').toLowerCase().includes(s)
      )
    }, [q])
    return (
      <div style={{ maxWidth: 500 }}>
        <SearchBar query={q} onQueryChange={setQ} results={results} onSelect={(item) => alert(`Selected: ${item.label}`)} />
      </div>
    )
  },
  parameters: {
    docs: {
      description: {
        story: 'Basic search bar for finding architecture elements by name or type.',
      },
    },
  },
}

export const WithResults: Story = {
  render: function WithResultsComponent() {
    const [q, setQ] = useState('api')
    const results = useMemo(() => {
      const s = q.trim().toLowerCase()
      if (!s) return []
      return ARCHITECTURE_ELEMENTS.filter(
        (i) => i.label.toLowerCase().includes(s) || (i.subLabel || '').toLowerCase().includes(s)
      )
    }, [q])
    return (
      <div style={{ maxWidth: 500 }}>
        <SearchBar query={q} onQueryChange={setQ} results={results} onSelect={(item) => alert(`Selected: ${item.label}`)} />
        {results.length > 0 && (
          <div style={{ marginTop: 8, fontSize: 12, color: 'var(--color-text-secondary)' }}>
            Found {results.length} {results.length === 1 ? 'result' : 'results'}
          </div>
        )}
      </div>
    )
  },
  parameters: {
    docs: {
      description: {
        story: 'Search bar with pre-filled query showing filtered results.',
      },
    },
  },
}
