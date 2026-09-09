import { describe, expect, it } from 'vitest'
import { toAppSnapshot, toShortcutCounts } from './tauri'

describe('shortcut aggregate parser', () => {
  it('accepts only fixed shortcut aggregate rows and safely defaults to empty', () => {
    expect(toShortcutCounts(undefined)).toEqual([])
    expect(toShortcutCounts([
      { shortcutId: 'Ctrl + C', count: 12.2 },
      { shortcutId: 'ctrl_shift_z', count: 3 },
      { shortcutId: 'Win + D', count: 7 },
      { shortcutId: 'alt-f4', count: 5 },
      { shortcutId: 'ctrl_q', count: 99 },
      { shortcutId: 'typed_secret', count: 4 },
      { shortcutId: 'ctrl_v', count: -1 },
    ])).toEqual([
      { shortcutId: 'ctrl_c', pressCount: 12 },
      { shortcutId: 'ctrl_shift_z', pressCount: 3 },
      { shortcutId: 'win_d', pressCount: 7 },
      { shortcutId: 'alt_f4', pressCount: 5 },
    ])
  })
})

describe('authoritative app snapshot parser', () => {
  it('keeps degraded collection and storage states visible', () => {
    const snapshot = toAppSnapshot({
      generatedAt: 1_725_000_000,
      dashboard: {
        totalPresses: 12,
        totalShortcuts: 2,
        uniqueKeys: 3,
        keyCounts: [{ keyId: 'key_a', count: 12 }],
        topKeys: [{ keyId: 'key_a', count: 12 }],
        topShortcuts: [{ shortcutId: 'ctrl_c', count: 2 }],
        hourlyDistribution: [{ hour: 9, count: 12 }],
      },
      collection: {
        enabled: true,
        overall: 'degraded',
        physicalListener: 'active',
        shortcutListener: 'unavailable',
      },
      storage: {
        status: 'degraded',
        lastSuccessfulFlushAt: 1_724_999_900,
        lastErrorCode: 'sqlite_write_failed',
        consecutiveFailures: 2,
        storageMode: 'local-hourly-aggregate-only',
      },
    }, 'today')

    expect(snapshot.collection.overall).toBe('degraded')
    expect(snapshot.collection.shortcutListener).toBe('unavailable')
    expect(snapshot.storage).toMatchObject({
      status: 'degraded',
      lastErrorCode: 'sqlite_write_failed',
      consecutiveFailures: 2,
    })
    expect(snapshot.dashboard.isCollecting).toBe(true)
    expect(snapshot.dashboard.totalPresses).toBe(12)
  })

  it('fails closed when health fields are absent or unknown', () => {
    const snapshot = toAppSnapshot({
      collection: { enabled: true, overall: 'unknown' as never },
      storage: { status: 'unknown' as never },
    }, 'week')

    expect(snapshot.collection.overall).toBe('unavailable')
    expect(snapshot.collection.physicalListener).toBe('unavailable')
    expect(snapshot.storage.status).toBe('unavailable')
  })
})
