import React from 'react'
import type { C4LayoutResult } from '@sruja/layout'

export function LayoutSVG({ result, width = 1200, height = 800 }: { result: C4LayoutResult; width?: number; height?: number }) {
  return (
    <svg width={width} height={height} viewBox={`0 0 ${width} ${height}`} style={{ background: '#fff', border: '1px solid #eee' }}>
      {Array.from(result.nodes.values()).map((n) => (
        <g key={n.nodeId}>
          <rect x={n.bbox.x} y={n.bbox.y} width={n.bbox.width} height={n.bbox.height} rx={8} ry={8} fill="#f7f7f7" stroke="#333" />
          <text x={n.bbox.x + 8} y={n.bbox.y + 20} fontSize={14} fill="#111">{n.nodeId}</text>
        </g>
      ))}
      {result.relationships.map((e) => (
        <g key={e.relationshipId}>
          {e.controlPoints ? (
            <path d={bezierPath(e)} fill="none" stroke="#1e6fff" strokeWidth={2} />
          ) : (
            e.points.slice(1).map((p, i) => {
              const prev = e.points[i]
              return <line key={i} x1={prev.x} y1={prev.y} x2={p.x} y2={p.y} stroke="#1e6fff" strokeWidth={2} />
            })
          )}
        </g>
      ))}
    </svg>
  )
}

function bezierPath(e: C4LayoutResult['relationships'][number]) {
  const [p0, p3] = e.points
  const [c1, c2] = e.controlPoints || [p0, p3]
  return `M ${p0.x} ${p0.y} C ${c1.x} ${c1.y}, ${c2.x} ${c2.y}, ${p3.x} ${p3.y}`
}
