import { describe, it, expect } from 'vitest'
import { CanvasTextMeasurer } from '../utils/canvas-text-measurer'

describe('CanvasTextMeasurer', () => {
  it('measures text width and multiline gracefully', () => {
    const m = new CanvasTextMeasurer()
    const s = m.measure('Hello World', 'SoftwareSystem' as any, 'context' as any)
    expect(s.width).toBeGreaterThan(0)
    const ml = m.measureMultiline('Hello world from canvas measurer example', 'SoftwareSystem' as any, 'context' as any, 100)
    expect(ml.lines.length).toBeGreaterThan(1)
    expect(ml.width).toBeLessThanOrEqual(100)
  })
})
