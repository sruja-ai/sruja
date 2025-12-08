import { describe, it, expect } from 'vitest'
import { flowLayout } from '../algorithms/flowLayout'
import { DefaultLayoutConfig } from '../presets/default'

describe('Flow layout', () => {
  it('lays out children vertically with spacing', () => {
    const children = [
      { id: 'a', size: { width: 100 as any, height: 40 as any } },
      { id: 'b', size: { width: 80 as any, height: 50 as any } },
      { id: 'c', size: { width: 120 as any, height: 60 as any } }
    ]
    const res = flowLayout(children, DefaultLayoutConfig)
    expect(res.children[0].bounds.y).toBe(DefaultLayoutConfig.padding as any)
    expect(res.children[1].bounds.y).toBe((DefaultLayoutConfig.padding + children[0].size.height + DefaultLayoutConfig.spacingY) as any)
    expect(res.nodes.bounds.width).toBe((120 + DefaultLayoutConfig.padding * 2) as any)
  })
})
