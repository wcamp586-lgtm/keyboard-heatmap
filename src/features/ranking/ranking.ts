import type { DashboardSnapshot } from '../../lib/contracts'
import { ALL_KEYS, ARROW_KEYS, FUNCTION_GROUPS, getKeyLabel, KEY_WIRE_IDS, NAVIGATION_KEYS, NUMPAD_KEYS } from '../../lib/keyboard-layout'
import { RECOGNIZED_SHORTCUTS } from '../../lib/shortcuts'

export const ZONES = ['主键区', '功能区', '编辑导航区', '方向区', '数字小键盘'] as const
export type RankingKind = 'key' | 'shortcut'
export type RankingSort = 'count-desc' | 'count-asc' | 'name'
export type RankingState = { kind: RankingKind; search: string; zone: string; sort: RankingSort; showZero: boolean; page: number }
export const initialRanking: RankingState = { kind: 'key', search: '', zone: '', sort: 'count-desc', showZero: false, page: 0 }
export type RankingRow = { id: string; order: number; label: string; detail: string; count: number; share: number; rank: number }

const zones = new Map<number, string>([
  ...FUNCTION_GROUPS.flat().map(({ id }): [number, string] => [id, '功能区']),
  ...NAVIGATION_KEYS.map(({ id }): [number, string] => [id, '编辑导航区']),
  ...ARROW_KEYS.map(({ id }): [number, string] => [id, '方向区']),
  ...NUMPAD_KEYS.map(({ id }): [number, string] => [id, '数字小键盘']),
])
const detailedLabels: Record<number, string> = {
  74: '左 Shift', 85: '右 Shift', 91: '左 Ctrl', 98: '右 Ctrl',
  92: '左 Win', 96: '右 Win', 93: '左 Alt', 95: '右 Alt', 94: 'Space 空格', 70: 'Enter 主回车',
  99: '← 左方向', 100: '↓ 下方向', 101: '→ 右方向', 86: '↑ 上方向',
}
export const getDetailedKeyLabel = (id: number) => detailedLabels[id]
  ?? (zones.get(id) === '数字小键盘' ? `小键盘 ${getKeyLabel(id)}` : getKeyLabel(id))

export function rankingRows(snapshot: DashboardSnapshot, kind: RankingKind): RankingRow[] {
  const total = kind === 'key' ? snapshot.totalPresses : snapshot.totalShortcuts
  const keyCounts = new Map(snapshot.heatmap.map(({ keyId, pressCount }) => [keyId, pressCount]))
  const shortcutCounts = new Map(snapshot.shortcutCounts.map(({ shortcutId, pressCount }) => [shortcutId, pressCount]))
  const rows = kind === 'key' ? ALL_KEYS.map(({ id }) => ({
    id: KEY_WIRE_IDS[id], order: id, label: getDetailedKeyLabel(id), detail: zones.get(id) ?? '主键区', count: keyCounts.get(id) ?? 0,
  })) : RECOGNIZED_SHORTCUTS.map(({ id, keys, label }, order) => ({
    id, order, label: keys, detail: label, count: shortcutCounts.get(id) ?? 0,
  }))
  return rows.sort((a, b) => b.count - a.count || a.order - b.order)
    .map((row, index) => ({ ...row, rank: index + 1, share: total > 0 ? row.count / total * 100 : 0 }))
}

export function filterRanking(rows: RankingRow[], state: RankingState) {
  const search = state.search.trim().toLocaleLowerCase()
  return rows.filter((row) => (state.showZero || row.count > 0)
    && (state.kind === 'shortcut' || !state.zone || row.detail === state.zone)
    && `${row.label} ${row.detail} ${row.id}`.toLocaleLowerCase().includes(search))
    .sort((a, b) => (state.sort === 'name' ? a.label.localeCompare(b.label, 'zh-CN')
      : state.sort === 'count-asc' ? a.count - b.count : b.count - a.count) || a.order - b.order)
}
