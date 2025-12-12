import type { C4Id } from '../brand'
import type { PositionedNode } from './coordinates'

/**
 * Remove overlaps between positioned nodes.
 * 
 * Two modes:
 * 1. Same-parent overlaps: Nodes with same parent are pushed apart (original behavior)
 * 2. Top-level overlaps: Top-level nodes (parent === undefined) including expanded 
 *    parent boundaries are pushed apart to prevent overlaps
 */
export function removeOverlaps(positioned: Map<C4Id, PositionedNode>, padding = 8, iterations = 10): Map<C4Id, PositionedNode> {
  const nodes = [...positioned.values()]

  for (let iter = 0; iter < iterations; iter++) {
    let moved = false

    for (let i = 0; i < nodes.length; i++) {
      for (let j = i + 1; j < nodes.length; j++) {
        const a = nodes[i]
        const b = nodes[j]

        // Check if we should resolve overlap between these nodes
        const shouldResolve = shouldResolveOverlap(a, b)
        if (!shouldResolve) continue

        if (rectsOverlap(a.bbox, b.bbox, padding)) {
          const dx = (a.bbox.x + a.bbox.width / 2) - (b.bbox.x + b.bbox.width / 2)
          const dy = (a.bbox.y + a.bbox.height / 2) - (b.bbox.y + b.bbox.height / 2)

          // Push nodes apart - use larger push for better convergence
          const pushFactor = 5
          const pushX = Math.sign(dx) * Math.max(pushFactor, Math.abs(dx) / 10)
          const pushY = Math.sign(dy) * Math.max(pushFactor, Math.abs(dy) / 10)

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

/**
 * Determine if overlap between two nodes should be resolved.
 * - Same parent: always resolve (siblings)
 * - Both top-level (no parent): resolve (external nodes, expanded systems)
 * - One is top-level, one has a parent: resolve if parent's bbox overlaps 
 *   (cross-hierarchy overlap like external overlapping expanded system)
 */
function shouldResolveOverlap(a: PositionedNode, b: PositionedNode): boolean {
  // Same parent - always resolve
  if (a.parent === b.parent) return true

  // Both are top-level nodes
  if (!a.parent && !b.parent) return true

  // One is top-level, one is a child - resolve to prevent external nodes 
  // from overlapping with expanded system contents
  if (!a.parent || !b.parent) return true

  return false
}

function rectsOverlap(a: { x: number; y: number; width: number; height: number }, b: { x: number; y: number; width: number; height: number }, pad: number): boolean {
  return !(a.x + a.width + pad < b.x || b.x + b.width + pad < a.x || a.y + a.height + pad < b.y || b.y + b.height + pad < a.y)
}

