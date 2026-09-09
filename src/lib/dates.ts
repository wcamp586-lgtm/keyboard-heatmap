import type { DateSelection, RangeKind } from './contracts'

export const localDate = (date = new Date()) =>
  `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`

export const dateFromString = (value: string) => {
  const [year, month, day] = value.split('-').map(Number)
  return new Date(year, month - 1, day)
}

export const addDays = (value: string, days: number) => {
  const date = dateFromString(value)
  date.setDate(date.getDate() + days)
  return localDate(date)
}

export const calendarDays = (start: string, end: string) => {
  const utcDate = (date: string) => Date.parse(`${date}T00:00:00Z`)
  return Math.round((utcDate(end) - utcDate(start)) / 86_400_000) + 1
}

export const validateDates = (start: string, end: string, today = localDate()): string | null => {
  const valid = (value: string) => /^\d{4}-\d{2}-\d{2}$/.test(value)
    && value >= '1970-01-01' && localDate(dateFromString(value)) === value
  if (!valid(start) || !valid(end)) return '请选择有效的开始和结束日期。'
  if (start > end) return '开始日期不能晚于结束日期。'
  if (end > today) return '不能查询未来日期。'
  return null
}

export const asSelection = (selection: DateSelection | RangeKind): DateSelection =>
  typeof selection === 'string' ? { range: selection } : selection

export const selectionKey = (selection: DateSelection) =>
  [selection.range, selection.startDate ?? '', selection.endDate ?? ''].join(':')

export const resolveDates = (selection: DateSelection, firstDate: string, today = localDate()) => {
  let start = today
  let end = today
  switch (selection.range) {
    case 'today': break
    case 'yesterday': start = end = addDays(today, -1); break
    case 'week': start = addDays(today, -((dateFromString(today).getDay() + 6) % 7)); break
    case 'month': start = `${today.slice(0, 8)}01`; break
    case 'last30': start = addDays(today, -29); break
    case 'all': start = firstDate < today ? firstDate : today; break
    case 'custom': start = selection.startDate ?? ''; end = selection.endDate ?? ''; break
  }
  const error = validateDates(start, end, today)
  if (error) throw new Error(error)
  return { start, end }
}
