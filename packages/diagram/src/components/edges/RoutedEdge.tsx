// packages/diagram/src/components/edges/RoutedEdge.tsx
import { BaseEdge, EdgeLabelRenderer } from '@xyflow/react'
import type { EdgeProps } from '@xyflow/react'

export function RoutedEdge({ id, sourceX, sourceY, targetX, targetY, data, markerEnd, style, label }: EdgeProps) {
  const points: Array<{ x: number; y: number }> | undefined = (data as any)?.points
  const path = points && points.length > 1
    ? `M ${points[0].x} ${points[0].y} ` + points.slice(1).map(p => `L ${p.x} ${p.y}`).join(' ')
    : `M ${sourceX} ${sourceY} L ${targetX} ${targetY}`

  const stroke = (style as any)?.stroke ?? '#707070'
  const strokeWidth = (style as any)?.strokeWidth ?? 1.5

  const labelPos = (data as any)?.labelPosition as { x: number; y: number } | undefined

  // Calculate label position with perpendicular offset to avoid overlapping the edge
  let finalLabelPos = labelPos
  if (label && labelPos && points && points.length >= 2) {
    // Find the segment closest to the label position
    let minDist = Infinity
    let closestSegmentIdx = 0
    
    for (let i = 0; i < points.length - 1; i++) {
      const p1 = points[i]
      const p2 = points[i + 1]
      const dx = p2.x - p1.x
      const dy = p2.y - p1.y
      const len = Math.hypot(dx, dy)
      
      if (len === 0) continue
      
      // Project label position onto the segment
      const t = Math.max(0, Math.min(1, 
        ((labelPos.x - p1.x) * dx + (labelPos.y - p1.y) * dy) / (len * len)
      ))
      const projX = p1.x + t * dx
      const projY = p1.y + t * dy
      const dist = Math.hypot(labelPos.x - projX, labelPos.y - projY)
      
      if (dist < minDist) {
        minDist = dist
        closestSegmentIdx = i
      }
    }
    
    // Calculate perpendicular offset
    const p1 = points[closestSegmentIdx]
    const p2 = points[Math.min(closestSegmentIdx + 1, points.length - 1)]
    const dx = p2.x - p1.x
    const dy = p2.y - p1.y
    const len = Math.hypot(dx, dy)
    
    if (len > 0) {
      // Perpendicular vector (normalized)
      const perpX = -dy / len
      const perpY = dx / len
      
      // Offset distance: 15px to move label away from edge line
      const offset = 15
      
      finalLabelPos = {
        x: labelPos.x + perpX * offset,
        y: labelPos.y + perpY * offset,
      }
    }
  }

  return (
    <>
      <BaseEdge id={id} path={path} markerEnd={markerEnd} style={{ stroke, strokeWidth }} />
      {label && finalLabelPos && (
        <EdgeLabelRenderer>
          <div
            style={{
              position: 'absolute',
              transform: `translate(${finalLabelPos.x}px, ${finalLabelPos.y}px) translate(-50%, -50%)`,
              background: 'rgba(255,255,255,0.9)',
              border: '1px solid #d1d5db',
              borderRadius: 6,
              padding: '2px 6px',
              color: '#374151',
              fontSize: 12,
              pointerEvents: 'none',
              whiteSpace: 'nowrap',
              boxShadow: '0 1px 2px rgba(0,0,0,0.1)',
            }}
          >
            {label}
          </div>
        </EdgeLabelRenderer>
      )}
    </>
  )
}
