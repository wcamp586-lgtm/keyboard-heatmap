import { useEffect, useReducer, useState } from 'react'
import { startPolling } from './polling'

export function useLocalQuery<T>(key: string, load: () => Promise<T>, interval: number | null, enabled = true) {
  const [result, setResult] = useState<{ key: string; data: T | null; error: boolean } | null>(null)
  const [revision, refresh] = useReducer((value: number) => value + 1, 0)
  useEffect(() => {
    if (!enabled) return
    const polling = startPolling({
      load, interval, isVisible: () => document.visibilityState !== 'hidden',
      onSuccess: (data) => setResult({ key, data, error: false }),
      onError: () => setResult((old) => ({ key, data: old?.key === key ? old.data : null, error: true })),
    })
    const visible = () => { if (document.visibilityState === 'visible') polling.refresh() }
    document.addEventListener('visibilitychange', visible)
    return () => { polling.stop(); document.removeEventListener('visibilitychange', visible) }
  }, [key, load, interval, enabled, revision])
  const current = result?.key === key ? result : null
  return { data: current?.data ?? null, error: current?.error ?? false, isLoading: enabled && !current, refresh }
}
