import { useMemo } from 'react'
import { Pagination } from '../../components/Pagination'
import type { DashboardSnapshot, HistoryTarget } from '../../lib/contracts'
import { formatNumber } from '../../lib/format'
import { filterRanking, rankingRows, ZONES, type RankingSort, type RankingState } from './ranking'

export function RankingPage({ snapshot, state, onChange, onHistory }: {
  snapshot: DashboardSnapshot; state: RankingState; onChange: (next: RankingState) => void; onHistory: (target: HistoryTarget) => void
}) {
  const rows = useMemo(() => rankingRows(snapshot, state.kind), [snapshot, state.kind])
  const filtered = useMemo(() => filterRanking(rows, state), [rows, state])
  const totalPages = Math.max(1, Math.ceil(filtered.length / 20))
  const page = Math.min(state.page, totalPages - 1)
  const update = (patch: Partial<RankingState>) => onChange({ ...state, ...patch, page: 0 })
  return <section className="statistics-panel panel" aria-labelledby="ranking-title">
    <div className="statistics-heading"><div><p className="eyebrow">不止前十名</p><h2 id="ranking-title">完整使用排行</h2></div>
      <div className="segmented" aria-label="排行类型">
        <button type="button" aria-pressed={state.kind === 'key'} onClick={() => update({ kind: 'key', search: '' })}>物理按键 · 104</button>
        <button type="button" aria-pressed={state.kind === 'shortcut'} onClick={() => update({ kind: 'shortcut', search: '' })}>内置快捷键 · 85</button>
      </div>
    </div>
    <div className="statistics-filters">
      <label className="search-field">搜索{state.kind === 'key' ? '按键' : '快捷键'}<input type="search" placeholder={state.kind === 'key' ? '名称、左右侧或键区' : '组合名称或用途，如复制'} value={state.search} onChange={(event) => update({ search: event.target.value })} /></label>
      {state.kind === 'key' ? <label>键区<select value={state.zone} onChange={(event) => update({ zone: event.target.value })}><option value="">全部键区</option>{ZONES.map((zone) => <option key={zone}>{zone}</option>)}</select></label> : null}
      <label>排序<select value={state.sort} onChange={(event) => update({ sort: event.target.value as RankingSort })}><option value="count-desc">次数从高到低</option><option value="count-asc">次数从低到高</option><option value="name">名称顺序</option></select></label>
      <label className="checkbox-field"><input type="checkbox" checked={state.showZero} onChange={(event) => update({ showZero: event.target.checked })} />显示零次项</label>
    </div>
    <p className="table-note">{filtered.length} / {rows.length} 项 · 名次与占比按当前日期范围内的全部{state.kind === 'key' ? '物理按键' : '内置快捷键'}计算，不随搜索改变。</p>
    <p className="table-scroll-hint">← 左右滑动表格，查看次数、占比与历史 →</p>
    <div className="table-scroll" tabIndex={0} role="region" aria-label="完整排行表，可横向滚动">
      <table className="data-table ranking-table"><caption className="sr-only">完整使用排行</caption><thead><tr><th scope="col">名次</th><th scope="col">{state.kind === 'key' ? '按键' : '快捷键'}</th><th scope="col">{state.kind === 'key' ? '键区' : '常见用途'}</th><th scope="col" className="numeric">次数</th><th scope="col" className="numeric">占比</th><th scope="col"><span className="sr-only">操作</span></th></tr></thead>
        <tbody>{filtered.slice(page * 20, (page + 1) * 20).map((row) => <tr key={row.id}>
          <td className="rank-number">{row.count ? String(row.rank).padStart(2, '0') : '—'}</td><th scope="row"><kbd>{row.label}</kbd></th><td className="muted">{row.detail}</td>
          <td className="numeric count-cell">{formatNumber(row.count)}</td><td className="numeric"><div className="share-cell"><span>{row.share.toFixed(2)}%</span><i aria-hidden="true" style={{ width: `${row.share}%` }} /></div></td>
          <td><button type="button" className="text-button" aria-label={`查看 ${row.label} 的历史`} onClick={() => onHistory({ kind: state.kind, id: row.id })}>查看历史 ↗</button></td>
        </tr>)}</tbody>
      </table>
    </div>
    {filtered.length === 0 ? <div className="empty-state"><h3>没有匹配的统计项</h3><p>{state.showZero ? '试试其他名称，或清除筛选条件。' : '可打开“显示零次项”，或清除筛选条件。'}</p><button type="button" className="subtle-button" onClick={() => update({ search: '', zone: '', showZero: true })}>清除筛选并显示全部</button></div> : null}
    <Pagination page={page} totalPages={totalPages} onChange={(next) => onChange({ ...state, page: next })} label="排行分页" />
    {state.kind === 'shortcut' ? <p className="table-note">仅识别 85 个内置组合；用途为常见说明，不代表实际运行的应用或命令。</p> : null}
  </section>
}
