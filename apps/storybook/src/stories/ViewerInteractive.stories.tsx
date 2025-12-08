import { useState } from 'react'
import { SrujaViewerView, type ArchitectureJSON } from '../../../../packages/viewer/src/index.ts'

export default { title: 'Viewer/SrujaViewerView Interactive' }

const DATA: ArchitectureJSON = {
  metadata: { name: 'Sample', version: '1.0.0' },
  architecture: {
    systems: [
      {
        id: 'WebApp',
        label: 'Web Application',
        containers: [
          { id: 'API', label: 'API Service' },
          { id: 'DB', label: 'Database' },
        ],
      },
    ],
    persons: [{ id: 'User', label: 'End User' }],
    relations: [
      { from: 'User', to: 'WebApp.API', verb: 'Visit' },
      { from: 'WebApp.API', to: 'WebApp.DB', verb: 'Query' },
    ],
  },
}

export const Selectable = () => {
  const [selected, setSelected] = useState<string | null>(null)
  return (
    <div>
      <div style={{ height: 400 }}>
        <SrujaViewerView data={DATA} onSelect={(id) => setSelected(id)} />
      </div>
      <div style={{ padding: 8 }}>Selected: {selected || 'none'}</div>
    </div>
  )
}
