import { useState } from 'react'
import { SearchDialog, type SearchItem } from '../../../../packages/ui/src/components/SearchDialog'

export default { title: 'Search/SearchDialog', component: SearchDialog, tags: ['autodocs'] }

const DATA: SearchItem[] = [
  { id: 'api', label: 'API Service', subLabel: 'WebApp' },
  { id: 'db', label: 'Database', subLabel: 'WebApp' },
  { id: 'user', label: 'End User', subLabel: 'Actor' },
]

async function fetchResults(q: string): Promise<SearchItem[]> {
  await new Promise((r) => setTimeout(r, 200))
  const s = q.trim().toLowerCase()
  if (!s) return []
  return DATA.filter((i) => i.label.toLowerCase().includes(s) || (i.subLabel || '').toLowerCase().includes(s))
}

export const Basic = () => {
  const [open, setOpen] = useState(true)
  const [sel, setSel] = useState<SearchItem | null>(null)
  return (
    <div>
      <button onClick={() => setOpen(true)}>Open</button>
      <SearchDialog isOpen={open} onClose={() => setOpen(false)} fetchResults={fetchResults} onSelect={setSel} />
      <div style={{ marginTop: 8 }}>Selected: {sel ? sel.label : 'None'}</div>
    </div>
  )
}

