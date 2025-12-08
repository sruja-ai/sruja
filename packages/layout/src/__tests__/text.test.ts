import { describe, it, expect } from 'vitest'
import { MockTextMeasurer } from '../text/TextMeasurer'
import { wrapText } from '../text/wrap'

describe('Text wrapping', () => {
  it('wraps long text into multiple lines', () => {
    const measurer = new MockTextMeasurer()
    const style = { fontFamily: 'Inter', fontSize: 16 }
    const res = wrapText('hello world from sruja layout', style, measurer, 120)
    expect(res.lines.length).toBeGreaterThan(1)
    expect(Math.max(...res.widths)).toBeLessThanOrEqual(120)
  })
})
