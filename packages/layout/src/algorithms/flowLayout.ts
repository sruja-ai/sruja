import type { LayoutNode, LayoutConfig, Size } from '../types'

export function flowLayout(children: { id: string; size: Size }[], config: LayoutConfig): { nodes: LayoutNode; children: LayoutNode[] } {
  let y = config.padding as any
  const laid: LayoutNode[] = []
  let maxW = 0 as any
  for (const c of children) {
    const node: LayoutNode = { id: c.id as any, bounds: { x: config.padding as any, y, width: c.size.width, height: c.size.height }, children: [] }
    laid.push(node)
    y = (y + c.size.height + config.spacingY) as any
    if (c.size.width > maxW) maxW = c.size.width
  }
  const container: LayoutNode = {
    id: 'container' as any,
    bounds: { x: 0 as any, y: 0 as any, width: (maxW + config.padding * 2) as any, height: (y - config.spacingY + config.padding) as any },
    children: laid
  }
  return { nodes: container, children: laid }
}
