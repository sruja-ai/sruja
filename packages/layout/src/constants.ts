/**
 * Layout constants for C4 diagrams
 * Based on Layout.md specification
 */

// Node dimensions
export const NODE_WIDTH = 200
export const NODE_HEIGHT = 120

// Spacing between nodes (tighter for better density)
export const H_SPACING = 120
export const V_SPACING = 150

// Boundary configuration
export const BOUNDARY_PADDING = 60

// External nodes gap (for L1/L2)
export const EXTERNAL_GAP = 120

// Lane height for L3 component layouts
export const LANE_HEIGHT = 160

// Edge routing
export const NO_ENTRY_MARGIN = 20 // No-entry zone padding around nodes
export const MIN_EDGE_LENGTH = 40 // Minimum edge segment length

// Grid configuration
export const MAX_COLS_L2 = 3 // Maximum columns for L2 container grid (forces vertical wrapping)
export const MAX_COLS_L3 = 3 // Maximum columns for L3 component lanes

// Optimization thresholds
export const CLUTTER_THRESHOLD = 0.7 // When to trigger re-optimization
export const MAX_OPTIMIZATION_PASSES = 3

/**
 * Get grid dimensions for n items
 */
export function getGridDimensions(n: number, preferredMaxCols = 4): { columns: number; rows: number } {
    let columns: number
    if (n <= 3) columns = n
    else if (n <= 6) columns = 3
    else columns = Math.min(preferredMaxCols, Math.ceil(Math.sqrt(n)))

    const rows = Math.ceil(n / columns)
    return { columns, rows }
}

/**
 * Calculate boundary size based on children
 */
export function calculateBoundarySize(
    childCount: number,
    nodeWidth = NODE_WIDTH,
    nodeHeight = NODE_HEIGHT,
    hSpacing = H_SPACING,
    vSpacing = V_SPACING,
    padding = BOUNDARY_PADDING,
    maxCols = MAX_COLS_L2
): { width: number; height: number } {
    const { columns, rows } = getGridDimensions(childCount, maxCols)

    const width = padding * 2 + columns * nodeWidth + (columns - 1) * hSpacing
    const height = padding * 2 + rows * nodeHeight + (rows - 1) * vSpacing

    return { width, height }
}

// ============================================================================
// ADAPTIVE SPACING
// Density-aware spacing calculations for better diagram structure
// ============================================================================

/** Spacing calculation mode */
export type SpacingMode = 'fixed' | 'adaptive' | 'presentation'

export interface AdaptiveSpacingResult {
    hSpacing: number
    vSpacing: number
    boundaryPadding: number
}

/**
 * Calculate density-aware spacing based on diagram characteristics.
 * 
 * @param nodeCount - Total number of nodes in the diagram
 * @param avgLabelLength - Average character length of node labels (default 15)
 * @param aspectRatio - Target aspect ratio width/height (default 16/9)
 * @param mode - Spacing mode: 'fixed' uses constants, 'adaptive' adjusts for density
 */
export function getAdaptiveSpacing(
    nodeCount: number,
    avgLabelLength: number = 15,
    aspectRatio: number = 16 / 9,
    mode: SpacingMode = 'adaptive'
): AdaptiveSpacingResult {
    // Fixed mode: return base constants
    if (mode === 'fixed') {
        return {
            hSpacing: H_SPACING,
            vSpacing: V_SPACING,
            boundaryPadding: BOUNDARY_PADDING
        }
    }

    // Presentation mode: maximize readability
    if (mode === 'presentation') {
        return {
            hSpacing: Math.round(H_SPACING * 1.5),
            vSpacing: Math.round(V_SPACING * 1.5),
            boundaryPadding: Math.round(BOUNDARY_PADDING * 1.3)
        }
    }

    // Adaptive mode: scale based on density and label length
    const baseH = H_SPACING
    const baseV = V_SPACING

    // Density factor: more nodes → more spacing (up to 1.5x)
    // Starts increasing after 5 nodes, maxes out at 25 nodes
    const densityFactor = Math.min(1.5, 1 + Math.max(0, nodeCount - 5) / 40)

    // Label factor: longer labels → more horizontal spacing (up to 1.4x)
    // Baseline is 15 chars, increases for longer labels
    const labelFactor = Math.min(1.4, 1 + Math.max(0, avgLabelLength - 15) / 50)

    // Aspect ratio adjustment: wider targets → compress vertical spacing
    const aspectAdjustment = aspectRatio > 1.5 ? 0.9 : 1.0

    return {
        hSpacing: Math.round(baseH * densityFactor * labelFactor),
        vSpacing: Math.round(baseV * densityFactor * aspectAdjustment),
        boundaryPadding: Math.round(BOUNDARY_PADDING * Math.min(1.3, densityFactor))
    }
}

/**
 * Calculate average label length from node labels
 */
export function calculateAvgLabelLength(labels: string[]): number {
    if (labels.length === 0) return 15
    const totalLength = labels.reduce((sum, label) => sum + label.length, 0)
    return Math.round(totalLength / labels.length)
}
