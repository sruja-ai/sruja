import type { C4Kind, C4Level } from '../c4-model'

export type Size = { width: number; height: number }

export interface TextMeasurer {
  measure(text: string, kind: C4Kind, level: C4Level, maxWidth?: number): Size
  measureMultiline(text: string, kind: C4Kind, level: C4Level, maxWidth: number): { width: number; height: number; lines: string[] }
  getLineHeight(kind: C4Kind, level: C4Level): number
  getDescent(kind: C4Kind, level: C4Level): number
}

export class MockTextMeasurer2 implements TextMeasurer {
  measure(text: string): Size { return { width: text.length * 8, height: 18 } }
  measureMultiline(text: string, _k: C4Kind, _l: C4Level, maxWidth: number) {
    const words = text.split(/\s+/).filter(Boolean)
    const lines: string[] = []
    let cur = ''
    let w = 0
    let maxW = 0
    for (const word of words) {
      const next = cur ? cur + ' ' + word : word
      const nw = next.length * 8
      if (nw > maxWidth && cur) { lines.push(cur); maxW = Math.max(maxW, w); cur = word; w = word.length * 8 } else { cur = next; w = nw }
    }
    if (cur) { lines.push(cur); maxW = Math.max(maxW, w) }
    return { width: maxW, height: lines.length * 18, lines }
  }
  getLineHeight(): number { return 18 }
  getDescent(): number { return 4 }
}
