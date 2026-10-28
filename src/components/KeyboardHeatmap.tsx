import { useMemo } from 'react'
import type { CSSProperties } from 'react'
import type { KeyCount } from '../lib/contracts'
import {
  ARROW_KEYS,
  FUNCTION_GROUPS,
  heatLevel,
  keyLabelFor,
  MAC_HIDDEN_KEY_IDS,
  MAIN_ROWS,
  NAVIGATION_KEYS,
  NUMPAD_KEYS,
} from '../lib/keyboard-layout'
import type { KeyboardKey } from '../lib/keyboard-layout'
import { formatNumber } from '../lib/format'
import { IS_MAC } from '../lib/platform'
import { nextSelectedKeyId } from '../lib/selection'

type KeyboardHeatmapProps = {
  counts: readonly KeyCount[]
  selectedKeyId: number | null
  onSelect: (keyId: number | null) => void
}

type HeatKeyProps = {
  definition: KeyboardKey
  count: number
  maxCount: number
  selectedKeyId: number | null
  onSelect: (keyId: number | null) => void
}

function HeatKey({ definition, count, maxCount, selectedKeyId, onSelect }: HeatKeyProps) {
  const level = heatLevel(count, maxCount)
  const selected = selectedKeyId === definition.id
  const style: CSSProperties = definition.column
    ? { gridColumn: definition.column, gridRow: definition.row }
    : { flexGrow: definition.width ?? 1 }
  const label = keyLabelFor(definition.id, IS_MAC)
  const compactLabel = label.length > 5

  return (
    <button
      type="button"
      className={`heat-key heat-key--${level}${selected ? ' is-selected' : ''}`}
      style={style}
      aria-label={`${label}，${formatNumber(count)} 次按下`}
      aria-pressed={selected}
      onClick={() => onSelect(nextSelectedKeyId(selectedKeyId, definition.id))}
    >
      <span className={`heat-key__label${compactLabel ? ' heat-key__label--compact' : ''}`}>
        {label}
      </span>
      <span className="heat-key__count" aria-hidden="true">
        {count > 0 ? formatNumber(count) : '—'}
      </span>
      <span className="heat-key__tooltip" role="tooltip">
        {formatNumber(count)} 次按下
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
  const isVisible = ({ id }: KeyboardKey) => !(IS_MAC && MAC_HIDDEN_KEY_IDS.has(id))
  const functionGroups = FUNCTION_GROUPS.map((group) => group.filter(isVisible)).filter((group) => group.length > 0)
  const mainRows = MAIN_ROWS.map((row) => row.filter(isVisible))
  const navigationKeys = NAVIGATION_KEYS.filter(isVisible)
  const heatKey = (definition: KeyboardKey) => (
    <HeatKey
      key={definition.id}
      definition={definition}
      count={countById.get(definition.id) ?? 0}
      maxCount={maxCount}
      selectedKeyId={selectedKeyId}
      onSelect={onSelect}
    />
  )

  return (
    <div className="keyboard-scroll" aria-label={IS_MAC ? 'Mac 键盘热力图' : '104 键键盘热力图'}>
      <div className="keyboard-heatmap">
        <div className={`function-row${IS_MAC ? ' function-row--mac' : ''}`}>
          {functionGroups.map((group, index) => (
            <div className="function-group" key={`function-group-${index}`}>
              {group.map(heatKey)}
            </div>
          ))}
        </div>

        <div className="keyboard-deck">
          <div className="main-keyboard" aria-label="主键区">
            {mainRows.map((row, index) => (
              <div className="key-row" key={`main-row-${index}`}>
                {row.map(heatKey)}
              </div>
            ))}
          </div>

          <div className={`navigation-pad${IS_MAC ? ' navigation-pad--mac' : ''}`} aria-label="导航键区">
            {navigationKeys.map(heatKey)}
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
