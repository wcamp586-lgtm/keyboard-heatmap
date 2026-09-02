import { describe, expect, it } from 'vitest'
import {
  getShortcutDefinition,
  getShortcutLabel,
  isRecognizedShortcutId,
  RECOGNIZED_SHORTCUTS,
} from './shortcuts'

describe('recognized shortcut registry', () => {
  it('contains only the fixed Ctrl, Alt, and Win combinations used by the dashboard', () => {
    expect(RECOGNIZED_SHORTCUTS).toHaveLength(85)
    expect(RECOGNIZED_SHORTCUTS.map(({ id }) => id)).toContain('ctrl_c')
    expect(RECOGNIZED_SHORTCUTS.map(({ id }) => id)).toContain('ctrl_shift_z')
    expect(RECOGNIZED_SHORTCUTS.map(({ id }) => id)).toContain('alt_tab')
    expect(RECOGNIZED_SHORTCUTS.map(({ id }) => id)).toContain('alt_f4')
    expect(RECOGNIZED_SHORTCUTS.map(({ id }) => id)).toContain('win_d')
    expect(RECOGNIZED_SHORTCUTS.map(({ id }) => id)).toContain('win_ctrl_f4')
    expect(RECOGNIZED_SHORTCUTS.map(({ id }) => id)).not.toContain('key_sequence')
  })

  it('uses Chinese action labels while normalizing fixed display aliases', () => {
    expect(getShortcutLabel('Ctrl + C')).toBe('Ctrl + C · 复制')
    expect(getShortcutLabel('ctrl-shift-z')).toBe('Ctrl + Shift + Z · 重做')
    expect(getShortcutDefinition('win-d')).toEqual({
      id: 'win_d',
      keys: 'Win + D',
      label: '显示桌面',
    })
    expect(getShortcutLabel('alt_f4')).toBe('Alt + F4 · 关闭窗口')
    expect(isRecognizedShortcutId('ctrl_v')).toBe(true)
    expect(isRecognizedShortcutId('win_d')).toBe(true)
    expect(isRecognizedShortcutId('alt_tab')).toBe(true)
    expect(isRecognizedShortcutId('ctrl_q')).toBe(false)
    expect(getShortcutLabel('ctrl_q')).toBeUndefined()
  })
})
