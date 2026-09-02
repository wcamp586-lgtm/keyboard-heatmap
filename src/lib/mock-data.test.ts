import { describe, expect, it } from 'vitest'
import { getMockDashboard } from './mock-data'

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
})
