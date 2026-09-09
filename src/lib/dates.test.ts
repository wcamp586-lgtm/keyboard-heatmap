import { describe, expect, it } from 'vitest'
import { addDays, calendarDays, resolveDates, selectionKey, validateDates } from './dates'

describe('local calendar selections', () => {
  it('resolves yesterday, Monday, month and last 30 days without UTC date shifts', () => {
    expect(resolveDates({ range: 'yesterday' }, '2024-01-01', '2024-03-01')).toEqual({ start: '2024-02-29', end: '2024-02-29' })
    expect(resolveDates({ range: 'week' }, '2026-01-01', '2026-09-09').start).toBe('2026-09-07')
    expect(resolveDates({ range: 'month' }, '2026-01-01', '2026-09-09').start).toBe('2026-09-01')
    const last = resolveDates({ range: 'last30' }, '2026-01-01', '2026-09-09')
    expect(calendarDays(last.start, last.end)).toBe(30)
    expect(addDays('2024-02-28', 1)).toBe('2024-02-29')
  })

  it('rejects invalid, reversed and future dates and separates custom query keys', () => {
    for (const [start, end] of [['2026-02-30', '2026-03-01'], ['2026-09-09', '2026-09-08'], ['2026-09-09', '2026-09-10'], ['2026-9-01', '2026-09-09'], ['', '']]) {
      expect(validateDates(start, end, '2026-09-09')).not.toBeNull()
    }
    expect(validateDates('2026-09-09', '2026-09-09', '2026-09-09')).toBeNull()
    expect(selectionKey({ range: 'custom', startDate: '2026-09-01', endDate: '2026-09-02' })).not.toBe(selectionKey({ range: 'custom', startDate: '2026-09-03', endDate: '2026-09-04' }))
  })
})
