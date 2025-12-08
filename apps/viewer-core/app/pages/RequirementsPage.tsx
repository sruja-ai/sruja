import { ShieldCheck } from 'lucide-react'

export function RequirementsPage({ requirements, onHighlight }: { requirements: any[], onHighlight: (ids: string[]) => void }) {
  if (requirements.length === 0) {
    return (
      <div className="page-content" style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
        <div style={{ textAlign: 'center', color: 'var(--text-muted)' }}>
          <ShieldCheck size={48} style={{ marginBottom: 16, opacity: 0.5 }} />
          <h3>No requirements defined</h3>
        </div>
      </div>
    )
  }

  return (
    <div className="page-content">
      <h2>Requirements</h2>
      <div className="cards-grid">
        {requirements.map(req => (
          <div
            key={req.id}
            className="req-card"
            onClick={() => onHighlight([])}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: 8 }}>
              <h3>{req.id}: {req.title || req.description || ''}</h3>
              <span className={`type-badge type-functional`}>{req.type || 'functional'}</span>
            </div>
            {req.description && <p style={{ fontSize: 14, color: 'var(--text-secondary)', margin: 0 }}>{req.description}</p>}
          </div>
        ))}
      </div>
    </div>
  )
}

