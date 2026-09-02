import { useMemo } from 'react'
import type { ShortcutCount } from '../lib/contracts'
import { getShortcutDefinition } from '../lib/shortcuts'

type ShortcutChartProps = {
  entries: readonly ShortcutCount[]
  totalShortcuts: number
}

const numberFormatter = new Intl.NumberFormat('zh-CN')

export function ShortcutChart({ entries, totalShortcuts }: ShortcutChartProps) {
  const rankedEntries = useMemo(
    () => [...entries]
      .sort((left, right) => right.pressCount - left.pressCount)
      .flatMap((entry) => {
        const shortcut = getShortcutDefinition(entry.shortcutId)
        return shortcut ? [{ ...entry, ...shortcut }] : []
      }),
    [entries],
  )
  const maxCount = useMemo(
    () => Math.max(1, ...rankedEntries.map(({ pressCount }) => pressCount)),
    [rankedEntries],
  )
  const totalLabel = numberFormatter.format(totalShortcuts)

  return (
    <article className="chart-card shortcut-card" id="shortcuts" aria-labelledby="shortcut-ranking-title">
      <div className="chart-card__heading">
        <div>
          <p className="eyebrow">固定目录汇总</p>
          <h2 id="shortcut-ranking-title">常用快捷键</h2>
        </div>
        <span className="chart-card__note">共 {totalLabel} 次</span>
      </div>

      <p className="shortcut-card__summary">
        只统计固定目录中的组合总次数；不写入按键顺序或输入内容，也不叠加到键帽颜色。
      </p>
      <p className="sr-only">
        当前范围共识别 {totalLabel} 次固定目录中的快捷键；以下按使用次数从高到低排列。
      </p>

      {rankedEntries.length > 0 ? (
        <ol className="shortcut-list" aria-label="常用快捷键排行">
          {rankedEntries.map(({ shortcutId, keys, label, pressCount }, index) => {
            const percentage = Math.max(3, (pressCount / maxCount) * 100)

            return (
              <li className="shortcut-list__item" key={shortcutId}>
                <span className="shortcut-list__position">{String(index + 1).padStart(2, '0')}</span>
                <span className="shortcut-list__identity" aria-label={`${keys}：${label}`}>
                  <kbd className="shortcut-list__label">{keys}</kbd>
                  <span className="shortcut-list__meaning">{label}</span>
                </span>
                <span className="shortcut-list__track" aria-hidden="true">
                  <span className="shortcut-list__bar" style={{ width: `${percentage}%` }} />
                </span>
                <span className="shortcut-list__count">{numberFormatter.format(pressCount)} 次</span>
              </li>
            )
          })}
        </ol>
      ) : (
        <p className="shortcut-card__empty">当前范围还没有识别到固定目录中的快捷键。</p>
      )}
    </article>
  )
}
