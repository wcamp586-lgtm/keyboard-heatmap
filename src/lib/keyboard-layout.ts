export type KeyboardKey = {
  id: number
  label: string
  width?: number
  column?: string
  row?: string
}

const key = (
  id: number,
  label: string,
  width = 1,
  placement: Pick<KeyboardKey, 'column' | 'row'> = {},
): KeyboardKey => ({ id, label, width, ...placement })

/**
 * Key ids mirror `src-tauri/src/keymap.rs::KeyId` exactly. The visual order is
 * physical keyboard order; persistence and IPC only ever exchange these ids.
 */
export const FUNCTION_GROUPS: readonly (readonly KeyboardKey[])[] = [
  [key(0, 'Esc')],
  [key(1, 'F1'), key(2, 'F2'), key(3, 'F3'), key(4, 'F4')],
  [key(5, 'F5'), key(6, 'F6'), key(7, 'F7'), key(8, 'F8')],
  [key(9, 'F9'), key(10, 'F10'), key(11, 'F11'), key(12, 'F12')],
  [key(13, 'PrtSc'), key(14, 'ScrLk'), key(15, 'Pause')],
]

export const MAIN_ROWS: readonly (readonly KeyboardKey[])[] = [
  [
    key(16, '`'),
    key(17, '1'),
    key(18, '2'),
    key(19, '3'),
    key(20, '4'),
    key(21, '5'),
    key(22, '6'),
    key(23, '7'),
    key(24, '8'),
    key(25, '9'),
    key(26, '0'),
    key(27, '−'),
    key(28, '='),
    key(29, 'Backspace', 2),
  ],
  [
    key(37, 'Tab', 1.5),
    key(38, 'Q'),
    key(39, 'W'),
    key(40, 'E'),
    key(41, 'R'),
    key(42, 'T'),
    key(43, 'Y'),
    key(44, 'U'),
    key(45, 'I'),
    key(46, 'O'),
    key(47, 'P'),
    key(48, '['),
    key(49, ']'),
    key(50, '\\', 1.5),
  ],
  [
    key(58, 'Caps', 1.75),
    key(59, 'A'),
    key(60, 'S'),
    key(61, 'D'),
    key(62, 'F'),
    key(63, 'G'),
    key(64, 'H'),
    key(65, 'J'),
    key(66, 'K'),
    key(67, 'L'),
    key(68, ';'),
    key(69, "'"),
    key(70, 'Enter', 2.25),
  ],
  [
    key(74, 'Shift', 2.25),
    key(75, 'Z'),
    key(76, 'X'),
    key(77, 'C'),
    key(78, 'V'),
    key(79, 'B'),
    key(80, 'N'),
    key(81, 'M'),
    key(82, ','),
    key(83, '.'),
    key(84, '/'),
    key(85, 'Shift', 2.75),
  ],
  [
    key(91, 'Ctrl', 1.25),
    key(92, 'Win', 1.25),
    key(93, 'Alt', 1.25),
    key(94, 'Space', 6.25),
    key(95, 'Alt', 1.25),
    key(96, 'Win', 1.25),
    key(97, 'Menu', 1.25),
    key(98, 'Ctrl', 1.25),
  ],
]

export const NAVIGATION_KEYS: readonly KeyboardKey[] = [
  key(30, 'Insert'),
  key(31, 'Home'),
  key(32, 'PgUp'),
  key(51, 'Delete'),
  key(52, 'End'),
  key(53, 'PgDn'),
]

export const ARROW_KEYS: readonly KeyboardKey[] = [
  key(86, '↑', 1, { column: '2', row: '1' }),
  key(99, '←', 1, { column: '1', row: '2' }),
  key(100, '↓', 1, { column: '2', row: '2' }),
  key(101, '→', 1, { column: '3', row: '2' }),
]

export const NUMPAD_KEYS: readonly KeyboardKey[] = [
  key(33, 'Num', 1, { column: '1', row: '1' }),
  key(34, '/', 1, { column: '2', row: '1' }),
  key(35, '*', 1, { column: '3', row: '1' }),
  key(36, '−', 1, { column: '4', row: '1' }),
  key(54, '7', 1, { column: '1', row: '2' }),
  key(55, '8', 1, { column: '2', row: '2' }),
  key(56, '9', 1, { column: '3', row: '2' }),
  key(57, '+', 1, { column: '4', row: '2 / span 2' }),
  key(71, '4', 1, { column: '1', row: '3' }),
  key(72, '5', 1, { column: '2', row: '3' }),
  key(73, '6', 1, { column: '3', row: '3' }),
  key(87, '1', 1, { column: '1', row: '4' }),
  key(88, '2', 1, { column: '2', row: '4' }),
  key(89, '3', 1, { column: '3', row: '4' }),
  key(90, 'Enter', 1, { column: '4', row: '4 / span 2' }),
  key(102, '0', 1, { column: '1 / span 2', row: '5' }),
  key(103, '.', 1, { column: '3', row: '5' }),
]

const flatten = (groups: readonly (readonly KeyboardKey[])[]) => groups.flat()

export const ALL_KEYS = [
  ...flatten(FUNCTION_GROUPS),
  ...flatten(MAIN_ROWS),
  ...NAVIGATION_KEYS,
  ...ARROW_KEYS,
  ...NUMPAD_KEYS,
] as const

/** Stable string ids returned by the Rust aggregate commands. */
export const KEY_WIRE_IDS = [
  'escape', 'f1', 'f2', 'f3', 'f4', 'f5', 'f6', 'f7', 'f8', 'f9', 'f10', 'f11', 'f12',
  'print_screen', 'scroll_lock', 'pause', 'grave', 'digit_1', 'digit_2', 'digit_3', 'digit_4',
  'digit_5', 'digit_6', 'digit_7', 'digit_8', 'digit_9', 'digit_0', 'minus', 'equal',
  'backspace', 'insert', 'home', 'page_up', 'num_lock', 'numpad_divide', 'numpad_multiply',
  'numpad_subtract', 'tab', 'key_q', 'key_w', 'key_e', 'key_r', 'key_t', 'key_y', 'key_u',
  'key_i', 'key_o', 'key_p', 'bracket_left', 'bracket_right', 'backslash', 'delete', 'end',
  'page_down', 'numpad_7', 'numpad_8', 'numpad_9', 'numpad_add', 'caps_lock', 'key_a', 'key_s',
  'key_d', 'key_f', 'key_g', 'key_h', 'key_j', 'key_k', 'key_l', 'semicolon', 'quote', 'enter',
  'numpad_4', 'numpad_5', 'numpad_6', 'shift_left', 'key_z', 'key_x', 'key_c', 'key_v', 'key_b',
  'key_n', 'key_m', 'comma', 'period', 'slash', 'shift_right', 'arrow_up', 'numpad_1',
  'numpad_2', 'numpad_3', 'numpad_enter', 'control_left', 'meta_left', 'alt_left', 'space',
  'alt_right', 'meta_right', 'context_menu', 'control_right', 'arrow_left', 'arrow_down',
  'arrow_right', 'numpad_0', 'numpad_decimal',
] as const

export const KEY_BY_ID = new Map(ALL_KEYS.map((keyboardKey) => [keyboardKey.id, keyboardKey]))
export const KEY_ID_BY_WIRE_ID = new Map<string, number>(
  KEY_WIRE_IDS.map((wireId, index): [string, number] => [wireId, index]),
)

if (ALL_KEYS.length !== 104 || KEY_BY_ID.size !== 104 || KEY_WIRE_IDS.length !== 104) {
  throw new Error('The ANSI 104-key layout must contain 104 unique KeyIds.')
}

export const getKeyLabel = (keyId: number) => KEY_BY_ID.get(keyId)?.label ?? 'Unknown'
export const getKeyIdFromWireId = (wireId: string) => KEY_ID_BY_WIRE_ID.get(wireId)

export const heatLevel = (count: number, maxCount: number) => {
  if (count <= 0 || maxCount <= 0) return 0

  const ratio = count / maxCount
  if (ratio < 0.12) return 1
  if (ratio < 0.32) return 2
  if (ratio < 0.62) return 3
  return 4
}

// Positions that do not exist on an Apple keyboard, plus the Windows menu key.
// The Rust collector still has a `KeyId` for each of them, so they are hidden
// in the UI rather than removed from the persisted catalog.
export const MAC_HIDDEN_KEY_IDS = new Set([13, 14, 15, 30, 97])

// Apple key caps and names for the positions that share a `KeyId` with the
// Windows layout.
const MAC_KEY_LABELS: Record<number, string> = {
  29: 'Delete',
  33: 'Clear',
  51: '⌦',
  91: '⌃',
  98: '⌃',
  92: '⌘',
  96: '⌘',
  93: '⌥',
  95: '⌥',
}

export const keyLabelFor = (keyId: number, isMac: boolean): string =>
  (isMac ? MAC_KEY_LABELS[keyId] : undefined) ?? getKeyLabel(keyId)

export const visibleKeyCount = (isMac: boolean): number =>
  isMac ? ALL_KEYS.length - MAC_HIDDEN_KEY_IDS.size : ALL_KEYS.length
