import { useState } from 'react'
import { RANGE_OPTIONS, type DateSelection } from '../lib/contracts'
import { localDate, validateDates } from '../lib/dates'

export function DateRangePicker({ selection, onChange }: { selection: DateSelection; onChange: (next: DateSelection) => void }) {
  const [customOpen, setCustomOpen] = useState(false)
  const [start, setStart] = useState(selection.startDate ?? localDate())
  const [end, setEnd] = useState(selection.endDate ?? localDate())
  const [error, setError] = useState<string | null>(null)
  return <div className="date-controls">
    <div className="range-picker" aria-label="选择统计范围">
      {RANGE_OPTIONS.map((option) => <button type="button" key={option.value}
        className={selection.range === option.value ? 'is-active' : ''} aria-pressed={selection.range === option.value}
        onClick={() => {
          setError(null)
          setCustomOpen(option.value === 'custom')
          if (option.value !== 'custom') onChange({ range: option.value })
        }}>{option.label}</button>)}
    </div>
    {customOpen || selection.range === 'custom' ? <form className="custom-dates" onSubmit={(event) => {
      event.preventDefault()
      const invalid = validateDates(start, end)
      setError(invalid)
      if (!invalid) { onChange({ range: 'custom', startDate: start, endDate: end }); setCustomOpen(false) }
    }} noValidate>
      <label>开始日期<input type="date" value={start} min="1970-01-01" max={localDate()} onChange={(event) => setStart(event.target.value)} /></label>
      <label>结束日期<input type="date" value={end} min="1970-01-01" max={localDate()} onChange={(event) => setEnd(event.target.value)} /></label>
      <button type="submit" className="subtle-button">应用日期</button>
      {error ? <p className="date-error" role="alert">{error}</p> : null}
    </form> : null}
  </div>
}
