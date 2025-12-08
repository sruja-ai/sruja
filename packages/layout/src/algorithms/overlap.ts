import { C4Id } from '../brand'
import type { PositionedNode } from './coordinates'

export function removeOverlaps(positioned: Map<C4Id, PositionedNode>, padding = 8, iterations = 10): Map<C4Id, PositionedNode> {
  const nodes = [...positioned.values()]
  for (let iter = 0; iter < iterations; iter++) {
    let moved = false
    for (let i = 0; i < nodes.length; i++) {
      for (let j = i + 1; j < nodes.length; j++) {
        const a = nodes[i]
        const b = nodes[j]
        if (a.parent !== b.parent) continue
        if (rectsOverlap(a.bbox, b.bbox, padding)) {
          const dx = (a.bbox.x + a.bbox.width / 2) - (b.bbox.x + b.bbox.width / 2)
          const dy = (a.bbox.y + a.bbox.height / 2) - (b.bbox.y + b.bbox.height / 2)
          const pushX = Math.sign(dx) * Math.max(1, Math.abs(dx) / 10)
          const pushY = Math.sign(dy) * Math.max(1, Math.abs(dy) / 10)
          a.x += pushX
          a.y += pushY
          b.x -= pushX
          b.y -= pushY
          a.bbox.x = a.x
          a.bbox.y = a.y
          b.bbox.x = b.x
          b.bbox.y = b.y
          moved = true
        }
      }
    }
    if (!moved) break
  }
  const out = new Map<C4Id, PositionedNode>()
  for (const n of nodes) out.set(n.id, n)
  return out
}

function rectsOverlap(a: { x: number; y: number; width: number; height: number }, b: { x: number; y: number; width: number; height: number }, pad: number): boolean {
  return !(a.x + a.width + pad < b.x || b.x + b.width + pad < a.x || a.y + a.height + pad < b.y || b.y + b.height + pad < a.y)
}
