import { describe, expect, it } from 'vitest'
import {
  ALL_KEYS,
  getKeyIdFromWireId,
  heatLevel,
  KEY_BY_ID,
  KEY_WIRE_IDS,
  keyLabelFor,
  MAC_HIDDEN_KEY_IDS,
  visibleKeyCount,
} from './keyboard-layout'

describe('fixed ANSI 104-key layout', () => {
  it('contains every persisted KeyId exactly once', () => {
    const sortedIds = [...KEY_BY_ID.keys()].sort((left, right) => left - right)

    expect(ALL_KEYS).toHaveLength(104)
    expect(KEY_BY_ID.size).toBe(104)
    expect(sortedIds).toEqual(Array.from({ length: 104 }, (_, index) => index))
    expect(KEY_WIRE_IDS).toHaveLength(104)
  })

  it('matches the Rust wire ids for split navigation, arrows, and numpad keys', () => {
    expect(getKeyIdFromWireId('key_w')).toBe(39)
    expect(getKeyIdFromWireId('enter')).toBe(70)
    expect(getKeyIdFromWireId('numpad_4')).toBe(71)
    expect(getKeyIdFromWireId('arrow_up')).toBe(86)
    expect(getKeyIdFromWireId('numpad_enter')).toBe(90)
    expect(getKeyIdFromWireId('control_left')).toBe(91)
    expect(getKeyIdFromWireId('space')).toBe(94)
    expect(getKeyIdFromWireId('numpad_0')).toBe(102)
    expect(getKeyIdFromWireId('numpad_decimal')).toBe(103)
  })

  it('uses five stable heat states without treating zero as activity', () => {
    expect(heatLevel(0, 900)).toBe(0)
    expect(heatLevel(1, 900)).toBe(1)
    expect(heatLevel(120, 900)).toBe(2)
    expect(heatLevel(400, 900)).toBe(3)
    expect(heatLevel(900, 900)).toBe(4)
  })
})

describe('macOS layout adaptation', () => {
  it('hides the five Windows-only positions and counts the rest', () => {
    expect([...MAC_HIDDEN_KEY_IDS].sort((left, right) => left - right)).toEqual([13, 14, 15, 30, 97])
    expect(visibleKeyCount(true)).toBe(99)
    expect(visibleKeyCount(false)).toBe(104)
  })

  it('uses Apple key-cap labels for modifiers and editing keys', () => {
    expect(keyLabelFor(91, true)).toBe('⌃')
    expect(keyLabelFor(92, true)).toBe('⌘')
    expect(keyLabelFor(93, true)).toBe('⌥')
    expect(keyLabelFor(29, true)).toBe('Delete')
    expect(keyLabelFor(33, true)).toBe('Clear')
    expect(keyLabelFor(51, true)).toBe('⌦')
    expect(keyLabelFor(91, false)).toBe('Ctrl')
    expect(keyLabelFor(29, false)).toBe('Backspace')
  })
})
