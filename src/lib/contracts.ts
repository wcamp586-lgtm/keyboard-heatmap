export const RANGE_OPTIONS = [
  { value: 'today', label: '今天' },
  { value: 'week', label: '本周' },
  { value: 'month', label: '本月' },
] as const

export type RangeKind = (typeof RANGE_OPTIONS)[number]['value']

export type KeyCount = {
  keyId: number
  pressCount: number
}

export type ShortcutCount = {
  shortcutId: string
  pressCount: number
}

export type HourlyCount = {
  hour: number
  pressCount: number
}

export type CollectionHealth = 'collecting' | 'paused' | 'unavailable'

export type DashboardSnapshot = {
  range: RangeKind
  totalPresses: number
  totalShortcuts: number
  activeKeyCount: number
  topKey: KeyCount | null
  heatmap: KeyCount[]
  topKeys: KeyCount[]
  topShortcuts: ShortcutCount[]
  hourly: HourlyCount[]
  isCollecting: boolean
  collectionEnabled: boolean
  collectionHealth: CollectionHealth
}

export type LiveCounts = {
  counts: KeyCount[]
  isCollecting: boolean
  collectionEnabled: boolean
  collectionHealth: CollectionHealth
}

export type PrivacyStatus = {
  isCollecting: boolean
  collectionEnabled: boolean
  collectionHealth: CollectionHealth
  storage: string
  capturesText: false
}
