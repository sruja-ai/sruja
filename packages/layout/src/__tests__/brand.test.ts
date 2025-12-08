import { describe, it, expect } from 'vitest'
import { createC4Id } from '../brand'

describe('Brand typing', () => {
  it('brands string as C4Id', () => {
    const id = createC4Id('system-1')
    expect(id).toBe('system-1')
  })
})
