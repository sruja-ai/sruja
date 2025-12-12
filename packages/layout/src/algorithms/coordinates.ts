import type { C4Id } from '../brand'
import type { Rect as BBox } from '../geometry/rect'
import type { SizedNode } from './sizing'
import type { C4LayoutOptions } from '../c4-options'
import type { HierarchyTree, HierarchyNode } from './hierarchy'
import { layoutSugiyama } from './sugiyama'
import { layoutL0 } from './l0-layout'
import { layoutL1SystemContext } from './l1-layout'


export interface PositionedNode extends SizedNode {
  x: number
  y: number
  bbox: BBox
}

/**
 * Position roots using the appropriate top-level layout algorithm
 */
function positionRoots(
  roots: HierarchyNode[],
  sizes: Map<C4Id, SizedNode>,
  relationships: { from: C4Id; to: C4Id }[],
  options: C4LayoutOptions,
  tree: HierarchyTree
): Map<C4Id, { x: number; y: number }> {
  const rootPositions = new Map<C4Id, { x: number; y: number }>()

  // Aggregate relationships to root level
  const nodeToRoot = new Map<C4Id, C4Id>()
  function mapToRoot(node: HierarchyNode, rootId: C4Id) {
    nodeToRoot.set(node.id, rootId)
    for (const c of node.children) mapToRoot(c, rootId)
  }
  for (const root of roots) mapToRoot(root, root.id)

  const rootRels = new Set<string>()
  const aggregatedRels: { from: C4Id; to: C4Id }[] = []
  for (const rel of relationships) {
    const rootFrom = nodeToRoot.get(rel.from)
    const rootTo = nodeToRoot.get(rel.to)
    if (rootFrom && rootTo && rootFrom !== rootTo) {
      const key = `${rootFrom}->${rootTo}`
      if (!rootRels.has(key)) {
        rootRels.add(key)
        aggregatedRels.push({ from: rootFrom, to: rootTo })
      }
    }
  }

  // Choose layout strategy
  // Choose layout strategy
  if (options.strategy === 'l1-context') {
    // L1 System Context: Star/Radial Layout
    // The new algorithm handles identifying center/satellites internally
    const l1Result = layoutL1SystemContext(tree, new Map([...sizes].map(([k, v]) => [k, v.size])), relationships, options)

    // Copy positions
    for (const [id, node] of l1Result) {
      if (rootPositions.has(id as C4Id)) continue // Don't overwrite if already set (unlikely)
      rootPositions.set(id as C4Id, { x: node.bbox.x, y: node.bbox.y })
    }
  } else if (options.strategy === 'incremental' && options.stability?.previousPositions) {
    // Incremental: Keep roots at previous positions
    for (const root of roots) {
      const prev = options.stability.previousPositions.get(root.id)
      if (prev) {
        rootPositions.set(root.id, { x: prev.x, y: prev.y })
      } else {
        // New root? Place at 0,0 (overlap removal will fix) or try to find a gap
        // For now, center at 0,0
        rootPositions.set(root.id, { x: 0, y: 0 })
      }
    }
  } else if (options.strategy === 'grid') {
    // L0 Grid layout
    const rootNodes = roots.map(r => ({ id: r.id, size: sizes.get(r.id)!.size }))
    const resultL0 = layoutL0(rootNodes, aggregatedRels, options, (id) => tree.nodeMap.get(id), (id) => sizes.get(id))
    resultL0.nodes.forEach(n => rootPositions.set(n.id, { x: n.x, y: n.y }))
  } else {
    // Default: Sugiyama hierarchical layout
    const rootNodes = roots.map(r => ({ id: r.id, size: sizes.get(r.id)!.size }))
    const layoutResult = layoutSugiyama(rootNodes, aggregatedRels, options)
    layoutResult.nodes.forEach(n => rootPositions.set(n.id, { x: n.x, y: n.y }))
  }

  return rootPositions
}

export function assignCoordinates(tree: HierarchyTree, sizes: Map<C4Id, SizedNode>, relationships: { from: C4Id; to: C4Id }[], options: C4LayoutOptions): Map<C4Id, PositionedNode> {
  const result = new Map<C4Id, PositionedNode>()
  const paddingMap = options.spacing.padding

  // Safety margin to ensure children have adequate breathing room within parent
  // Safety margin to ensure children have adequate breathing room within parent
  const CONTAINMENT_BUFFER = 30

  /**
   * Recursively position a node and all its children (top-down, pre-order)
   * @param node - The hierarchy node to position
   * @param absolutePos - The absolute position for this node
   */
  function positionNode(node: HierarchyNode, absolutePos: { x: number; y: number }): void {
    const sized = sizes.get(node.id)!
    const padding = paddingMap[node.node.kind] ?? 16

    // Create positioned node with absolute coordinates
    const positioned: PositionedNode = {
      ...sized,
      x: absolutePos.x,
      y: absolutePos.y,
      bbox: {
        x: absolutePos.x,
        y: absolutePos.y,
        width: sized.size.width,
        height: sized.size.height
      }
    }
    result.set(node.id, positioned)

    // Position children if not collapsed
    const hasChildLayout = !!sized.childLayout
    const isNotCollapsed = !node.node.collapseChildren
    const hasChildren = node.children.length > 0

    console.log(`[COORDS] Processing ${node.id}:`, {
      hasChildLayout,
      isNotCollapsed,
      hasChildren,
      childLayoutPositions: sized.childLayout?.positions?.size ?? 0,
      nodeChildren: node.children.length
    })

    if (hasChildLayout && isNotCollapsed && hasChildren) {
      const headerHeight = sized.contentSize.height + padding

      // Content area starts after header with additional safety buffer
      const contentStartX = absolutePos.x + padding + CONTAINMENT_BUFFER
      const contentStartY = absolutePos.y + headerHeight + CONTAINMENT_BUFFER

      // First, find the min offset in child positions to normalize them
      let minX = Infinity
      let minY = Infinity
      for (const child of node.children) {
        const relativePos = sized.childLayout!.positions.get(child.id)
        if (relativePos) {
          minX = Math.min(minX, relativePos.x)
          minY = Math.min(minY, relativePos.y)
        }
      }

      // Normalize: shift all positions so minX/minY are 0
      const offsetX = minX < 0 ? -minX : 0
      const offsetY = minY < 0 ? -minY : 0

      // Position each child using normalized relative positions
      for (const child of node.children) {
        const relativePos = sized.childLayout!.positions.get(child.id)
        if (relativePos) {
          const childAbsolutePos = {
            x: contentStartX + relativePos.x + offsetX,
            y: contentStartY + relativePos.y + offsetY
          }
          positionNode(child, childAbsolutePos)
        }
      }

      // After positioning children, verify containment and resize parent if needed
      let maxChildRight = 0
      let maxChildBottom = 0
      for (const child of node.children) {
        const childPos = result.get(child.id)
        if (childPos) {
          maxChildRight = Math.max(maxChildRight, childPos.bbox.x + childPos.bbox.width)
          maxChildBottom = Math.max(maxChildBottom, childPos.bbox.y + childPos.bbox.height)
        }
      }

      // Calculate required parent size with additional safety buffer
      const requiredWidth = (maxChildRight - absolutePos.x) + padding + CONTAINMENT_BUFFER
      const requiredHeight = (maxChildBottom - absolutePos.y) + padding + CONTAINMENT_BUFFER

      // Resize parent if children exceed current bounds
      if (requiredWidth > positioned.bbox.width || requiredHeight > positioned.bbox.height) {
        positioned.bbox.width = Math.max(positioned.bbox.width, requiredWidth)
        positioned.bbox.height = Math.max(positioned.bbox.height, requiredHeight)
        positioned.size = {
          width: positioned.bbox.width,
          height: positioned.bbox.height
        }
        // Update in result map
        result.set(node.id, positioned)
      }
    }
  }

  // Step 1: Position all roots
  const rootPositions = positionRoots(tree.roots, sizes, relationships, options, tree)

  // Step 2: Recursively position each root and its descendants
  for (const root of tree.roots) {
    const pos = rootPositions.get(root.id)
    if (pos) {
      positionNode(root, pos)
    }
  }

  return result
}
