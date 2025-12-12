import type { CSSProperties } from 'react'

const item: CSSProperties = { display: 'flex', alignItems: 'center', gap: 8 }
const swatch: CSSProperties = { width: 16, height: 12, borderRadius: 2, border: '1px solid #9ca3af' }
const pill: CSSProperties = { padding: '2px 6px', border: '1px solid #d1d5db', borderRadius: 6, background: 'rgba(255,255,255,0.8)', fontSize: 12 }

export function Legend() {
  return (
    <div style={{ position: 'absolute', right: 12, bottom: 12, background: 'rgba(17,24,39,0.8)', color: '#e5e7eb', padding: 12, borderRadius: 8, fontSize: 12 }}>
      <div style={{ fontWeight: 600, marginBottom: 8 }}>Legend</div>
      <div style={item}><div style={{ ...swatch, background: '#08427B' }} /> Person</div>
      <div style={item}><div style={{ ...swatch, background: '#1168BD' }} /> System</div>
      <div style={item}><div style={{ ...swatch, background: '#438DD5' }} /> Container</div>
      <div style={item}><div style={{ ...swatch, background: '#85BBF0' }} /> Component</div>
      <div style={item}><div style={{ ...swatch, background: 'transparent', borderStyle: 'dashed' }} /> Boundary</div>
      <div style={{ marginTop: 8 }}><span style={pill}>verb</span> Edge label</div>
    </div>
  )
}

