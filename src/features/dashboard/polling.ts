type PollingOptions<T> = {
  load: () => Promise<T>
  onSuccess: (data: T) => void
  onError: () => void
  isVisible: () => boolean
  interval: number | null
}

// The owner cancels this generation on every query change. A late response
// cannot publish into a different range, page, or selected key.
export function startPolling<T>(options: PollingOptions<T>) {
  let active = true
  let inFlight = false
  let timer: ReturnType<typeof setTimeout> | undefined
  const refresh = async () => {
    if (!active || inFlight || !options.isVisible()) return
    if (timer !== undefined) clearTimeout(timer)
    inFlight = true
    try {
      const data = await options.load()
      if (active) options.onSuccess(data)
    } catch {
      if (active) options.onError()
    } finally {
      inFlight = false
      if (active && options.interval !== null && options.isVisible()) timer = setTimeout(() => void refresh(), options.interval)
    }
  }
  void refresh()
  return {
    refresh: () => { void refresh() },
    stop: () => { active = false; if (timer !== undefined) clearTimeout(timer) },
  }
}
