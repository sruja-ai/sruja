import type { C4Id } from "../brand";
import type { PositionedNode } from "./coordinates";

/**
 * Remove overlaps between positioned nodes.
 *
 * Two modes:
 * 1. Same-parent overlaps: Nodes with same parent are pushed apart (original behavior)
 * 2. Top-level overlaps: Top-level nodes (parent === undefined) including expanded
 *    parent boundaries are pushed apart to prevent overlaps
 */
export function removeOverlaps(
  positioned: Map<C4Id, PositionedNode>,
  padding = 8,
  iterations = 10
): Map<C4Id, PositionedNode> {
  const nodes = [...positioned.values()];

  // Optimized: cache nodes length to avoid repeated property access
  const nodesLength = nodes.length;

  for (let iter = 0; iter < iterations; iter++) {
    let moved = false;

    for (let i = 0; i < nodesLength; i++) {
      for (let j = i + 1; j < nodesLength; j++) {
        const a = nodes[i];
        const b = nodes[j];

        // Check if we should resolve overlap between these nodes
        const shouldResolve = shouldResolveOverlap(a, b);
        if (!shouldResolve) continue;

        // Optimized: cache bbox references to avoid repeated property access
        const aBbox = a.bbox;
        const bBbox = b.bbox;

        if (rectsOverlap(aBbox, bBbox, padding)) {
          // Optimized: cache center calculations
          const aCenterX = aBbox.x + aBbox.width * 0.5;
          const aCenterY = aBbox.y + aBbox.height * 0.5;
          const bCenterX = bBbox.x + bBbox.width * 0.5;
          const bCenterY = bBbox.y + bBbox.height * 0.5;

          const dx = aCenterX - bCenterX;
          const dy = aCenterY - bCenterY;

          // Push nodes apart - use larger push for better convergence
          // Increase push factor for better spacing, especially for expanded nodes
          const pushFactor = Math.max(8, padding * 0.5);
          const absDx = Math.abs(dx);
          const absDy = Math.abs(dy);
          const pushX = Math.sign(dx) * Math.max(pushFactor, absDx * 0.125); // Use multiplication instead of division
          const pushY = Math.sign(dy) * Math.max(pushFactor, absDy * 0.125);

          const newAX = a.x + pushX;
          const newAY = a.y + pushY;
          const newBX = b.x - pushX;
          const newBY = b.y - pushY;

          a.x = newAX;
          a.y = newAY;
          b.x = newBX;
          b.y = newBY;
          aBbox.x = newAX;
          aBbox.y = newAY;
          bBbox.x = newBX;
          bBbox.y = newBY;
          moved = true;
        }
      }
    }
    if (!moved) break;
  }

  const out = new Map<C4Id, PositionedNode>();
  for (const n of nodes) out.set(n.id, n);
  return out;
}

/**
 * Determine if overlap between two nodes should be resolved.
 * - Same parent: always resolve (siblings)
 * - Both top-level (no parent): resolve (external nodes, expanded systems)
 * - Parent-child pairs: NEVER resolve (child must stay inside parent)
 * - Cross-hierarchy (different parents): only resolve if truly independent
 */
function shouldResolveOverlap(a: PositionedNode, b: PositionedNode): boolean {
  // CRITICAL: Never resolve overlap between a parent and its own child
  // This would push the child outside the parent, breaking containment
  if (a.parent?.id === b.id || b.parent?.id === a.id) {
    return false;
  }

  // Same parent - always resolve (siblings)
  if (a.parent === b.parent) return true;

  // Both are top-level nodes - resolve (external nodes, expanded systems)
  if (!a.parent && !b.parent) return true;

  // One is top-level, one has a parent
  // Only resolve if the top-level node is NOT an ancestor of the child
  // This handles external systems overlapping with expanded system boundaries
  if (!a.parent && b.parent) {
    // a is top-level, b has a parent
    // Check if a is b's grandparent or ancestor
    let current: typeof b.parent | undefined = b.parent;
    while (current) {
      if (current.id === a.id) return false; // a is b's ancestor, don't push apart
      current = current.parent;
    }
    return true; // Different hierarchies, resolve overlap
  }
  if (!b.parent && a.parent) {
    // b is top-level, a has a parent
    let current: typeof a.parent | undefined = a.parent;
    while (current) {
      if (current.id === b.id) return false; // b is a's ancestor, don't push apart
      current = current.parent;
    }
    return true; // Different hierarchies, resolve overlap
  }

  // Both have parents but different parents - resolve if parents are different
  return false;
}

function rectsOverlap(
  a: { x: number; y: number; width: number; height: number },
  b: { x: number; y: number; width: number; height: number },
  pad: number
): boolean {
  // Optimized: cache boundaries to avoid repeated calculations
  const aRight = a.x + a.width;
  const aBottom = a.y + a.height;
  const bRight = b.x + b.width;
  const bBottom = b.y + b.height;

  return !(aRight + pad < b.x || bRight + pad < a.x || aBottom + pad < b.y || bBottom + pad < a.y);
}
