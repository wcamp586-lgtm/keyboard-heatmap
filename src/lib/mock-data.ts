import type { DashboardSnapshot, DateSelection, DayDetail, HistoryPage, HistoryTarget, KeyCount, RangeKind } from './contracts'
import { addDays, asSelection, calendarDays, dateFromString, localDate, resolveDates } from './dates'
import { ALL_KEYS, getKeyIdFromWireId, getKeyLabel } from './keyboard-layout'
import { RECOGNIZED_SHORTCUTS } from './shortcuts'

const hotKeys: Record<number, number> = { 39: 860, 40: 720, 41: 635, 59: 670, 60: 780, 61: 735, 62: 450, 63: 390, 70: 515, 91: 265, 94: 1240 }
const shortcutBases: Record<string, number> = { ctrl_c: 184, ctrl_v: 163, win_d: 142, alt_tab: 118, ctrl_z: 91, win_shift_s: 86, ctrl_s: 74, ctrl_a: 68, alt_f4: 58, win_e: 54, ctrl_f: 43, ctrl_x: 37, ctrl_y: 31, ctrl_shift_z: 29, ctrl_p: 22, ctrl_w: 18, ctrl_shift_s: 15, win_v: 14, ctrl_tab: 13, win_ctrl_left: 11 }
const baseCount = (id: number) => id % 11 === 0 ? 0 : 12 + (id * 29 + 17) % 118 + (hotKeys[id] ?? 0)
const sortKeys = (entries: KeyCount[]) => [...entries].filter((entry) => entry.pressCount > 0).sort((a, b) => b.pressCount - a.pressCount || a.keyId - b.keyId)
const firstMockDate = () => addDays(localDate(), -64)

// All demo views use identical synthetic hour buckets. No keyboard events.
function mockBuckets(date: string) {
  const start = Math.ceil(dateFromString(date).getTime() / 3_600_000) * 3_600_000
  const end = Math.min(dateFromString(addDays(date, 1)).getTime(), Math.floor(Date.now() / 3_600_000) * 3_600_000 + 3_600_000)
  const ordinal = Math.floor(Date.parse(`${date}T00:00:00Z`) / 86_400_000)
  const empty = date < firstMockDate() || (ordinal % 11 === 0 && date !== localDate())
  return Array.from({ length: Math.max(0, Math.ceil((end - start) / 3_600_000)) }, (_, index) => {
    const stamp = start + index * 3_600_000
    const hour = new Date(stamp).getHours()
    const weight = empty ? 0 : (hour >= 8 && hour <= 22 ? 0.8 + (hour * 7 % 11) / 10 : 0.08) * (0.6 + ordinal % 7 / 10) / 20
    return {
      stamp, hour,
      keys: ALL_KEYS.map(({ id }) => ({ keyId: id, pressCount: Math.floor(baseCount(id) * weight) })),
      shortcuts: RECOGNIZED_SHORTCUTS.map(({ id }) => ({ shortcutId: id, pressCount: Math.floor((shortcutBases[id] ?? 0) * weight) })),
    }
  })
}

export function getMockDayDetail(date: string, target?: HistoryTarget | null): DayDetail {
  resolveDates({ range: 'custom', startDate: date, endDate: date }, firstMockDate())
  return {
    date, generatedAt: Math.floor(Date.now() / 1000),
    hours: mockBuckets(date).map((bucket) => {
      const keys = target?.kind === 'shortcut' ? [] : bucket.keys.filter((key) => !target || key.keyId === getKeyIdFromWireId(target.id))
      const shortcuts = target?.kind === 'key' ? [] : bucket.shortcuts.filter((shortcut) => !target || shortcut.shortcutId === target.id)
      const offset = -new Date(bucket.stamp).getTimezoneOffset()
      return {
        hourStartUtc: bucket.stamp / 1000,
        label: `${String(bucket.hour).padStart(2, '0')}:${String(new Date(bucket.stamp).getMinutes()).padStart(2, '0')}`,
        utcOffset: `${offset >= 0 ? '+' : '-'}${String(Math.floor(Math.abs(offset) / 60)).padStart(2, '0')}:${String(Math.abs(offset) % 60).padStart(2, '0')}`,
        totalPresses: keys.reduce((sum, key) => sum + key.pressCount, 0),
        totalShortcuts: shortcuts.reduce((sum, shortcut) => sum + shortcut.pressCount, 0),
        topKey: sortKeys(keys)[0] ?? null,
      }
    }),
  }
}

export function getMockHistory(selection: DateSelection, target: HistoryTarget | null, requestedPage: number): HistoryPage {
  const { start, end } = resolveDates(selection, firstMockDate())
  const totalPages = Math.ceil(calendarDays(start, end) / 30)
  const page = Math.min(requestedPage, totalPages - 1)
  const pageEnd = addDays(end, -page * 30)
  const days = Array.from({ length: Math.min(30, calendarDays(start, pageEnd)) }, (_, index) => {
    const date = addDays(pageEnd, -index)
    const detail = getMockDayDetail(date, target)
    const counts = new Map<number, number>()
    if (target?.kind !== 'shortcut') {
      for (const bucket of mockBuckets(date)) for (const key of bucket.keys) {
        if (key.pressCount && (!target || key.keyId === getKeyIdFromWireId(target.id))) counts.set(key.keyId, (counts.get(key.keyId) ?? 0) + key.pressCount)
      }
    }
    const peak = [...detail.hours].sort((a, b) => b.totalPresses - a.totalPresses)[0]
    return {
      date, totalPresses: detail.hours.reduce((sum, hour) => sum + hour.totalPresses, 0),
      totalShortcuts: detail.hours.reduce((sum, hour) => sum + hour.totalShortcuts, 0),
      uniqueKeys: counts.size,
      topKey: sortKeys([...counts].map(([keyId, pressCount]) => ({ keyId, pressCount })))[0] ?? null,
      peakHour: peak?.totalPresses ? `${peak.label} (UTC${peak.utcOffset})` : null,
    }
  })
  return { generatedAt: Math.floor(Date.now() / 1000), rangeStartDate: start, rangeEndDate: end, page, totalPages, days }
}

export function getMockDashboard(input: RangeKind | DateSelection): DashboardSnapshot {
  const selection = asSelection(input)
  const { start, end } = resolveDates(selection, firstMockDate())
  const counts = new Map(ALL_KEYS.map(({ id }) => [id, 0]))
  const shortcutCounts = new Map<string, number>(RECOGNIZED_SHORTCUTS.map(({ id }) => [id, 0]))
  const hourly = Array.from({ length: 24 }, (_, hour) => ({ hour, pressCount: 0 }))
  for (let date = start < firstMockDate() ? firstMockDate() : start; date <= end; date = addDays(date, 1)) {
    for (const bucket of mockBuckets(date)) {
      for (const key of bucket.keys) {
        counts.set(key.keyId, counts.get(key.keyId)! + key.pressCount)
        hourly[bucket.hour].pressCount += key.pressCount
      }
      for (const shortcut of bucket.shortcuts) shortcutCounts.set(shortcut.shortcutId, shortcutCounts.get(shortcut.shortcutId)! + shortcut.pressCount)
    }
  }
  const heatmap = [...counts].map(([keyId, pressCount]) => ({ keyId, pressCount }))
  const allShortcuts = [...shortcutCounts].map(([shortcutId, pressCount]) => ({ shortcutId, pressCount }))
  const topKeys = sortKeys(heatmap).slice(0, 10)
  return {
    range: selection.range, rangeStartDate: start, rangeEndDate: end,
    totalPresses: heatmap.reduce((sum, key) => sum + key.pressCount, 0),
    totalShortcuts: allShortcuts.reduce((sum, shortcut) => sum + shortcut.pressCount, 0),
    activeKeyCount: heatmap.filter((key) => key.pressCount > 0).length,
    topKey: topKeys[0] ?? null, heatmap, topKeys,
    topShortcuts: [...allShortcuts].filter((shortcut) => shortcut.pressCount > 0).sort((a, b) => b.pressCount - a.pressCount).slice(0, 10),
    shortcutCounts: allShortcuts, hourly, isCollecting: true, collectionEnabled: true, collectionHealth: 'collecting',
  }
}

export const getMockLiveCounts = (): KeyCount[] => getMockDashboard('today').heatmap
export { getKeyLabel }
