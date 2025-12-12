import type { C4Id } from '../brand'
import type { Size } from '../types'
import { isExternalKind } from '../c4-model'
import type { C4LayoutOptions } from '../c4-options'
import type { TextMeasurer } from '../utils/text-measurer'
import type { HierarchyNode, HierarchyTree } from './hierarchy'
import { layoutGrid } from './grid-layout'
import {
  layoutL2Containers,
  layoutL3Components,
  type C4LayoutNode
} from './c4-level-layouts'

export interface SizedNode extends HierarchyNode {
  size: Size
  labelLines: string[]
  contentSize: Size
  childLayout?: { width: number; height: number; positions: Map<C4Id, { x: number; y: number }> }
}

function mapToC4LayoutNode(node: SizedNode): C4LayoutNode {
  return {
    id: node.id,
    size: node.size,
    isExternal: isExternalKind(node.node.kind),
    laneHint: detectLaneHint(node.node.label)
  }
}

function detectLaneHint(label: string): 'controller' | 'service' | 'repository' | 'other' | undefined {
  const lower = label.toLowerCase();
  if (lower.includes('controller') || lower.includes('handler') || lower.includes('api') || lower.includes('endpoint')) {
    return 'controller';
  }
  if (lower.includes('repo') || lower.includes('repository') || lower.includes('dao') || lower.includes('store')) {
    return 'repository';
  }
  if (lower.includes('service') || lower.includes('manager') || lower.includes('processor')) {
    return 'service';
  }
  return undefined;
}


/**
 * Calculate the size of a leaf node (node without children or with collapsed children)
 * This is a pure function that only measures text and icons
 */
function sizeLeafNode(
  node: HierarchyNode,
  measurer: TextMeasurer,
  options: C4LayoutOptions
): { size: Size; contentSize: Size; labelLines: string[] } {
  const kind = node.node.kind
  const level = node.node.level

  // -- Legacy Cytoscape Sizing Logic --
  // We mirror the padding and icon placement rules from packages/viewer/src/style/node-styles.ts

  let paddingLeft = 12
  let paddingRight = 12
  let paddingTop = 12
  let paddingBottom = 12
  let maxLabelWidth = 120 // Default text-max-width
  let iconHeightSpace = 0 // Extra vertical space for top-icons

  if (kind === 'SoftwareSystem') {
    // Icon Left (16px pos + 18px width) -> padding-left: 50
    paddingLeft = 50
    paddingRight = 16
    paddingTop = 16
    paddingBottom = 16
    maxLabelWidth = 110
  } else if (kind === 'Container') {
    // Icon Left (12px pos + 16px width) -> padding-left: 40
    // We assume containers have icons for consistency with "Smart" look
    paddingLeft = 40
    paddingRight = 12
    paddingTop = 12
    paddingBottom = 12
    maxLabelWidth = 120
  } else if (kind === 'Person') {
    // Icon Top (background-position-y: 20%) -> text-margin-y: 14 mechanism
    // In our semantic layout, we just add height to the "header" area
    paddingLeft = 20
    paddingRight = 20
    paddingTop = 20 // box padding
    paddingBottom = 20
    iconHeightSpace = 24 // effectively text-margin-y + icon visual space
    maxLabelWidth = 140
  } else if (kind === 'Database' || kind === 'Queue') {
    // Icon Top
    paddingLeft = 12
    paddingRight = 12
    paddingTop = 12
    iconHeightSpace = 20
    maxLabelWidth = 120
  }

  // 1. Measure text with wrapping
  // Note: maxLabelWidth here controls where we wrap
  const textMetrics = measurer.measureMultiline(node.node.label, kind, level, maxLabelWidth)

  let contentWidth = textMetrics.width
  let contentHeight = textMetrics.height + iconHeightSpace

  // 2. Add description if present
  if (node.node.description) {
    const descMetrics = measurer.measureMultiline(node.node.description, kind, level, maxLabelWidth)
    contentWidth = Math.max(contentWidth, descMetrics.width)
    contentHeight += descMetrics.height + 8
  }

  // 3. Add technology if present
  if (node.node.technology) {
    const techMetrics = measurer.measure(`[${node.node.technology}]`, kind, level, maxLabelWidth)
    contentWidth = Math.max(contentWidth, techMetrics.width)
    contentHeight += techMetrics.height + 4
  }

  // 4. Calculate final size with padding
  const width = contentWidth + paddingLeft + paddingRight
  const height = contentHeight + paddingTop + paddingBottom

  // 6. Apply constraints
  const constrainedWidth = Math.max(options.minSize.width, Math.min(options.maxSize.width, width))
  const constrainedHeight = Math.max(options.minSize.height, Math.min(options.maxSize.height, height))

  // 7. Apply hints if present
  const finalWidth = node.node.widthHint ?? constrainedWidth
  const finalHeight = node.node.heightHint ?? constrainedHeight

  return {
    size: { width: finalWidth, height: finalHeight },
    contentSize: { width: contentWidth, height: contentHeight },
    labelLines: textMetrics.lines
  }
}

/**
 * Layout children of a parent node and return their relative positions
 * This function chooses the appropriate layout algorithm based on the parent type
 */
function layoutChildren(
  parentNode: HierarchyNode,
  childSizes: SizedNode[],
  relationships: { from: C4Id; to: C4Id }[],
  options: C4LayoutOptions
): { positions: Map<C4Id, { x: number; y: number }>; boundingBox: { width: number; height: number } } {
  const kind = parentNode.node.kind

  // Map child sizes to C4LayoutNode format
  const c4Children = childSizes.map(mapToC4LayoutNode)

  // Select layout algorithm based on parent type
  if (options.strategy === 'incremental' && options.stability?.previousPositions) {
    const parentPrev = options.stability.previousPositions.get(parentNode.id)
    // Only use incremental if we have parent position and at least one child position
    const knownChildren = c4Children.filter(c => options.stability!.previousPositions!.has(c.id))

    if (parentPrev && knownChildren.length > 0) {
      // Logic: Preserve relative positions for known children
      const positions = new Map<C4Id, { x: number; y: number }>()
      let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity

      // 1. Place known children
      for (const child of c4Children) {
        const prev = options.stability!.previousPositions!.get(child.id)
        if (prev) {
          // Calculate relative to parent
          // Note: coordinates.ts will normalize this by subtracting minX/minY
          // So we just need consistent relative offsets
          const relX = prev.x - parentPrev.x
          const relY = prev.y - parentPrev.y
          positions.set(child.id, { x: relX, y: relY })

          minX = Math.min(minX, relX)
          minY = Math.min(minY, relY)
          maxX = Math.max(maxX, relX + child.size.width)
          maxY = Math.max(maxY, relY + child.size.height)
        }
      }

      // 2. Place unknown children (newly added/expanded)
      // Heuristic: Place them at the bottom, or grid them?
      // Since filtering happens per-parent, if *some* correspond to prev, use strict.
      // If we are expanding a node, usually ALL its children are new (so this block skipped).
      // If we are in a parent where a sibling expanded, the sibling has a position!
      // So this block handles STABILITY of siblings.
      // For the Expanded Node itself (a child of this parent):
      // It exists in prevPositions (it was a leaf).
      // So it gets placed at its old top-left.
      // BUT its size has increased.
      // We calculate bounding box based on NEW size.

      for (const child of c4Children) {
        if (!positions.has(child.id)) {
          // New child in existing container? 
          // Place at bottom left of current bounds
          const x = minX !== Infinity ? minX : 0
          const y = maxY !== -Infinity ? maxY + 20 : 0
          positions.set(child.id, { x, y })

          maxX = Math.max(maxX, x + child.size.width)
          maxY = Math.max(maxY, y + child.size.height)
        } else {
          // Update max bounds using NEW size (important for expanded nodes)
          // position is fixed (top-left), but width/height grew
          const pos = positions.get(child.id)!
          maxX = Math.max(maxX, pos.x + child.size.width)
          maxY = Math.max(maxY, pos.y + child.size.height)
        }
      }

      return {
        positions,
        boundingBox: { width: maxX - minX, height: maxY - minY }
      }
    }
  }

  if (kind === 'SoftwareSystem') {
    // L1→L2: System contains Containers
    const relevantEdges = relationships
      .filter(r => c4Children.some(c => c.id === r.from) && c4Children.some(c => c.id === r.to))
      .map(r => ({ source: r.from as string, target: r.to as string }))

    const layoutRes = layoutL2Containers(c4Children, [], relevantEdges, { x: 0, y: 0 }, options)
    return {
      positions: layoutRes.positions,
      boundingBox: layoutRes.boundarySize || { width: 0, height: 0 }
    }
  } else if (kind === 'Container') {
    // L2→L3: Container contains Components
    const relevantEdges = relationships
      .filter(r => c4Children.some(c => c.id === r.from) && c4Children.some(c => c.id === r.to))
      .map(r => ({ source: r.from as string, target: r.to as string }))

    const layoutRes = layoutL3Components(c4Children, [], relevantEdges, { x: 0, y: 0 }, options)
    return {
      positions: layoutRes.positions,
      boundingBox: layoutRes.boundarySize || { width: 0, height: 0 }
    }
  } else {
    // Default: Grid layout
    const layoutParams = childSizes.map(s => ({ id: s.id, size: s.size }))
    const gridResult = layoutGrid(layoutParams, {
      maxColumns: 3,
      nodeSpacing: options.spacing.node.Container ?? 40,
      rowSpacing: options.spacing.rank.Container ?? 40
    })
    return {
      positions: gridResult.positions,
      boundingBox: { width: gridResult.width, height: gridResult.height }
    }
  }
}

export function calculateSizes(tree: HierarchyTree, relationships: { from: C4Id; to: C4Id }[], measurer: TextMeasurer, options: C4LayoutOptions): Map<C4Id, SizedNode> {
  const sizes = new Map<C4Id, SizedNode>()

  /**
   * Process node in post-order (children before parents)
   * This ensures all children are sized before we calculate parent size
   */
  function processNode(node: HierarchyNode): SizedNode {
    console.log(`[SIZING] Processing ${node.id}:`, {
      childrenCount: node.children.length,
      collapseChildren: node.node.collapseChildren,
      kind: node.node.kind
    })

    // Is this a leaf node or collapsed parent?
    if (node.children.length === 0 || node.node.collapseChildren) {
      // --- LEAF NODE PATH ---
      const leafData = sizeLeafNode(node, measurer, options)
      const sizedNode: SizedNode = {
        ...node,
        size: leafData.size,
        contentSize: leafData.contentSize,
        labelLines: leafData.labelLines,
        childLayout: undefined
      }
      sizes.set(node.id, sizedNode)
      return sizedNode
    } else {
      // --- PARENT NODE PATH ---
      console.log(`[SIZING] ${node.id} taking PARENT path with ${node.children.length} children`)
      // Step 1: Process all children first (post-order)
      const childSizedNodes = node.children.map(c => processNode(c))

      // Step 2: Calculate this node's content size (header)
      const kind = node.node.kind
      const padding = options.spacing.padding[kind] ?? 16
      const headerData = sizeLeafNode(node, measurer, options)

      // Step 3: Layout children to get their relative positions and bounding box
      const childLayout = layoutChildren(node, childSizedNodes, relationships, options)

      // Step 4: Calculate total parent size
      const headerHeight = headerData.contentSize.height + padding
      const safetyPadding = 60

      const totalWidth = childLayout.boundingBox.width + (padding * 2) + safetyPadding
      const totalHeight = headerHeight + childLayout.boundingBox.height + padding + safetyPadding

      // Step 5: Apply constraints
      let width = Math.max(options.minSize.width, Math.min(options.maxSize.width, totalWidth))
      let height = Math.max(options.minSize.height, Math.min(options.maxSize.height, totalHeight))

      // Step 6: Apply hints if present
      if (node.node.widthHint) width = node.node.widthHint
      if (node.node.heightHint) height = node.node.heightHint

      const sizedNode: SizedNode = {
        ...node,
        size: { width, height },
        contentSize: headerData.contentSize,
        labelLines: headerData.labelLines,
        childLayout: {
          width: childLayout.boundingBox.width,
          height: childLayout.boundingBox.height,
          positions: childLayout.positions
        }
      }
      sizes.set(node.id, sizedNode)
      return sizedNode
    }
  }

  // Process all roots (post-order traversal happens inside processNode)
  for (const root of tree.roots) {
    processNode(root)
  }

  return sizes
}
