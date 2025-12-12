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

  return (
    <>
      <BaseEdge id={id} path={path} markerEnd={markerEnd} style={{ stroke, strokeWidth }} />
      {label && labelPos && (
        <EdgeLabelRenderer>
          <div
            style={{
              position: 'absolute',
              transform: `translate(${labelPos.x}px, ${labelPos.y}px)`,
              background: 'rgba(255,255,255,0.8)',
              border: '1px solid #d1d5db',
              borderRadius: 6,
              padding: '2px 6px',
              color: '#374151',
              fontSize: 12,
              pointerEvents: 'none',
              whiteSpace: 'nowrap',
            }}
          >
            {label}
          </div>
        </EdgeLabelRenderer>
      )}
    </>
  )
}
