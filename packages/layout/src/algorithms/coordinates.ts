import { C4Id } from '../brand'
import { Rect as BBox } from '../geometry/rect'
import { SizedNode } from './sizing'
import { C4LayoutOptions } from '../c4-options'
import { HierarchyTree } from './hierarchy'
import { isContainerKind } from '../c4-model'

export interface PositionedNode extends SizedNode {
  x: number
  y: number
  bbox: BBox
}

export function assignCoordinates(tree: HierarchyTree, sizes: Map<C4Id, SizedNode>, options: C4LayoutOptions): Map<C4Id, PositionedNode> {
  const result = new Map<C4Id, PositionedNode>()
  const paddingMap = options.spacing.padding

  function placeNode(nodeId: C4Id, x: number, y: number): void {
    const node = sizes.get(nodeId)!
    const bbox = { x, y, width: node.size.width, height: node.size.height }
    const positioned: PositionedNode = { ...node, x, y, bbox }
    result.set(nodeId, positioned)

    if (isContainerKind(node.node.kind) && node.childLayout && node.children.length > 0) {
      const padding = paddingMap[node.node.kind] ?? 16
      const headerHeight = node.contentSize.height + padding

      const startX = x + padding
      const startY = y + headerHeight

      for (const child of node.children) {
        const pos = node.childLayout.positions.get(child.id)
        if (pos) {
          placeNode(child.id, startX + pos.x, startY + pos.y)
        }
      }
    }
  }

  // Top-level layout (Basic Grid for Roots)
  // Treat roots as children of a virtual container
  const rootSizes = tree.roots.map(r => sizes.get(r.id)!.size)
  const rootLayout = calculateRootGrid(rootSizes, options)

  let currentY = 0 // Start at 0,0
  let currentRow = 0
  const gap = options.spacing.node.SoftwareSystem ?? 40

  for (let i = 0; i < tree.roots.length; i++) {
    const root = tree.roots[i]
    const col = i % rootLayout.cols
    const row = Math.floor(i / rootLayout.cols)

    if (row > currentRow) {
      currentY += rootLayout.rowHeights[currentRow] + gap
      currentRow = row
    }

    let currentX = 0
    for (let c = 0; c < col; c++) {
      currentX += rootLayout.colWidths[c] + gap
    }

    const cellW = rootLayout.colWidths[col]
    const cellH = rootLayout.rowHeights[row]
    const rootSize = sizes.get(root.id)!.size
    const offX = (cellW - rootSize.width) / 2
    const offY = (cellH - rootSize.height) / 2

    placeNode(root.id, currentX + offX, currentY + offY)
  }

  return result
}

// Duplicate grid logic for roots (could be shared but kept simple here)
function calculateRootGrid(childSizes: any[], options: C4LayoutOptions) {
  const n = childSizes.length
  if (n === 0) return { width: 0, height: 0, cols: 0, rows: 0, colWidths: [], rowHeights: [] }
  const targetRatio = 1.618
  const cols = Math.ceil(Math.sqrt(n * targetRatio))
  const rows = Math.ceil(n / cols)
  const colWidths = new Array(cols).fill(0)
  const rowHeights = new Array(rows).fill(0)
  const gap = options.spacing.node.SoftwareSystem ?? 40
  for (let i = 0; i < n; i++) {
    const col = i % cols
    const row = Math.floor(i / cols)
    colWidths[col] = Math.max(colWidths[col], childSizes[i].width)
    rowHeights[row] = Math.max(rowHeights[row], childSizes[i].height)
  }
  return { cols, rows, colWidths, rowHeights }
}
