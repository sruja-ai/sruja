import { describe, it, expect } from 'vitest'
import {
    getAdaptiveSpacing,
    calculateAvgLabelLength,
    H_SPACING,
    V_SPACING,
    BOUNDARY_PADDING
} from '../constants'

describe('Adaptive Spacing', () => {
    describe('getAdaptiveSpacing', () => {
        it('returns base constants in fixed mode', () => {
            const result = getAdaptiveSpacing(10, 15, 16 / 9, 'fixed')

            expect(result.hSpacing).toBe(H_SPACING)
            expect(result.vSpacing).toBe(V_SPACING)
            expect(result.boundaryPadding).toBe(BOUNDARY_PADDING)
        })

        it('increases spacing for presentation mode', () => {
            const result = getAdaptiveSpacing(10, 15, 16 / 9, 'presentation')

            expect(result.hSpacing).toBeGreaterThan(H_SPACING)
            expect(result.vSpacing).toBeGreaterThan(V_SPACING)
            expect(result.boundaryPadding).toBeGreaterThan(BOUNDARY_PADDING)
        })

        it('increases spacing for more nodes in adaptive mode', () => {
            const sparse = getAdaptiveSpacing(3, 15, 16 / 9, 'adaptive')
            const dense = getAdaptiveSpacing(20, 15, 16 / 9, 'adaptive')

            expect(dense.hSpacing).toBeGreaterThan(sparse.hSpacing)
            expect(dense.vSpacing).toBeGreaterThan(sparse.vSpacing)
        })

        it('increases horizontal spacing for longer labels', () => {
            const shortLabels = getAdaptiveSpacing(10, 10, 16 / 9, 'adaptive')
            const longLabels = getAdaptiveSpacing(10, 40, 16 / 9, 'adaptive')

            expect(longLabels.hSpacing).toBeGreaterThan(shortLabels.hSpacing)
            // Vertical spacing should be less affected by label length
        })

        it('compresses vertical spacing for wide aspect ratios', () => {
            const narrow = getAdaptiveSpacing(10, 15, 1.0, 'adaptive')
            const wide = getAdaptiveSpacing(10, 15, 2.0, 'adaptive')

            expect(wide.vSpacing).toBeLessThan(narrow.vSpacing)
        })

        it('caps density factor at 1.5x', () => {
            const result = getAdaptiveSpacing(100, 15, 16 / 9, 'adaptive')

            // Max is 1.5x base
            expect(result.hSpacing).toBeLessThanOrEqual(Math.round(H_SPACING * 1.5 * 1.4)) // 1.5 density * 1.4 label
        })
    })

    describe('calculateAvgLabelLength', () => {
        it('returns default for empty array', () => {
            expect(calculateAvgLabelLength([])).toBe(15)
        })

        it('calculates average correctly', () => {
            const labels = ['Short', 'A Bit Longer Label', 'Medium Sized']
            const avg = calculateAvgLabelLength(labels)

            const expected = Math.round((5 + 18 + 12) / 3)
            expect(avg).toBe(expected)
        })

        it('handles single label', () => {
            expect(calculateAvgLabelLength(['Hello World'])).toBe(11)
        })
    })
})
