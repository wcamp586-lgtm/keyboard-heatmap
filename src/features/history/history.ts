import type { HistoryTarget } from '../../lib/contracts'
import { getKeyIdFromWireId } from '../../lib/keyboard-layout'
import { getShortcutDefinition } from '../../lib/shortcuts'
import { getDetailedKeyLabel } from '../ranking/ranking'

export type HistoryState = { target: HistoryTarget | null; page: number; expanded: string | null; metric: 'key' | 'shortcut' }
export const initialHistory: HistoryState = { target: null, page: 0, expanded: null, metric: 'key' }
export const targetKey = (target: HistoryTarget | null) => target ? `${target.kind}:${target.id}` : 'all'
export const targetLabel = (target: HistoryTarget) => target.kind === 'key'
  ? getDetailedKeyLabel(getKeyIdFromWireId(target.id) ?? -1)
  : getShortcutDefinition(target.id)?.keys ?? '未知快捷键'
