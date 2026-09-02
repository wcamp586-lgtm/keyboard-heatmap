import type { DashboardSnapshot, KeyCount, RangeKind, ShortcutCount } from './contracts'
import { ALL_KEYS, getKeyLabel } from './keyboard-layout'

const rangeMultiplier: Record<RangeKind, number> = {
  today: 1,
  week: 5.6,
  month: 21.4,
}

const hotKeyWeights: Record<number, number> = {
  39: 860, // W
  40: 720, // E
  41: 635, // R
  59: 670, // A
  60: 780, // S
  61: 735, // D
  62: 450, // F
  63: 390, // G
  70: 515, // Enter
  91: 265, // left Ctrl
  94: 1240, // Space
}

const shortcutBaseCounts: readonly ShortcutCount[] = [
  { shortcutId: 'ctrl_c', pressCount: 184 },
  { shortcutId: 'ctrl_v', pressCount: 163 },
  { shortcutId: 'win_d', pressCount: 142 },
  { shortcutId: 'alt_tab', pressCount: 118 },
  { shortcutId: 'ctrl_z', pressCount: 91 },
  { shortcutId: 'win_shift_s', pressCount: 86 },
  { shortcutId: 'ctrl_s', pressCount: 74 },
  { shortcutId: 'ctrl_a', pressCount: 68 },
  { shortcutId: 'alt_f4', pressCount: 58 },
  { shortcutId: 'win_e', pressCount: 54 },
  { shortcutId: 'ctrl_f', pressCount: 43 },
  { shortcutId: 'ctrl_x', pressCount: 37 },
  { shortcutId: 'ctrl_y', pressCount: 31 },
  { shortcutId: 'ctrl_shift_z', pressCount: 29 },
  { shortcutId: 'ctrl_p', pressCount: 22 },
  { shortcutId: 'ctrl_w', pressCount: 18 },
  { shortcutId: 'ctrl_shift_s', pressCount: 15 },
  { shortcutId: 'win_v', pressCount: 14 },
  { shortcutId: 'ctrl_tab', pressCount: 13 },
  { shortcutId: 'win_ctrl_left', pressCount: 11 },
]

const baseCountFor = (keyId: number) => {
  if (keyId % 11 === 0) return 0
  return 12 + ((keyId * 29 + 17) % 118) + (hotKeyWeights[keyId] ?? 0)
}

const sortDescending = (entries: readonly KeyCount[]) =>
  [...entries].sort((left, right) => right.pressCount - left.pressCount)

export const getMockDashboard = (range: RangeKind): DashboardSnapshot => {
  const multiplier = rangeMultiplier[range]
  const heatmap = ALL_KEYS.map(({ id }) => ({
    keyId: id,
    pressCount: Math.round(baseCountFor(id) * multiplier),
  }))
  const topKeys = sortDescending(heatmap).slice(0, 10)
  const totalPresses = heatmap.reduce((sum, entry) => sum + entry.pressCount, 0)
  const topShortcuts = shortcutBaseCounts
    .map(({ shortcutId, pressCount }) => ({
      shortcutId,
      pressCount: Math.round(pressCount * multiplier),
    }))
    .sort((left, right) => right.pressCount - left.pressCount)
    .slice(0, 10)
  const totalShortcuts = shortcutBaseCounts.reduce(
    (sum, { pressCount }) => sum + Math.round(pressCount * multiplier),
    0,
  )
  const hourly = Array.from({ length: 24 }, (_, hour) => {
    const commuteLift = hour >= 8 && hour <= 10 ? 1.35 : 1
    const eveningLift = hour >= 18 && hour <= 23 ? 1.55 : 1
    const quietHours = hour >= 1 && hour <= 6 ? 0.19 : 1
    const cadence = 32 + ((hour * 19 + 7) % 78)

    return {
      hour,
      pressCount: Math.round(cadence * commuteLift * eveningLift * quietHours * multiplier),
    }
  })

  return {
    range,
    totalPresses,
    totalShortcuts,
    activeKeyCount: heatmap.filter(({ pressCount }) => pressCount > 0).length,
    topKey: topKeys[0] ?? null,
    heatmap,
    topKeys,
    topShortcuts,
    hourly,
    isCollecting: true,
    collectionEnabled: true,
    collectionHealth: 'collecting',
  }
}

export const getMockLiveCounts = (): KeyCount[] => getMockDashboard('today').heatmap

export { getKeyLabel }
