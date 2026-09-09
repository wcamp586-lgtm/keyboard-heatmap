import { describe, expect, it } from 'vitest'
import { getMockDashboard } from '../../lib/mock-data'
import { filterRanking, getDetailedKeyLabel, initialRanking, rankingRows } from './ranking'

describe('complete ranking', () => {
  it('covers both fixed catalogs and retains range-wide shares during filtering', () => {
    const snapshot = getMockDashboard('last30')
    const keys = rankingRows(snapshot, 'key')
    const shortcuts = rankingRows(snapshot, 'shortcut')
    expect(keys).toHaveLength(104)
    expect(shortcuts).toHaveLength(85)
    expect(shortcuts.filter((row) => row.count > 0).length).toBeGreaterThan(10)
    expect(keys.reduce((sum, row) => sum + row.share, 0)).toBeCloseTo(100)
    expect(shortcuts.reduce((sum, row) => sum + row.share, 0)).toBeCloseTo(100)
    const filtered = filterRanking(keys, { ...initialRanking, search: '左 Ctrl' })
    expect(filtered).toHaveLength(1)
    expect(filtered[0].share).toBe(keys.find((row) => row.id === 'control_left')?.share)
    expect(filtered[0].rank).toBeGreaterThan(1)
  })

  it('distinguishes sides and numpad, filters zones, shows zero rows and sorts ties stably', () => {
    expect(getDetailedKeyLabel(91)).not.toBe(getDetailedKeyLabel(98))
    expect(getDetailedKeyLabel(70)).not.toBe(getDetailedKeyLabel(90))
    const snapshot = getMockDashboard({ range: 'custom', startDate: '2024-01-01', endDate: '2024-01-01' })
    const rows = rankingRows(snapshot, 'key')
    expect(filterRanking(rows, initialRanking)).toHaveLength(0)
    const all = filterRanking(rows, { ...initialRanking, showZero: true })
    expect(all).toHaveLength(104)
    expect(all.map((row) => row.order)).toEqual(Array.from({ length: 104 }, (_, index) => index))
    const numpad = filterRanking(rows, { ...initialRanking, showZero: true, zone: '数字小键盘', sort: 'name' })
    expect(numpad).toHaveLength(17)
    expect(numpad.every((row) => row.share === 0)).toBe(true)
    expect(filterRanking(rows, { ...initialRanking, showZero: true, search: 'numpad_enter' })[0].label).toBe('小键盘 Enter')
  })
})
