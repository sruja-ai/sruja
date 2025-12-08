import { Monitor } from 'lucide-react'

export function SystemsPage({ systems, onSelect }: { systems: any[], onSelect: (id: string) => void }) {
  if (systems.length === 0) {
    return (
      <div className="page-content" style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
        <div style={{ textAlign: 'center', color: 'var(--text-muted)' }}>
          <Monitor size={48} style={{ marginBottom: 16, opacity: 0.5 }} />
          <h3>No systems defined</h3>
        </div>
      </div>
    )
  }

  return (
    <div className="page-content">
      <h2>Systems</h2>
      <div className="cards-grid">
        {systems.map(sys => (
          <div key={sys.id} className="scenario-card" onClick={() => onSelect(sys.id)}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 12 }}>
              <div style={{ width: 40, height: 40, borderRadius: 8, backgroundColor: 'var(--bg-tertiary)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                <Monitor size={20} color="var(--accent-primary)" />
              </div>
              <div>
                <h3 style={{ margin: 0 }}>{sys.label || sys.id}</h3>
                <span style={{ fontSize: 12, color: 'var(--text-muted)' }}>{sys.id}</span>
              </div>
            </div>

            {sys.description && (
              <p style={{ fontSize: 14, color: 'var(--text-secondary)', margin: '0 0 16px 0' }}>{sys.description}</p>
            )}

            <div style={{ display: 'flex', gap: 12, marginTop: 'auto', borderTop: '1px solid var(--border-color)', paddingTop: 12 }}>
              <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', flex: 1 }}>
                <span style={{ fontSize: 18, fontWeight: 700, color: 'var(--text-primary)' }}>{sys.containers?.length || 0}</span>
                <span style={{ fontSize: 11, color: 'var(--text-muted)', textTransform: 'uppercase' }}>Containers</span>
              </div>
              <div style={{ width: 1, backgroundColor: 'var(--border-color)' }} />
              <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', flex: 1 }}>
                <span style={{ fontSize: 18, fontWeight: 700, color: 'var(--text-primary)' }}>
                  {(sys.containers || []).reduce((acc: number, c: any) => acc + (c.components?.length || 0), 0)}
                </span>
                <span style={{ fontSize: 11, color: 'var(--text-muted)', textTransform: 'uppercase' }}>Components</span>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}

