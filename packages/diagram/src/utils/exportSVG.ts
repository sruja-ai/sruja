import type { Node, Edge } from '@xyflow/react'
import type { C4NodeData } from '../types'

function esc(text: string) {
  return String(text)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;')
}

export function exportSVG(nodes: Node<C4NodeData>[], edges: Edge[]): string {
  const bbox = computeBBox(nodes)
  const width = Math.max(1, bbox.w)
  const height = Math.max(1, bbox.h)
  const originX = bbox.x1
  const originY = bbox.y1

  let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${bbox.w} ${bbox.h}">`
  svg += '<defs>'
  svg += '<style type="text/css"><![CDATA['
  svg += `.node-label { font-family: system-ui, sans-serif; font-size: 12px; fill: #1f2937; }`
  svg += `.edge-label { font-family: system-ui, sans-serif; font-size: 10px; fill: #6b7280; }`
  svg += ']]></style>'
  svg += `<marker id="arrowhead" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto"><polygon points="0 0, 10 3, 0 6" fill="#6b7280"/></marker>`
  svg += '</defs>'
  svg += `<rect x="0" y="0" width="${bbox.w}" height="${bbox.h}" fill="#0b0f14"/>`

  for (const edge of edges) {
    const points = (edge.data as any)?.points as Array<{ x: number; y: number }> | undefined
    const path = points && points.length > 1
      ? `M ${points[0].x - originX} ${points[0].y - originY} ` + points.slice(1).map(p => `L ${p.x - originX} ${p.y - originY}`).join(' ')
      : `M ${getNodeCenter(nodes, edge.source).x - originX} ${getNodeCenter(nodes, edge.source).y - originY} L ${getNodeCenter(nodes, edge.target).x - originX} ${getNodeCenter(nodes, edge.target).y - originY}`
    svg += `<path d="${path}" stroke="#6b7280" stroke-width="2" fill="none" marker-end="url(#arrowhead)"/>`
    const label = edge.label
    const lp = (edge.data as any)?.labelPosition as { x: number; y: number } | undefined
    if (label && lp) {
      const x = lp.x - originX
      const y = lp.y - originY
      const w = Math.max(24, String(label).length * 6)
      const h = 16
      const rx = 6
      svg += `<rect x="${x - w / 2}" y="${y - h / 2}" width="${w}" height="${h}" rx="${rx}" fill="rgba(255,255,255,0.85)" stroke="#d1d5db"/>`
      svg += `<text x="${x}" y="${y}" text-anchor="middle" dominant-baseline="middle" class="edge-label">${esc(String(label))}</text>`
    }
  }

  for (const node of nodes) {
    const x = node.position.x + (node.parentId ? getParent(nodes, node.parentId)?.position.x || 0 : 0)
    const y = node.position.y + (node.parentId ? getParent(nodes, node.parentId)?.position.y || 0 : 0)
    const cx = x - originX
    const cy = y - originY
    const w = (node as any).width || 180
    const h = (node as any).height || 100
    const type = (node.data as any)?.type || 'node'
    const palette: Record<string, { bg: string; border: string; text: string }> = {
      person: { bg: '#2563eb', border: '#1d4ed8', text: '#ffffff' },
      system: { bg: '#111827', border: '#374151', text: '#ffffff' },
      container: { bg: '#0b0f14', border: '#374151', text: '#e5e7eb' },
      datastore: { bg: '#0b0f14', border: '#4b5563', text: '#e5e7eb' },
      queue: { bg: '#0b0f14', border: '#374151', text: '#e5e7eb' },
    }
    const color = palette[type] || { bg: '#0b0f14', border: '#6b7280', text: '#e5e7eb' }
    const rx = type === 'person' ? w / 2 : 8
    const isBoundary = type === 'system-boundary' || type === 'container-boundary' || type === 'enterprise-boundary'
    const dash = isBoundary ? ' stroke-dasharray="6,4"' : ''
    const fill = isBoundary ? 'none' : color.bg
    svg += `<rect x="${cx - w / 2}" y="${cy - h / 2}" width="${w}" height="${h}" rx="${rx}" fill="${fill}" stroke="${color.border}" stroke-width="2"${dash}/>`
    const label = (node.data as any)?.label || node.id
    if (isBoundary) {
      const title = ((node.data as any)?.titleBar?.label) || label
      const count = (node.data as any)?.childCount
      const countLabel = ((node.data as any)?.titleBar?.countLabel) || (type === 'enterprise-boundary' ? 'systems' : type === 'system-boundary' ? 'containers' : 'components')
      const text = count != null ? `${title}  •  ${count} ${countLabel}` : `${title}`
      const bw = Math.max(60, text.length * 7)
      const bh = 18
      const bx = cx - w / 2 + 8
      const by = cy - h / 2 + 8
      svg += `<rect x="${bx}" y="${by}" width="${bw}" height="${bh}" rx="6" fill="rgba(17,24,39,0.85)" stroke="#374151"/>`
      svg += `<text x="${bx + bw / 2}" y="${by + bh / 2}" text-anchor="middle" dominant-baseline="middle" class="node-label" fill="#e5e7eb">${esc(text)}</text>`
    } else {
      svg += `<text x="${cx}" y="${cy}" text-anchor="middle" dominant-baseline="middle" class="node-label" fill="${color.text}">${esc(label)}</text>`
    }
  }

  svg += '</svg>'
  return svg
}

function computeBBox(nodes: Node<C4NodeData>[]) {
  let x1 = Infinity, y1 = Infinity, x2 = -Infinity, y2 = -Infinity
  for (const n of nodes) {
    const x = n.position.x + (n.parentId ? getParent(nodes, n.parentId)?.position.x || 0 : 0)
    const y = n.position.y + (n.parentId ? getParent(nodes, n.parentId)?.position.y || 0 : 0)
    const w = (n as any).width || 180
    const h = (n as any).height || 100
    x1 = Math.min(x1, x - w / 2)
    y1 = Math.min(y1, y - h / 2)
    x2 = Math.max(x2, x + w / 2)
    y2 = Math.max(y2, y + h / 2)
  }
  const w = x2 - x1
  const h = y2 - y1
  return { x1, y1, x2, y2, w, h }
}

function getParent(nodes: Node<C4NodeData>[], id?: string) {
  if (!id) return undefined
  return nodes.find(n => n.id === id)
}

function getNodeCenter(nodes: Node<C4NodeData>[], id: string) {
  const n = nodes.find(nn => nn.id === id)
  if (!n) return { x: 0, y: 0 }
  const x = n.position.x + (n.parentId ? getParent(nodes, n.parentId)?.position.x || 0 : 0)
  const y = n.position.y + (n.parentId ? getParent(nodes, n.parentId)?.position.y || 0 : 0)
  return { x, y }
}
