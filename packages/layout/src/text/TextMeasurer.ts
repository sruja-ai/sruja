import type { TextStyle } from '../types'

export type MeasuredLine = { text: string; width: number; lineHeight: number }

export interface TextMeasurer {
  measureLine(text: string, style: TextStyle): MeasuredLine
}

export class MockTextMeasurer implements TextMeasurer {
  measureLine(text: string, style: TextStyle): MeasuredLine {
    const baseWidth = text.length * Math.max(style.fontSize, 10) * 0.6
    const lineHeight = style.lineHeight ?? style.fontSize * 1.2
    return { text, width: baseWidth, lineHeight }
  }
}
