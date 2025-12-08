import { TextStyle } from '../types'
import { TextMeasurer } from './TextMeasurer'

export function wrapText(text: string, style: TextStyle, measurer: TextMeasurer, maxWidth: number): { lines: string[]; widths: number[]; lineHeight: number } {
  const words = text.split(/\s+/).filter(Boolean)
  const lines: string[] = []
  const widths: number[] = []
  let current = ''
  let currentWidth = 0
  const lh = measurer.measureLine('A', style).lineHeight
  for (const w of words) {
    const next = current ? current + ' ' + w : w
    const nextWidth = measurer.measureLine(next, style).width
    if (nextWidth > maxWidth && current) {
      lines.push(current)
      widths.push(currentWidth)
      current = w
      currentWidth = measurer.measureLine(current, style).width
    } else {
      current = next
      currentWidth = nextWidth
    }
  }
  if (current) {
    lines.push(current)
    widths.push(currentWidth)
  }
  return { lines, widths, lineHeight: lh }
}
