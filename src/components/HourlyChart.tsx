import { useMemo } from 'react'
import type { HourlyCount } from '../lib/contracts'

type HourlyChartProps = {
  entries: readonly HourlyCount[]
}

const numberFormatter = new Intl.NumberFormat('zh-CN')

export function HourlyChart({ entries }: HourlyChartProps) {
  const hours = useMemo(() => {
    const byHour = new Map(entries.map(({ hour, pressCount }) => [hour, pressCount]))
    return Array.from({ length: 24 }, (_, hour) => ({ hour, pressCount: byHour.get(hour) ?? 0 }))
  }, [entries])
  const maxCount = useMemo(
    () => Math.max(1, ...hours.map(({ pressCount }) => pressCount)),
    [hours],
  )

  return (
    <article className="chart-card hourly-card" id="activity">
      <div className="chart-card__heading">
        <div>
          <p className="eyebrow">活跃节奏</p>
          <h2>每小时按键数</h2>
        </div>
        <span className="chart-card__note">00–23 时</span>
      </div>

      <div className="hourly-chart" role="list" aria-label="按小时统计">
        {hours.map(({ hour, pressCount }) => {
          const height = Math.max(3, (pressCount / maxCount) * 100)
          const label = String(hour).padStart(2, '0')

          return (
            <div className="hourly-chart__column" role="listitem" key={hour}>
              <span className="sr-only">{label} 时：{numberFormatter.format(pressCount)} 次按下</span>
              <span
                className="hourly-chart__bar"
                style={{ height: `${height}%` }}
                aria-hidden="true"
                title={`${label}:00 · ${numberFormatter.format(pressCount)} 次`}
              />
              {hour % 3 === 0 ? <span className="hourly-chart__label">{label}</span> : null}
            </div>
          )
        })}
      </div>
    </article>
  )
}
