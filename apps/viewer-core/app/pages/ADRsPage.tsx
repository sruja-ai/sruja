import { FileText } from 'lucide-react'

export function ADRsPage({ adrs, onHighlight }: { adrs: any[], onHighlight: (ids: string[]) => void }) {
  if (adrs.length === 0) {
    return (
      <div className="page-content" style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
        <div style={{ textAlign: 'center', color: 'var(--text-muted)' }}>
          <FileText size={48} style={{ marginBottom: 16, opacity: 0.5 }} />
          <h3>No ADRs defined</h3>
        </div>
      </div>
    )
  }

  const getStatusClass = (status: string) => {
    const s = status.toLowerCase()
    if (s === 'accepted') return 'status-accepted'
    if (s === 'proposed') return 'status-proposed'
    return 'status-proposed'
  }

  return (
    <div className="page-content">
      <h2>Architecture Decision Records</h2>
      <div className="cards-grid">
        {adrs.map(adr => {
          const status = (adr.status || 'proposed').toLowerCase()
          return (
            <div
              key={adr.id}
              className="adr-card"
              onClick={() => onHighlight([])}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: 12 }}>
                <h3>{adr.id}: {adr.title || ''}</h3>
                {adr.status && <span className={`status-badge ${getStatusClass(adr.status)}`}>{adr.status}</span>}
              </div>

              <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                {adr.context && (
                  <div className="section">
                    <h4>Context</h4>
                    <p>{adr.context}</p>
                  </div>
                )}
                {adr.decision && (
                  <div className="section">
                    <h4>Decision</h4>
                    <p>{adr.decision}</p>
                  </div>
                )}
                {adr.consequences && (
                  <div className="section">
                    <h4>Consequences</h4>
                    <p>{adr.consequences}</p>
                  </div>
                )}
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}

