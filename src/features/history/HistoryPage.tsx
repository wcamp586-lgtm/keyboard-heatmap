import { Fragment, useCallback, useId } from 'react'
import { Pagination } from '../../components/Pagination'
import type { DailyRecord, DateSelection, HistoryTarget } from '../../lib/contracts'
import { localDate, selectionKey } from '../../lib/dates'
import { formatNumber } from '../../lib/format'
import { getDayDetail, getHistory, nativeBridgeAvailable } from '../../lib/tauri'
import { useLocalQuery } from '../dashboard/useLocalQuery'
import { getDetailedKeyLabel } from '../ranking/ranking'
import { targetKey, targetLabel, type HistoryState } from './history'

function TrendChart({ days, metric }: { days: DailyRecord[]; metric: 'key' | 'shortcut' }) {
  const titleId = useId()
  const ordered = [...days].reverse()
  const values = ordered.map((day) => metric === 'key' ? day.totalPresses : day.totalShortcuts)
  const max = Math.max(1, ...values)
  const points = values.map((value, index) => ({ x: values.length === 1 ? 490 : 60 + index / (values.length - 1) * 890, y: 180 - value / max * 146, value }))
  const line = points.map((point) => `${point.x},${point.y}`).join(' ')
  return <figure className="history-trend">
    <figcaption><span>本页每日{metric === 'key' ? '按键' : '快捷键'}趋势</span><strong>{formatNumber(values.reduce((sum, value) => sum + value, 0))}<small> 次 / 本页</small></strong></figcaption>
    <svg viewBox="0 0 980 220" role="img" aria-labelledby={titleId}>
      <title id={titleId}>本页 {ordered[0]?.date} 至 {ordered.at(-1)?.date} 每日统计，具体数值见下方记录表。</title>
      {[0, 0.5, 1].map((part) => <g key={part}><line x1="60" x2="950" y1={180 - part * 146} y2={180 - part * 146} className="trend-grid" /><text className="trend-axis-y" x="48" y={185 - part * 146} textAnchor="end">{formatNumber(Math.round(max * part))}</text></g>)}
      {points.length > 1 ? <polygon points={`60,180 ${line} 950,180`} className="trend-area" /> : null}
      <polyline points={line} className="trend-line" />
      {points.map((point, index) => <circle key={ordered[index].date} cx={point.x} cy={point.y} r="3.5" className="trend-point"><title>{ordered[index].date}：{formatNumber(point.value)} 次</title></circle>)}
      <text x="60" y="209">{ordered[0]?.date}</text><text x="950" y="209" textAnchor="end">{ordered.at(-1)?.date}</text>
    </svg>
  </figure>
}

function HourDetails({ date, target, revision }: { date: string; target: HistoryTarget | null; revision: number }) {
  const load = useCallback(() => getDayDetail(date, target), [date, target])
  const query = useLocalQuery(`${date}:${targetKey(target)}:${revision}`, load, date === localDate() && nativeBridgeAvailable() ? 3000 : null)
  return <section className="hour-details" aria-label={`${date} 小时记录`}>
    <div className="statistics-heading"><div><h3>{date} · 小时明细</h3><p className="table-note">按本机时区展示 UTC 小时桶；标注偏移以区分重复小时。今天仅列出已到达的小时。</p></div>
      <button type="button" className="subtle-button" onClick={query.refresh}>刷新小时记录</button></div>
    {query.error ? <p role="alert" className="error-notice">小时记录读取失败。{query.data ? '下方保留上次结果，可能已过期。' : ''}<button type="button" className="text-button" onClick={query.refresh}>重试</button></p> : null}
    {query.isLoading ? <p role="status">正在读取小时记录…</p> : null}
    {query.data ? <table className="data-table hour-table"><caption className="sr-only">{date} 小时汇总</caption><thead><tr><th scope="col">本地时间（UTC 偏移）</th>
      {target?.kind !== 'shortcut' ? <th scope="col" className="numeric">按键次数</th> : null}
      {target?.kind !== 'key' ? <th scope="col" className="numeric">快捷键次数</th> : null}
      {!target ? <th scope="col">最常用按键</th> : null}
      <th scope="col">记录状态</th></tr></thead><tbody>{query.data.hours.map((hour) => <tr key={hour.hourStartUtc}>
        <th scope="row">{hour.label} <span className="muted">UTC{hour.utcOffset}</span></th>
        {target?.kind !== 'shortcut' ? <td className="numeric">{formatNumber(hour.totalPresses)}</td> : null}
        {target?.kind !== 'key' ? <td className="numeric">{formatNumber(hour.totalShortcuts)}</td> : null}
        {!target ? <td>{hour.topKey ? `${getDetailedKeyLabel(hour.topKey.keyId)} · ${formatNumber(hour.topKey.pressCount)} 次` : '—'}</td> : null}
        <td className="muted">{hour.totalPresses || hour.totalShortcuts ? '已保存汇总' : '无已保存记录'}</td>
      </tr>)}</tbody></table> : null}
    {query.data?.hours.length === 0 ? <p className="table-note">这个日期还没有可展示的小时桶。</p> : null}
  </section>
}

export function HistoryPage({ selection, state, onChange, revision, onRanking }: {
  selection: DateSelection; state: HistoryState; onChange: (next: HistoryState) => void; revision: number; onRanking: () => void
}) {
  const load = useCallback(() => getHistory(selection, state.target, state.page), [selection, state.target, state.page])
  const query = useLocalQuery(`${selectionKey(selection)}:${targetKey(state.target)}:${state.page}:${revision}`, load,
    selection.range === 'today' && nativeBridgeAvailable() ? 3000 : null)
  const metric = state.target?.kind ?? state.metric
  const columnCount = state.target?.kind === 'shortcut' ? 3 : state.target ? 4 : 7
  return <section className="statistics-panel panel" aria-labelledby="history-title">
    <div className="statistics-heading"><div><p className="eyebrow">按天回顾，按小时展开</p><h2 id="history-title">{state.target ? `${targetLabel(state.target)} 的使用记录` : '使用记录'}</h2></div>
      <div className="history-actions">{state.target ? <><button type="button" className="subtle-button" onClick={onRanking}>返回排行</button><button type="button" className="subtle-button" onClick={() => onChange({ ...state, target: null, page: 0, expanded: null })}>查看全部记录</button></> : null}
        <button type="button" className="subtle-button" onClick={query.refresh}>刷新记录</button></div>
    </div>
    <p className="table-note">只展示已保存的小时聚合。零次数表示“无已保存记录”，不代表当天没打字或应用一直运行。</p>
    {query.error ? <p role="alert" className="error-notice">记录读取失败。{query.data ? '下方保留上次结果，可能已过期。' : ''}<button type="button" className="text-button" onClick={query.refresh}>重试</button></p> : null}
    {query.isLoading ? <div className="empty-state" role="status">正在读取历史记录…</div> : null}
    {query.data ? <>
      <div className="history-range"><span>{query.data.rangeStartDate} — {query.data.rangeEndDate} <span className="muted">· 本机日历日期</span></span>
        {!state.target ? <div className="segmented" aria-label="趋势指标"><button type="button" aria-pressed={metric === 'key'} onClick={() => onChange({ ...state, metric: 'key' })}>按键</button><button type="button" aria-pressed={metric === 'shortcut'} onClick={() => onChange({ ...state, metric: 'shortcut' })}>快捷键</button></div> : null}
      </div>
      <TrendChart days={query.data.days} metric={metric} />
      <p className="table-note">每天一行，新日期在前 · 每页最多 30 个日历日 · 图表按时间从左到右排列</p>
      <p className="table-scroll-hint">← 左右滑动表格，查看更多字段及小时明细 →</p>
      <div className="table-scroll" tabIndex={0} role="region" aria-label="每日记录表，可横向滚动">
        <table className="data-table history-table"><caption className="sr-only">每日使用记录</caption><thead><tr><th scope="col">日期</th>
          {state.target?.kind !== 'shortcut' ? <th scope="col" className="numeric">按键次数</th> : null}
          {state.target?.kind !== 'key' ? <th scope="col" className="numeric">快捷键次数</th> : null}
          {!state.target ? <><th scope="col" className="numeric">使用键数</th><th scope="col">最常用按键</th></> : null}
          {state.target?.kind !== 'shortcut' ? <th scope="col">按键峰值小时</th> : null}<th scope="col"><span className="sr-only">操作</span></th></tr></thead>
          <tbody>{query.data.days.map((day) => <Fragment key={day.date}><tr className={state.expanded === day.date ? 'is-expanded' : ''}>
            <th scope="row">{day.date}{!day.totalPresses && !day.totalShortcuts ? <small className="record-empty">无已保存记录</small> : null}</th>
            {state.target?.kind !== 'shortcut' ? <td className="numeric count-cell">{formatNumber(day.totalPresses)}</td> : null}
            {state.target?.kind !== 'key' ? <td className="numeric">{formatNumber(day.totalShortcuts)}</td> : null}
            {!state.target ? <><td className="numeric">{day.uniqueKeys}</td><td>{day.topKey ? <><span>{getDetailedKeyLabel(day.topKey.keyId)}</span><small className="record-empty">{formatNumber(day.topKey.pressCount)} 次</small></> : '—'}</td></> : null}
            {state.target?.kind !== 'shortcut' ? <td className="muted">{day.peakHour ?? '—'}</td> : null}
            <td><button type="button" className="text-button" aria-label={`${state.expanded === day.date ? '收起' : '展开'} ${day.date} 小时记录`} aria-expanded={state.expanded === day.date} aria-controls={`hours-${day.date}`} onClick={() => onChange({ ...state, expanded: state.expanded === day.date ? null : day.date })}>{state.expanded === day.date ? '收起 −' : '按小时 +'}</button></td>
          </tr>{state.expanded === day.date ? <tr><td colSpan={columnCount} className="detail-cell" id={`hours-${day.date}`}><HourDetails date={day.date} target={state.target} revision={revision} /></td></tr> : null}</Fragment>)}</tbody>
        </table>
      </div>
      <Pagination page={query.data.page} totalPages={query.data.totalPages} label="记录分页" onChange={(page) => onChange({ ...state, page, expanded: null })} />
    </> : null}
  </section>
}
