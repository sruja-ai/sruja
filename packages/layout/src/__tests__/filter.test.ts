import { describe, it, expect } from 'vitest'
import { applyFilter, C4ViewFilter } from '../c4-view'

describe('applyFilter', () => {
  it('includes by tag and excludes by kind', () => {
    const node = { id: 'X' as any, kind: 'SoftwareSystem' as any, tags: new Set(['critical']) }
    const filter: C4ViewFilter = { includeTags: new Set(['critical']), excludeKinds: new Set(['Container'] as any) }
    expect(applyFilter(node as any, filter)).toBe(true)
  })

  it('excludes matching exclude tags', () => {
    const node = { id: 'Y' as any, kind: 'Container' as any, tags: new Set(['deprecated']) }
    const filter: C4ViewFilter = { excludeTags: new Set(['deprecated']) }
    expect(applyFilter(node as any, filter)).toBe(false)
  })
})
