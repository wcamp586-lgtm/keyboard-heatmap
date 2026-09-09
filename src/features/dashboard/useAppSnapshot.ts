import { useCallback } from 'react'
import type { DateSelection } from '../../lib/contracts'
import { selectionKey } from '../../lib/dates'
import { getAppSnapshot, nativeBridgeAvailable } from '../../lib/tauri'
import { useLocalQuery } from './useLocalQuery'

export function useAppSnapshot(selection: DateSelection, page: string) {
  const load = useCallback(() => getAppSnapshot(selection), [selection])
  const query = useLocalQuery(`${selectionKey(selection)}:${page}`, load, selection.range === 'today' && nativeBridgeAvailable() ? 3000 : null)
  return {
    snapshot: query.data,
    error: query.error ? '暂时无法读取本机状态。请重试；已有数据可能已过期。' : null,
    isLoading: query.isLoading,
    isStale: query.error && query.data !== null,
    refreshNow: query.refresh,
  }
}
