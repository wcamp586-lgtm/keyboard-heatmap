import { afterEach, describe, expect, it, vi } from 'vitest'
import { startPolling } from './polling'

afterEach(() => vi.useRealTimers())

describe('visible, non-overlapping local refresh', () => {
  it('refreshes today every three seconds and recovers from errors', async () => {
    vi.useFakeTimers()
    const success = vi.fn(), error = vi.fn()
    const load = vi.fn().mockRejectedValueOnce(new Error('offline')).mockResolvedValue({ collection: 'paused' })
    const query = startPolling({ load, onSuccess: success, onError: error, interval: 3000, isVisible: () => true })
    await vi.advanceTimersByTimeAsync(0)
    expect(error).toHaveBeenCalledOnce()
    await vi.advanceTimersByTimeAsync(3000)
    expect(success).toHaveBeenCalledWith({ collection: 'paused' })
    load.mockResolvedValue({ collection: 'collecting' })
    query.refresh()
    await vi.advanceTimersByTimeAsync(0)
    expect(success).toHaveBeenLastCalledWith({ collection: 'collecting' })
    query.stop()
    await vi.advanceTimersByTimeAsync(6000)
    expect(load).toHaveBeenCalledTimes(3)
  })

  it('does not overlap, publish obsolete responses or poll fixed past ranges', async () => {
    vi.useFakeTimers()
    let resolve!: (value: string) => void
    const load = vi.fn(() => new Promise<string>((done) => { resolve = done }))
    const success = vi.fn()
    const query = startPolling({ load, onSuccess: success, onError: vi.fn(), interval: null, isVisible: () => true })
    query.refresh()
    await vi.advanceTimersByTimeAsync(9000)
    expect(load).toHaveBeenCalledOnce()
    query.stop()
    resolve('old range')
    await vi.advanceTimersByTimeAsync(0)
    expect(success).not.toHaveBeenCalled()
  })

  it('stops while hidden and reloads when explicitly shown', async () => {
    vi.useFakeTimers()
    let visible = false
    const load = vi.fn().mockResolvedValue(1)
    const query = startPolling({ load, onSuccess: vi.fn(), onError: vi.fn(), interval: 3000, isVisible: () => visible })
    expect(load).not.toHaveBeenCalled()
    visible = true
    query.refresh()
    await vi.advanceTimersByTimeAsync(0)
    expect(load).toHaveBeenCalledOnce()
    visible = false
    await vi.advanceTimersByTimeAsync(9000)
    expect(load).toHaveBeenCalledOnce()
    visible = true
    query.refresh()
    await vi.advanceTimersByTimeAsync(0)
    expect(load).toHaveBeenCalledTimes(2)
    query.stop()
  })
})
