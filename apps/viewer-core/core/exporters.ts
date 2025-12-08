import type { Core } from 'cytoscape'
import { Colors, getCssVar } from '@sruja/shared/utils/cssVars'

function escapeXml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;')
}

export function exportPNG(cy: Core | null, options?: any): string {
  if (!cy) return ''
  return cy.png({
    output: 'base64uri',
    full: true,
    bg: Colors.background(),
    scale: options?.scale || 2,
  })
}

export function exportSVG(cy: Core | null, options?: any): string {
  if (!cy) return ''

  const scale = options?.scale || 1
  const bbox = cy.elements().boundingBox()
  const width = bbox.w * scale
  const height = bbox.h * scale
  const originX = bbox.x1
  const originY = bbox.y1

  let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${bbox.w} ${bbox.h}">`
  svg += '<defs>'
  svg += '<style type="text/css"><![CDATA['
  svg += `.node-label { font-family: system-ui, sans-serif; font-size: 12px; fill: ${Colors.textPrimary()}; }`
  svg += `.edge-label { font-family: system-ui, sans-serif; font-size: 10px; fill: ${Colors.neutral500()}; }`
  svg += ']]></style>'
  svg += `<marker id="arrowhead" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto"><polygon points="0 0, 10 3, 0 6" fill="${Colors.neutral500()}"/></marker>`
  svg += '</defs>'

  svg += `<rect x="0" y="0" width="${bbox.w}" height="${bbox.h}" fill="${Colors.background()}"/>`

  const edges = cy.edges()
  edges.forEach(edge => {
    const source = edge.source()
    const target = edge.target()
    const sourcePosRaw = source.position()
    const targetPosRaw = target.position()
    const sourcePos = { x: sourcePosRaw.x - originX, y: sourcePosRaw.y - originY }
    const targetPos = { x: targetPosRaw.x - originX, y: targetPosRaw.y - originY }

    const dx = targetPos.x - sourcePos.x
    const dy = targetPos.y - sourcePos.y
    const dist = Math.sqrt(dx * dx + dy * dy)
    const curvature = Math.min(dist * 0.3, 50)

    const controlX1 = sourcePos.x + dx * 0.5
    const controlY1 = sourcePos.y - curvature
    const controlX2 = targetPos.x - dx * 0.5
    const controlY2 = targetPos.y - curvature

    const path = `M ${sourcePos.x} ${sourcePos.y} C ${controlX1} ${controlY1}, ${controlX2} ${controlY2}, ${targetPos.x} ${targetPos.y}`

    svg += `<path d="${path}" stroke="${Colors.neutral500()}" stroke-width="2" fill="none" marker-end="url(#arrowhead)"/>`

    const label = edge.data('label')
    if (label) {
      const midX = (sourcePos.x + targetPos.x) / 2
      const midY = (sourcePos.y + targetPos.y) / 2 - curvature / 2
      svg += `<text x="${midX}" y="${midY}" text-anchor="middle" class="edge-label">${escapeXml(label)}</text>`
    }
  })

  const nodes = cy.nodes()
  nodes.forEach(node => {
    const posRaw = node.position()
    const pos = { x: posRaw.x - originX, y: posRaw.y - originY }
    const data = node.data()
    const width = node.width()
    const height = node.height()
    const type = data.type || 'node'

    const palette: Record<string, { bg: string; border: string; text: string }> = {
      person: { bg: getCssVar('--color-primary-500'), border: getCssVar('--color-primary-600'), text: Colors.background() },
      system: { bg: getCssVar('--color-neutral-900'), border: getCssVar('--color-neutral-700'), text: Colors.background() },
      container: { bg: Colors.background(), border: getCssVar('--color-neutral-700'), text: Colors.textPrimary() },
      datastore: { bg: Colors.background(), border: getCssVar('--color-neutral-600'), text: Colors.textPrimary() },
      queue: { bg: Colors.background(), border: getCssVar('--color-neutral-700'), text: Colors.textPrimary() },
    }

    const color = palette[type] || { bg: Colors.background(), border: Colors.neutral500(), text: Colors.textPrimary() }
    const rx = type === 'person' ? width / 2 : 8
    svg += `<rect x="${pos.x - width / 2}" y="${pos.y - height / 2}" width="${width}" height="${height}" rx="${rx}" fill="${color.bg}" stroke="${color.border}" stroke-width="2"/>`
    const label = data.label || data.id
    if (label) {
      svg += `<text x="${pos.x}" y="${pos.y}" text-anchor="middle" dominant-baseline="middle" class="node-label" fill="${color.text}">${escapeXml(label)}</text>`
    }
  })

  svg += '</svg>'
  return svg
}
