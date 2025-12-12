/**
 * Grid-based child layout for containers
 * 
 * Instead of laying children horizontally (which makes containers too wide),
 * this arranges children in a grid with configurable max columns.
 */

import type { C4Id } from '../brand'
import type { Size } from '../types'

export interface GridLayoutResult {
    width: number
    height: number
    positions: Map<C4Id, { x: number; y: number }>
}

export interface GridLayoutOptions {
    maxColumns?: number
    nodeSpacing?: number
    rowSpacing?: number
}

/**
 * Lay out children in a grid pattern
 */
export function layoutGrid(
    nodes: { id: C4Id; size: Size }[],
    options: GridLayoutOptions = {}
): GridLayoutResult {
    const {
        maxColumns = 3,
        nodeSpacing = 40,
        rowSpacing = 40
    } = options

    if (nodes.length === 0) {
        return { width: 0, height: 0, positions: new Map() }
    }

    const positions = new Map<C4Id, { x: number; y: number }>()

    // Calculate optimal column count
    const cols = Math.min(maxColumns, nodes.length)
    const rows = Math.ceil(nodes.length / cols)

    // Find max dimensions per column and row
    const colWidths: number[] = Array(cols).fill(0)
    const rowHeights: number[] = Array(rows).fill(0)

    nodes.forEach((node, idx) => {
        const col = idx % cols
        const row = Math.floor(idx / cols)
        colWidths[col] = Math.max(colWidths[col], node.size.width)
        rowHeights[row] = Math.max(rowHeights[row], node.size.height)
    })

    // Calculate column X positions
    const colX: number[] = []
    let x = 0
    for (let c = 0; c < cols; c++) {
        colX.push(x)
        x += colWidths[c] + nodeSpacing
    }

    // Calculate row Y positions
    const rowY: number[] = []
    let y = 0
    for (let r = 0; r < rows; r++) {
        rowY.push(y)
        y += rowHeights[r] + rowSpacing
    }

    // Position each node
    nodes.forEach((node, idx) => {
        const col = idx % cols
        const row = Math.floor(idx / cols)

        // Center node within its cell
        const cellWidth = colWidths[col]
        const cellHeight = rowHeights[row]
        const offsetX = (cellWidth - node.size.width) / 2
        const offsetY = (cellHeight - node.size.height) / 2

        positions.set(node.id, {
            x: colX[col] + offsetX,
            y: rowY[row] + offsetY
        })
    })

    // Calculate total dimensions
    const totalWidth = colWidths.reduce((sum, w) => sum + w, 0) + nodeSpacing * (cols - 1)
    const totalHeight = rowHeights.reduce((sum, h) => sum + h, 0) + rowSpacing * (rows - 1)

    return {
        width: Math.max(0, totalWidth),
        height: Math.max(0, totalHeight),
        positions
    }
}
