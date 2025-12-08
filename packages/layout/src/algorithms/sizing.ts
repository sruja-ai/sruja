import { C4Id } from '../brand'
import { Size } from '../types'
import { isContainerKind, C4Kind } from '../c4-model'
import { C4LayoutOptions } from '../c4-options'
import { TextMeasurer } from '../utils/text-measurer'
import { HierarchyNode, HierarchyTree } from './hierarchy'
import { layoutSugiyama } from './sugiyama'

export interface SizedNode extends HierarchyNode {
  size: Size
  labelLines: string[]
  contentSize: Size
  childLayout?: { width: number; height: number; positions: Map<C4Id, { x: number; y: number }> }
}

export function calculateSizes(tree: HierarchyTree, relationships: { from: C4Id; to: C4Id }[], measurer: TextMeasurer, options: C4LayoutOptions): Map<C4Id, SizedNode> {
  const sizes = new Map<C4Id, SizedNode>()

  function processNode(node: HierarchyNode): SizedNode {
    const kind = node.node.kind
    const level = node.node.level
    const padding = options.spacing.padding[kind] ?? 16

    // 1. Measure text
    const maxLabelWidth = options.maxSize.width - padding * 2
    const textMetrics = measurer.measureMultiline(node.node.label, kind, level, maxLabelWidth)

    let contentWidth = textMetrics.width
    let contentHeight = textMetrics.height

    if (node.node.description) {
      const descMetrics = measurer.measureMultiline(node.node.description, kind, level, maxLabelWidth)
      contentWidth = Math.max(contentWidth, descMetrics.width)
      contentHeight += descMetrics.height + 8
    }
    if (node.node.technology) {
      const techMetrics = measurer.measure(`[${node.node.technology}]`, kind, level, maxLabelWidth)
      contentWidth = Math.max(contentWidth, techMetrics.width)
      contentHeight += techMetrics.height + 4
    }

    let width = 0
    let height = 0
    let childPositions: Map<C4Id, { x: number; y: number }> | undefined

    // 2. Container sizing (Recursive Sugiyama)
    if (isContainerKind(kind) && node.children.length > 0) {
      const childSizes = node.children.map(c => processNode(c))

      // Filter relationships relevant to these children
      const childIds = new Set(node.children.map(c => c.id))
      const relevantRels = relationships.filter(r => childIds.has(r.from) && childIds.has(r.to))

      // Run Sugiyama
      const layoutParams = childSizes.map(s => ({ id: s.id, size: s.size }))
      const layoutResult = layoutSugiyama(layoutParams, relevantRels, options)

      const headerHeight = contentHeight + padding
      width = layoutResult.width + padding * 2
      // Sugiyama usually doesn't include parent padding, so we add it.
      // layoutResult.height is the bounding box of children.
      height = layoutResult.height + headerHeight + padding

      childPositions = new Map()
      layoutResult.nodes.forEach((n, id) => {
        childPositions!.set(id, { x: n.x, y: n.y })
      })

    } else {
      width = contentWidth + padding * 2
      height = contentHeight + padding * 2
    }

    // 3. Constraints
    width = Math.max(options.minSize.width, Math.min(options.maxSize.width, width))
    height = Math.max(options.minSize.height, Math.min(options.maxSize.height, height))

    // 4. Hints
    if (node.node.widthHint) width = node.node.widthHint
    if (node.node.heightHint) height = node.node.heightHint

    const sizedNode: SizedNode = {
      ...node,
      size: { width, height },
      labelLines: textMetrics.lines,
      contentSize: { width: contentWidth, height: contentHeight },
      childLayout: childPositions ? { width: width - padding * 2, height: height - (contentHeight + padding * 2), positions: childPositions } : undefined
    }
    sizes.set(node.id, sizedNode)
    return sizedNode
  }

  for (const root of tree.roots) {
    processNode(root)
  }
  return sizes
}
