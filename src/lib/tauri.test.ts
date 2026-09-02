import { describe, expect, it } from 'vitest'
import { toShortcutCounts } from './tauri'

describe('shortcut aggregate parser', () => {
  it('accepts only fixed shortcut aggregate rows and safely defaults to empty', () => {
    expect(toShortcutCounts(undefined)).toEqual([])
    expect(toShortcutCounts([
      { shortcutId: 'Ctrl + C', count: 12.2 },
      { shortcutId: 'ctrl_shift_z', count: 3 },
      { shortcutId: 'Win + D', count: 7 },
      { shortcutId: 'alt-f4', count: 5 },
      { shortcutId: 'ctrl_q', count: 99 },
      { shortcutId: 'typed_secret', count: 4 },
      { shortcutId: 'ctrl_v', count: -1 },
    ])).toEqual([
      { shortcutId: 'ctrl_c', pressCount: 12 },
      { shortcutId: 'ctrl_shift_z', pressCount: 3 },
      { shortcutId: 'win_d', pressCount: 7 },
      { shortcutId: 'alt_f4', pressCount: 5 },
    ])
  })
})
