import { invoke } from '@tauri-apps/api/core'
import type {
  AppSnapshot,
  CollectionStatus,
  CollectionHealth,
  DashboardSnapshot,
  KeyCount,
  LiveCounts,
  PrivacyStatus,
  RangeKind,
  ShortcutCount,
  StorageStatus,
  DateSelection, HistoryTarget, HistoryPage, DayDetail, DailyRecord, HourRecord,
} from './contracts'
import { getMockDashboard, getMockLiveCounts, getMockHistory, getMockDayDetail } from './mock-data'
import { asSelection } from './dates'
import { getKeyIdFromWireId } from './keyboard-layout'
import { isRecognizedShortcutId, normalizeShortcutId } from './shortcuts'

type BackendKeyCount = {
  keyId: string | number
  count: number
}

type BackendShortcutCount = {
  shortcutId: string
  count: number
}

type BackendDashboard = {
  rangeStartDate?: string
  rangeEndDate?: string
  range?: string
  totalPresses?: number
  totalShortcuts?: number
  uniqueKeys?: number
  keyCounts?: BackendKeyCount[]
  topKeys?: BackendKeyCount[]
  topShortcuts?: BackendShortcutCount[]
  shortcutCounts?: BackendShortcutCount[]
  hourlyDistribution?: Array<{ hour: number; count: number }>
}

type BackendPrivacyStatus = {
  collectionEnabled?: boolean
  collection_enabled?: boolean
  rawInputActive?: boolean
  raw_input_active?: boolean
  collectionHealth?: string
  collection_health?: string
  storageMode?: string
}

type BackendLiveCounts = BackendPrivacyStatus & {
  keyCounts?: BackendKeyCount[]
}

type BackendAppSnapshot = {
  generatedAt?: number
  dashboard?: BackendDashboard
  collection?: Partial<CollectionStatus>
  storage?: Partial<StorageStatus>
}

let browserCollectionEnabled = true

export const nativeBridgeAvailable = () =>
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

const toFrontEndKeyId = (keyId: string | number) => {
  if (typeof keyId === 'number') return Number.isInteger(keyId) && keyId >= 0 && keyId < 104 ? keyId : undefined
  return getKeyIdFromWireId(keyId)
}

const toNonNegativeCount = (value: unknown) =>
  typeof value === 'number' && Number.isFinite(value) && value >= 0 ? Math.round(value) : 0

const toKeyCounts = (entries: readonly BackendKeyCount[] | undefined): KeyCount[] =>
  (entries ?? []).flatMap(({ keyId, count }) => {
    const frontEndKeyId = toFrontEndKeyId(keyId)
    if (frontEndKeyId === undefined || !Number.isFinite(count) || count < 0) return []

    return [{ keyId: frontEndKeyId, pressCount: Math.round(count) }]
  })

export const toShortcutCounts = (entries: readonly BackendShortcutCount[] | undefined): ShortcutCount[] =>
  (entries ?? []).flatMap(({ shortcutId, count }) => {
    const normalizedShortcutId = normalizeShortcutId(shortcutId)
    if (!isRecognizedShortcutId(normalizedShortcutId) || !Number.isFinite(count) || count < 0) return []

    return [{ shortcutId: normalizedShortcutId, pressCount: Math.round(count) }]
  })

const toCollectionHealth = (status: BackendPrivacyStatus): {
  collectionEnabled: boolean
  collectionHealth: CollectionHealth
} => {
  const collectionEnabled = status.collectionEnabled ?? status.collection_enabled ?? false
  const rawInputActive = status.rawInputActive ?? status.raw_input_active
  const reportedHealth = status.collectionHealth ?? status.collection_health

  if (reportedHealth === 'collecting' || reportedHealth === 'paused' || reportedHealth === 'unavailable') {
    return { collectionEnabled, collectionHealth: reportedHealth }
  }

  if (!collectionEnabled) return { collectionEnabled, collectionHealth: 'paused' }
  return { collectionEnabled, collectionHealth: rawInputActive === false ? 'unavailable' : 'collecting' }
}

const toPrivacyStatus = (status: BackendPrivacyStatus): PrivacyStatus => {
  const { collectionEnabled, collectionHealth } = toCollectionHealth(status)

  return {
    isCollecting: collectionHealth === 'collecting',
    collectionEnabled,
    collectionHealth,
    storage: status.storageMode || '本机 SQLite 聚合库',
    capturesText: false,
  }
}

const browserPrivacyStatus = (enabled = browserCollectionEnabled): PrivacyStatus => ({
  isCollecting: enabled,
  collectionEnabled: enabled,
  collectionHealth: enabled ? 'collecting' : 'paused',
  storage: '浏览器演示数据（不读取实体键盘）',
  capturesText: false,
})

const browserCollectionStatus = (): CollectionStatus => ({
  enabled: browserCollectionEnabled,
  overall: browserCollectionEnabled ? 'collecting' : 'paused',
  physicalListener: browserCollectionEnabled ? 'active' : 'paused',
  shortcutListener: browserCollectionEnabled ? 'active' : 'paused',
})

const browserStorageStatus = (): StorageStatus => ({
  status: 'healthy',
  lastSuccessfulFlushAt: null,
  lastErrorCode: null,
  consecutiveFailures: 0,
  storageMode: 'browser-demo',
})

export const getBrowserAppSnapshot = (range: RangeKind | DateSelection): AppSnapshot => {
  const collection = browserCollectionStatus()
  const dashboard = getMockDashboard(range)
  return {
    generatedAt: Math.floor(Date.now() / 1_000),
    dashboard: {
      ...dashboard,
      isCollecting: collection.overall === 'collecting',
      collectionEnabled: collection.enabled,
      collectionHealth: collection.overall,
    },
    collection,
    storage: browserStorageStatus(),
  }
}

const isCollectionHealth = (value: unknown): value is CollectionHealth =>
  value === 'collecting' || value === 'degraded' || value === 'paused' || value === 'unavailable'

export const toAppSnapshot = (result: BackendAppSnapshot, range: RangeKind): AppSnapshot => {
  const dashboard = result.dashboard ?? {}
  const rawCollection = result.collection ?? {}
  const enabled = rawCollection.enabled === true
  const overall = isCollectionHealth(rawCollection.overall)
    ? rawCollection.overall
    : enabled
      ? 'unavailable'
      : 'paused'
  const collection: CollectionStatus = {
    enabled,
    overall,
    physicalListener: rawCollection.physicalListener === 'active' || rawCollection.physicalListener === 'paused'
      ? rawCollection.physicalListener
      : 'unavailable',
    shortcutListener: rawCollection.shortcutListener === 'active' || rawCollection.shortcutListener === 'paused'
      ? rawCollection.shortcutListener
      : 'unavailable',
  }
  const heatmap = toKeyCounts(dashboard.keyCounts)
  const topKeys = toKeyCounts(dashboard.topKeys)
  const storageStatus = result.storage?.status
  const storage: StorageStatus = {
    status: storageStatus === 'healthy' || storageStatus === 'degraded' ? storageStatus : 'unavailable',
    lastSuccessfulFlushAt: typeof result.storage?.lastSuccessfulFlushAt === 'number'
      ? result.storage.lastSuccessfulFlushAt
      : null,
    lastErrorCode: typeof result.storage?.lastErrorCode === 'string' ? result.storage.lastErrorCode : null,
    consecutiveFailures: toNonNegativeCount(result.storage?.consecutiveFailures),
    storageMode: typeof result.storage?.storageMode === 'string'
      ? result.storage.storageMode
      : 'local-hourly-aggregate-only',
  }

  return {
    generatedAt: typeof result.generatedAt === 'number' ? result.generatedAt : Math.floor(Date.now() / 1_000),
    dashboard: {
      range,
      rangeStartDate: dashboard.rangeStartDate ?? '',
      rangeEndDate: dashboard.rangeEndDate ?? '',
      totalPresses: toNonNegativeCount(dashboard.totalPresses),
      totalShortcuts: toNonNegativeCount(dashboard.totalShortcuts),
      activeKeyCount: toNonNegativeCount(dashboard.uniqueKeys),
      topKey: topKeys[0] ?? null,
      heatmap,
      topKeys,
      topShortcuts: toShortcutCounts(dashboard.topShortcuts),
      shortcutCounts: toShortcutCounts(dashboard.shortcutCounts),
      hourly: (dashboard.hourlyDistribution ?? []).flatMap(({ hour, count }) => {
        if (!Number.isInteger(hour) || hour < 0 || hour > 23) return []
        return [{ hour, pressCount: toNonNegativeCount(count) }]
      }),
      isCollecting: overall === 'collecting' || overall === 'degraded',
      collectionEnabled: enabled,
      collectionHealth: overall,
    },
    collection,
    storage,
  }
}

export const getAppSnapshot = async (input: RangeKind | DateSelection): Promise<AppSnapshot> => {
  const selection = asSelection(input)
  if (!nativeBridgeAvailable()) return getBrowserAppSnapshot(selection)
  const result = await invoke<BackendAppSnapshot>('get_app_snapshot', { ...selection })
  return toAppSnapshot(result, selection.range)
}

type BackendDaily = Omit<DailyRecord, 'topKey'> & { topKey: BackendKeyCount | null }
type BackendHour = Omit<HourRecord, 'topKey'> & { topKey: BackendKeyCount | null }
export const getHistory = async (selection: DateSelection, target: HistoryTarget | null, page: number): Promise<HistoryPage> => {
  if (!nativeBridgeAvailable()) return getMockHistory(selection, target, page)
  const result = await invoke<Omit<HistoryPage, 'days'> & { days: BackendDaily[] }>('get_history', { ...selection, target, page })
  return { ...result, days: result.days.map((day) => ({ ...day, topKey: toKeyCounts(day.topKey ? [day.topKey] : [])[0] ?? null })) }
}
export const getDayDetail = async (date: string, target: HistoryTarget | null): Promise<DayDetail> => {
  if (!nativeBridgeAvailable()) return getMockDayDetail(date, target)
  const result = await invoke<Omit<DayDetail, 'hours'> & { hours: BackendHour[] }>('get_day_detail', { date, target })
  return { ...result, hours: result.hours.map((hour) => ({ ...hour, topKey: toKeyCounts(hour.topKey ? [hour.topKey] : [])[0] ?? null })) }
}

export const getDashboard = async (range: RangeKind): Promise<DashboardSnapshot> => {
  return (await getAppSnapshot(range)).dashboard
}

export const getLiveCounts = async (): Promise<LiveCounts> => {
  if (!nativeBridgeAvailable()) {
    return { counts: getMockLiveCounts(), ...browserPrivacyStatus() }
  }

  const result = await invoke<BackendLiveCounts>('get_live_counts')
  const privacyStatus = toPrivacyStatus(result)
  return {
    counts: toKeyCounts(result.keyCounts),
    isCollecting: privacyStatus.isCollecting,
    collectionEnabled: privacyStatus.collectionEnabled,
    collectionHealth: privacyStatus.collectionHealth,
  }
}

export const setCollectionEnabled = async (enabled: boolean): Promise<PrivacyStatus> => {
  if (!nativeBridgeAvailable()) {
    browserCollectionEnabled = enabled
    return browserPrivacyStatus(enabled)
  }

  const status = await invoke<BackendPrivacyStatus>('set_collection_enabled', { enabled })
  return toPrivacyStatus(status)
}

export const getPrivacyStatus = async (): Promise<PrivacyStatus> => {
  if (!nativeBridgeAvailable()) {
    return browserPrivacyStatus()
  }

  const status = await invoke<BackendPrivacyStatus>('get_privacy_status')
  return toPrivacyStatus(status)
}
