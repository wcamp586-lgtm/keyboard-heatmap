import { describe, expect, it } from 'vitest'
import { formatNumber } from './format'

describe('shared number formatter', () => {
  it('formats dashboard counts with the zh-CN grouping style', () => {
    expect(formatNumber(12_345)).toBe('12,345')
    expect(formatNumber(0)).toBe('0')
  })
})
