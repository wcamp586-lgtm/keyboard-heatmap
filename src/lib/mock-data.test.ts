import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { getMockDashboard, getMockDayDetail, getMockHistory } from './mock-data'

beforeEach(() => { vi.useFakeTimers(); vi.setSystemTime(new Date(2026, 8, 9, 18, 30)) })
afterEach(() => vi.useRealTimers())

describe('dashboard aggregate mock', () => {
  it('only exposes aggregate structures for all dashboard ranges', () => {
    for (const range of ['today', 'week', 'month'] as const) {
      const snapshot = getMockDashboard(range)

      expect(snapshot.range).toBe(range)
      expect(snapshot.heatmap).toHaveLength(104)
      expect(snapshot.topKeys).toHaveLength(10)
      expect(snapshot.topShortcuts).toHaveLength(10)
      expect(snapshot.hourly).toHaveLength(24)
      expect(snapshot.activeKeyCount).toBeGreaterThan(0)
      expect(snapshot.totalPresses).toBeGreaterThan(0)
      expect(snapshot.totalShortcuts).toBeGreaterThan(0)
      expect(Object.keys(snapshot.heatmap[0] ?? {})).toEqual(['keyId', 'pressCount'])
      expect(Object.keys(snapshot.topShortcuts[0] ?? {})).toEqual(['shortcutId', 'pressCount'])
      expect(Object.keys(snapshot.hourly[0] ?? {})).toEqual(['hour', 'pressCount'])
    }
  })

  it('makes wider time ranges larger while preserving a deterministic top key', () => {
    const today = getMockDashboard('today')
    const week = getMockDashboard('week')
    const month = getMockDashboard('month')

    expect(week.totalPresses).toBeGreaterThan(today.totalPresses)
    expect(month.totalPresses).toBeGreaterThan(week.totalPresses)
    expect(week.totalShortcuts).toBeGreaterThan(today.totalShortcuts)
    expect(month.totalShortcuts).toBeGreaterThan(week.totalShortcuts)
    expect(today.topKey?.keyId).toBe(94)
    expect(week.topKey?.keyId).toBe(94)
    expect(today.topShortcuts[0]?.shortcutId).toBe('ctrl_c')
    expect(week.topShortcuts[0]?.shortcutId).toBe('ctrl_c')
    expect(today.topShortcuts.map(({ shortcutId }) => shortcutId)).toContain('win_d')
    expect(today.topShortcuts.map(({ shortcutId }) => shortcutId)).toContain('alt_tab')
  })

  it('keeps detailed rankings, daily pages and hours consistent', () => {
    const selection = { range: 'last30' } as const
    const snapshot = getMockDashboard(selection)
    const history = getMockHistory(selection, null, 0)
    expect(snapshot.shortcutCounts).toHaveLength(85)
    expect(snapshot.totalPresses).toBe(history.days.reduce((sum, day) => sum + day.totalPresses, 0))
    expect(snapshot.totalShortcuts).toBe(history.days.reduce((sum, day) => sum + day.totalShortcuts, 0))
    expect(snapshot.hourly.reduce((sum, hour) => sum + hour.pressCount, 0)).toBe(snapshot.totalPresses)
    expect(history.days.some((day) => !day.totalPresses && !day.totalShortcuts)).toBe(true)
    const target = { kind: 'key', id: 'space' } as const
    const targeted = getMockHistory(selection, target, 0)
    expect(targeted.days.reduce((sum, day) => sum + day.totalPresses, 0)).toBe(snapshot.heatmap.find((key) => key.keyId === 94)?.pressCount)
    const today = getMockDayDetail('2026-09-09', target)
    expect(today.hours).toHaveLength(19)
    expect(today.hours.reduce((sum, hour) => sum + hour.totalPresses, 0)).toBe(targeted.days[0].totalPresses)
  })

  it('paginates all history and retains old empty calendar dates', () => {
    const all = getMockHistory({ range: 'all' }, null, 99)
    expect(all.page).toBe(2)
    expect(all.totalPages).toBe(3)
    expect(all.days).toHaveLength(5)
    const old = getMockHistory({ range: 'custom', startDate: '2024-02-28', endDate: '2024-03-01' }, null, 0)
    expect(old.days.map((day) => day.date)).toEqual(['2024-03-01', '2024-02-29', '2024-02-28'])
    expect(old.days.every((day) => day.totalPresses === 0 && day.topKey === null)).toBe(true)
  })
})
