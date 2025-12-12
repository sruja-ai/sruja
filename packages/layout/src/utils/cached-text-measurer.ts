import type { TextMeasurer } from './text-measurer'
import type { C4Kind, C4Level } from '../c4-model'

export class CachedTextMeasurer implements TextMeasurer {
  private base: TextMeasurer
  private cache = new Map<string, { width: number; height: number; lines?: string[] }>()
  constructor(base: TextMeasurer) { this.base = base }
  private key(text: string, kind: C4Kind, level: C4Level, maxWidth?: number) { return `${kind}:${level}:${maxWidth ?? ''}:${text}` }
  measure(text: string, kind: C4Kind, level: C4Level, maxWidth?: number) {
    const k = this.key(text, kind, level, maxWidth)
    const c = this.cache.get(k)
    if (c) return { width: c.width, height: c.height }
    const res = this.base.measure(text, kind, level, maxWidth)
    this.cache.set(k, res)
    return res
  }
  measureMultiline(text: string, kind: C4Kind, level: C4Level, maxWidth: number) {
    const hyphenated = hyphenate(text)
    const k = this.key(hyphenated, kind, level, maxWidth)
    const c = this.cache.get(k)
    if (c && c.lines) return { width: c.width, height: c.height, lines: c.lines }
    const res = this.base.measureMultiline(hyphenated, kind, level, maxWidth)
    this.cache.set(k, res)
    return res
  }
  getLineHeight(kind: C4Kind, level: C4Level) { return this.base.getLineHeight(kind, level) }
  getDescent(kind: C4Kind, level: C4Level) { return this.base.getDescent(kind, level) }
}

function hyphenate(text: string): string {
  return text.replace(/([a-z]{6,})/gi, (w) => w.slice(0, Math.floor(w.length / 2)) + '-' + w.slice(Math.floor(w.length / 2)))
}
