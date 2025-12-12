import type { TextMeasurer } from './text-measurer'
import type { C4Kind, C4Level } from '../c4-model'

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

  /**
   * Get font styles mirroring packages/viewer/src/style/node-styles.ts
   */
  private static getFontMetrics(kind: C4Kind) {
    if (kind === 'SoftwareSystem') {
      return { size: 14, weight: 700, uppercase: true, lineHeight: 20 }
    } else if (kind === 'Person') {
      return { size: 11, weight: 600, uppercase: false, lineHeight: 16 }
    } else {
      // Container, Component, others
      return { size: 12, weight: 500, uppercase: false, lineHeight: 18 }
    }
  }

  private setFont(kind: C4Kind) {
    if (this.ctx && 'font' in this.ctx) {
      const style = CanvasTextMeasurer.getFontMetrics(kind)
        // 'Inter' is the primary font in the UI, fall back to system-ui
        // Cast to any to assume 'font' property exists (validated by check above)
        ; (this.ctx as any).font = `${style.weight} ${style.size}px Inter, system-ui, sans-serif`
    }
  }

  measure(text: string, kind: C4Kind, _level: C4Level, _maxWidth?: number): Size {
    const style = CanvasTextMeasurer.getFontMetrics(kind)
    const transformText = style.uppercase ? text.toUpperCase() : text

    if (this.ctx) {
      this.setFont(kind)
      const m = (this.ctx as any).measureText(transformText)
      const ascent = (m.actualBoundingBoxAscent ?? 0)
      const descent = (m.actualBoundingBoxDescent ?? 0)
      const height = (ascent + descent) || style.lineHeight
      // Add small buffer to width for anti-aliasing diffs
      return { width: (m.width || transformText.length * 8) + 2, height }
    }
    // Fallback estimation
    return { width: transformText.length * (style.size * 0.6), height: style.lineHeight }
  }

  measureMultiline(text: string, kind: C4Kind, level: C4Level, maxWidth: number): { width: number; height: number; lines: string[] } {
    const style = CanvasTextMeasurer.getFontMetrics(kind)
    const transformText = style.uppercase ? text.toUpperCase() : text

    const words = transformText.split(/\s+/).filter(Boolean)
    const lines: string[] = []
    let current = ''
    let maxW = 0
    const lineHeight = style.lineHeight

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

  getLineHeight(kind: C4Kind, _level: C4Level): number {
    return CanvasTextMeasurer.getFontMetrics(kind).lineHeight
  }
  getDescent(_kind: C4Kind, _level: C4Level): number { return 4 }
}
