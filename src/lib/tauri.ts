import { invoke } from '@tauri-apps/api/core'
import type {
  CollectionHealth,
  DashboardSnapshot,
  KeyCount,
  LiveCounts,
  PrivacyStatus,
  RangeKind,
  ShortcutCount,
} from './contracts'
import { getMockDashboard, getMockLiveCounts } from './mock-data'
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
  range?: string
  totalPresses?: number
  totalShortcuts?: number
  uniqueKeys?: number
  keyCounts?: BackendKeyCount[]
  topKeys?: BackendKeyCount[]
  topShortcuts?: BackendShortcutCount[]
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

const browserPrivacyStatus = (enabled = true): PrivacyStatus => ({
  isCollecting: enabled,
  collectionEnabled: enabled,
  collectionHealth: enabled ? 'collecting' : 'paused',
  storage: '浏览器演示数据（不读取实体键盘）',
  capturesText: false,
})

export const getDashboard = async (range: RangeKind): Promise<DashboardSnapshot> => {
  if (!nativeBridgeAvailable()) return getMockDashboard(range)

  const [dashboard, privacy] = await Promise.all([
    invoke<BackendDashboard>('get_dashboard', { range }),
    invoke<BackendPrivacyStatus>('get_privacy_status'),
  ])
  const heatmap = toKeyCounts(dashboard.keyCounts)
  const topKeys = toKeyCounts(dashboard.topKeys)
  const topShortcuts = toShortcutCounts(dashboard.topShortcuts)
  const privacyStatus = toPrivacyStatus(privacy)

  return {
    range,
    totalPresses: toNonNegativeCount(dashboard.totalPresses),
    totalShortcuts: toNonNegativeCount(dashboard.totalShortcuts),
    activeKeyCount: toNonNegativeCount(dashboard.uniqueKeys),
    topKey: topKeys[0] ?? null,
    heatmap,
    topKeys,
    topShortcuts,
    hourly: (dashboard.hourlyDistribution ?? []).flatMap(({ hour, count }) => {
      if (!Number.isInteger(hour) || hour < 0 || hour > 23) return []

      return [{
        hour,
        pressCount: toNonNegativeCount(count),
      }]
    }),
    isCollecting: privacyStatus.isCollecting,
    collectionEnabled: privacyStatus.collectionEnabled,
    collectionHealth: privacyStatus.collectionHealth,
  }
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
