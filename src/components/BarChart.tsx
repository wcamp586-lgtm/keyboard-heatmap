import { useMemo } from 'react'
import type { KeyCount } from '../lib/contracts'
import { formatNumber } from '../lib/format'
import { getKeyLabel } from '../lib/keyboard-layout'

type BarChartProps = {
  entries: readonly KeyCount[]
}

export function BarChart({ entries }: BarChartProps) {
  const maxCount = useMemo(
    () => Math.max(1, ...entries.map(({ pressCount }) => pressCount)),
    [entries],
  )

  return (
    <article className="chart-card rank-card" id="ranking">
      <div className="chart-card__heading">
        <div>
          <p className="eyebrow">按键使用排行</p>
          <h2>最常用的 10 个键</h2>
        </div>
        <span className="chart-card__note">按下次数</span>
      </div>

      <ol className="rank-list">
        {entries.map((entry, index) => {
          const percentage = Math.max(2, (entry.pressCount / maxCount) * 100)
          const label = getKeyLabel(entry.keyId)

          return (
            <li className="rank-list__item" key={entry.keyId}>
              <span className="rank-list__position">{String(index + 1).padStart(2, '0')}</span>
              <span className="rank-list__label">{label}</span>
              <span className="rank-list__track" aria-label={`${label} ${formatNumber(entry.pressCount)} 次`}>
                <span className="rank-list__bar" style={{ width: `${percentage}%` }} />
              </span>
              <span className="rank-list__count">{formatNumber(entry.pressCount)}</span>
            </li>
          )
        })}
      </ol>
    </article>
  )
}
