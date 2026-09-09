export const RANGE_OPTIONS = [
  { value: 'today', label: '今天' },
  { value: 'yesterday', label: '昨天' },
  { value: 'week', label: '本周' },
  { value: 'month', label: '本月' },
  { value: 'last30', label: '近 30 天' },
  { value: 'all', label: '全部' },
  { value: 'custom', label: '自定义' },
] as const

export type RangeKind = (typeof RANGE_OPTIONS)[number]['value']

export type DateSelection = { range: RangeKind; startDate?: string; endDate?: string }
export type HistoryTarget = { kind: 'key' | 'shortcut'; id: string }
export type DailyRecord = {
  date: string
  totalPresses: number
  totalShortcuts: number
  uniqueKeys: number
  topKey: KeyCount | null
  peakHour: string | null
}
export type HistoryPage = {
  generatedAt: number
  rangeStartDate: string
  rangeEndDate: string
  page: number
  totalPages: number
  days: DailyRecord[]
}
export type HourRecord = {
  hourStartUtc: number
  label: string
  utcOffset: string
  totalPresses: number
  totalShortcuts: number
  topKey: KeyCount | null
}
export type DayDetail = { generatedAt: number; date: string; hours: HourRecord[] }

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

export type CollectionHealth = 'collecting' | 'degraded' | 'paused' | 'unavailable'
export type ListenerHealth = 'active' | 'paused' | 'unavailable'
export type StorageHealth = 'healthy' | 'degraded' | 'unavailable'

export type DashboardSnapshot = {
  range: RangeKind
  rangeStartDate: string
  rangeEndDate: string
  totalPresses: number
  totalShortcuts: number
  activeKeyCount: number
  topKey: KeyCount | null
  heatmap: KeyCount[]
  topKeys: KeyCount[]
  topShortcuts: ShortcutCount[]
  shortcutCounts: ShortcutCount[]
  hourly: HourlyCount[]
  isCollecting: boolean
  collectionEnabled: boolean
  collectionHealth: CollectionHealth
}

export type CollectionStatus = {
  enabled: boolean
  overall: CollectionHealth
  physicalListener: ListenerHealth
  shortcutListener: ListenerHealth
}

export type StorageStatus = {
  status: StorageHealth
  lastSuccessfulFlushAt: number | null
  lastErrorCode: string | null
  consecutiveFailures: number
  storageMode: string
}

export type AppSnapshot = {
  generatedAt: number
  dashboard: DashboardSnapshot
  collection: CollectionStatus
  storage: StorageStatus
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
