import { useMemo } from 'react'
import type { CSSProperties } from 'react'
import type { KeyCount } from '../lib/contracts'
import {
  ARROW_KEYS,
  FUNCTION_GROUPS,
  heatLevel,
  MAIN_ROWS,
  NAVIGATION_KEYS,
  NUMPAD_KEYS,
} from '../lib/keyboard-layout'
import type { KeyboardKey } from '../lib/keyboard-layout'

type KeyboardHeatmapProps = {
  counts: readonly KeyCount[]
  selectedKeyId: number | null
  onSelect: (keyId: number) => void
}

const formatCount = (count: number) => new Intl.NumberFormat('zh-CN').format(count)

type HeatKeyProps = {
  definition: KeyboardKey
  count: number
  maxCount: number
  selected: boolean
  onSelect: (keyId: number) => void
}

function HeatKey({ definition, count, maxCount, selected, onSelect }: HeatKeyProps) {
  const level = heatLevel(count, maxCount)
  const style: CSSProperties = definition.column
    ? { gridColumn: definition.column, gridRow: definition.row }
    : { flexGrow: definition.width ?? 1 }
  const compactLabel = definition.label.length > 5

  return (
    <button
      type="button"
      className={`heat-key heat-key--${level}${selected ? ' is-selected' : ''}`}
      style={style}
      aria-label={`${definition.label}，${formatCount(count)} 次按下`}
      aria-pressed={selected}
      onClick={() => onSelect(definition.id)}
    >
      <span className={`heat-key__label${compactLabel ? ' heat-key__label--compact' : ''}`}>
        {definition.label}
      </span>
      <span className="heat-key__count" aria-hidden="true">
        {count > 0 ? formatCount(count) : '—'}
      </span>
      <span className="heat-key__tooltip" role="tooltip">
        {formatCount(count)} 次按下
      </span>
    </button>
  )
}

export function KeyboardHeatmap({ counts, selectedKeyId, onSelect }: KeyboardHeatmapProps) {
  const countById = useMemo(() => new Map(counts.map(({ keyId, pressCount }) => [keyId, pressCount])), [counts])
  const maxCount = useMemo(
    () => Math.max(1, ...counts.map(({ pressCount }) => pressCount)),
    [counts],
  )
  const heatKey = (definition: KeyboardKey) => (
    <HeatKey
      key={definition.id}
      definition={definition}
      count={countById.get(definition.id) ?? 0}
      maxCount={maxCount}
      selected={selectedKeyId === definition.id}
      onSelect={onSelect}
    />
  )

  return (
    <div className="keyboard-scroll" aria-label="104 键键盘热力图">
      <div className="keyboard-heatmap">
        <div className="function-row">
          {FUNCTION_GROUPS.map((group, index) => (
            <div className="function-group" key={`function-group-${index}`}>
              {group.map(heatKey)}
            </div>
          ))}
        </div>

        <div className="keyboard-deck">
          <div className="main-keyboard" aria-label="主键区">
            {MAIN_ROWS.map((row, index) => (
              <div className="key-row" key={`main-row-${index}`}>
                {row.map(heatKey)}
              </div>
            ))}
          </div>

          <div className="navigation-pad" aria-label="导航键区">
            {NAVIGATION_KEYS.map(heatKey)}
          </div>

          <div className="arrow-pad" aria-label="方向键区">
            {ARROW_KEYS.map(heatKey)}
          </div>

          <div className="numpad" aria-label="数字键盘区">
            {NUMPAD_KEYS.map(heatKey)}
          </div>
        </div>
      </div>
    </div>
  )
}
