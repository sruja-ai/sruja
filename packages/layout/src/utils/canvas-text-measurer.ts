import { TextMeasurer } from './text-measurer'
import { C4Kind, C4Level } from '../c4-model'

type Size = { width: number; height: number }

export class CanvasTextMeasurer implements TextMeasurer {
  private canvas?: HTMLCanvasElement | OffscreenCanvas
  private ctx?: CanvasRenderingContext2D | OffscreenCanvasRenderingContext2D


  constructor() {
    try {
      if (typeof document !== 'undefined') {
        this.canvas = document.createElement('canvas')
        this.ctx = this.canvas.getContext('2d') as any
      } else if (typeof OffscreenCanvas !== 'undefined') {
        this.canvas = new OffscreenCanvas(1, 1) as any
        this.ctx = (this.canvas as any).getContext('2d') as any
      }
    } catch {
      this.canvas = undefined
      this.ctx = undefined
    }
  }

  private setFont(kind: C4Kind, level: C4Level) {
    const fontSize = this.getLineHeight(kind, level) - 4
    const font = `${fontSize}px system-ui`
    if (this.ctx && 'font' in this.ctx) (this.ctx as any).font = font
  }

  measure(text: string, kind: C4Kind, level: C4Level, _maxWidth?: number): Size {
    if (this.ctx) {
      this.setFont(kind, level)
      const m = (this.ctx as any).measureText(text)
      const ascent = (m.actualBoundingBoxAscent ?? 0)
      const descent = (m.actualBoundingBoxDescent ?? 0)
      const height = ascent + descent || this.getLineHeight(kind, level)
      return { width: m.width || text.length * 8, height }
    }
    return { width: text.length * 8, height: this.getLineHeight(kind, level) }
  }

  measureMultiline(text: string, kind: C4Kind, level: C4Level, maxWidth: number): { width: number; height: number; lines: string[] } {
    const words = text.split(/\s+/).filter(Boolean)
    const lines: string[] = []
    let current = ''
    let maxW = 0
    const lineHeight = this.getLineHeight(kind, level)
    for (const w of words) {
      const next = current ? current + ' ' + w : w
      const m = this.measure(next, kind, level)
      if (m.width > maxWidth && current) {
        lines.push(current)
        maxW = Math.max(maxW, this.measure(current, kind, level).width)
        current = w
      } else {
        current = next
      }
    }
    if (current) {
      lines.push(current)
      maxW = Math.max(maxW, this.measure(current, kind, level).width)
    }
    return { width: maxW, height: lines.length * lineHeight, lines }
  }

  getLineHeight(_kind: C4Kind, _level: C4Level): number { return 18 }
  getDescent(_kind: C4Kind, _level: C4Level): number { return 4 }
}
