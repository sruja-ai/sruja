import type { PositionedNode } from './algorithms/coordinates'
import { C4Id } from './brand'

export function beautify(positioned: Map<C4Id, PositionedNode>, opts: { alignNodes?: boolean; gridSize?: number; snapToGrid?: boolean } = {}): Map<C4Id, PositionedNode> {
  const grid = opts.gridSize ?? 20
  const snap = !!opts.snapToGrid
  if (snap) {
    for (const n of positioned.values()) {
      n.x = Math.round(n.x / grid) * grid
      n.y = Math.round(n.y / grid) * grid
      n.bbox.x = n.x
      n.bbox.y = n.y
    }
  }
  return positioned
}
