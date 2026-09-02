export type ShortcutDefinition = {
  id: string
  keys: string
  label: string
}

// This is intentionally a finite, product-owned directory. The UI rejects
// every other backend string so it never turns arbitrary input into a view.
export const RECOGNIZED_SHORTCUTS = [
  { id: 'ctrl_a', keys: 'Ctrl + A', label: '全选' },
  { id: 'ctrl_c', keys: 'Ctrl + C', label: '复制' },
  { id: 'ctrl_x', keys: 'Ctrl + X', label: '剪切' },
  { id: 'ctrl_v', keys: 'Ctrl + V', label: '粘贴' },
  { id: 'ctrl_z', keys: 'Ctrl + Z', label: '撤销' },
  { id: 'ctrl_y', keys: 'Ctrl + Y', label: '重做' },
  { id: 'ctrl_s', keys: 'Ctrl + S', label: '保存' },
  { id: 'ctrl_f', keys: 'Ctrl + F', label: '查找' },
  { id: 'ctrl_p', keys: 'Ctrl + P', label: '打印' },
  { id: 'ctrl_n', keys: 'Ctrl + N', label: '新建' },
  { id: 'ctrl_o', keys: 'Ctrl + O', label: '打开' },
  { id: 'ctrl_w', keys: 'Ctrl + W', label: '关闭标签/窗口' },
  { id: 'ctrl_t', keys: 'Ctrl + T', label: '新标签' },
  { id: 'ctrl_l', keys: 'Ctrl + L', label: '地址栏' },
  { id: 'ctrl_shift_z', keys: 'Ctrl + Shift + Z', label: '重做' },
  { id: 'ctrl_shift_s', keys: 'Ctrl + Shift + S', label: '另存为' },
  { id: 'ctrl_shift_n', keys: 'Ctrl + Shift + N', label: '新窗口/无痕' },
  { id: 'ctrl_shift_t', keys: 'Ctrl + Shift + T', label: '恢复标签' },
  { id: 'ctrl_b', keys: 'Ctrl + B', label: '加粗' },
  { id: 'ctrl_i', keys: 'Ctrl + I', label: '斜体' },
  { id: 'ctrl_u', keys: 'Ctrl + U', label: '下划线' },
  { id: 'ctrl_k', keys: 'Ctrl + K', label: '链接/搜索' },
  { id: 'ctrl_h', keys: 'Ctrl + H', label: '替换/历史' },
  { id: 'ctrl_g', keys: 'Ctrl + G', label: '查找下一个' },
  { id: 'ctrl_r', keys: 'Ctrl + R', label: '刷新' },
  { id: 'ctrl_d', keys: 'Ctrl + D', label: '收藏' },
  { id: 'ctrl_j', keys: 'Ctrl + J', label: '下载' },
  { id: 'ctrl_tab', keys: 'Ctrl + Tab', label: '下个标签' },
  { id: 'ctrl_shift_tab', keys: 'Ctrl + Shift + Tab', label: '上个标签' },
  { id: 'ctrl_shift_c', keys: 'Ctrl + Shift + C', label: '开发者工具' },
  { id: 'ctrl_shift_v', keys: 'Ctrl + Shift + V', label: '纯文本粘贴' },
  { id: 'ctrl_shift_p', keys: 'Ctrl + Shift + P', label: '命令/私密窗口' },
  { id: 'ctrl_shift_w', keys: 'Ctrl + Shift + W', label: '关闭窗口' },
  { id: 'ctrl_shift_b', keys: 'Ctrl + Shift + B', label: '书签栏' },
  { id: 'ctrl_shift_delete', keys: 'Ctrl + Shift + Delete', label: '清除数据' },
  { id: 'ctrl_plus', keys: 'Ctrl + +', label: '放大' },
  { id: 'ctrl_minus', keys: 'Ctrl + −', label: '缩小' },
  { id: 'ctrl_0', keys: 'Ctrl + 0', label: '重置缩放' },
  { id: 'ctrl_1', keys: 'Ctrl + 1', label: '切换到标签 1' },
  { id: 'ctrl_2', keys: 'Ctrl + 2', label: '切换到标签 2' },
  { id: 'ctrl_3', keys: 'Ctrl + 3', label: '切换到标签 3' },
  { id: 'ctrl_4', keys: 'Ctrl + 4', label: '切换到标签 4' },
  { id: 'ctrl_5', keys: 'Ctrl + 5', label: '切换到标签 5' },
  { id: 'ctrl_6', keys: 'Ctrl + 6', label: '切换到标签 6' },
  { id: 'ctrl_7', keys: 'Ctrl + 7', label: '切换到标签 7' },
  { id: 'ctrl_8', keys: 'Ctrl + 8', label: '切换到标签 8' },
  { id: 'ctrl_9', keys: 'Ctrl + 9', label: '切换到最后一个标签' },
  { id: 'alt_tab', keys: 'Alt + Tab', label: '切换窗口' },
  { id: 'alt_shift_tab', keys: 'Alt + Shift + Tab', label: '反向切换窗口' },
  { id: 'alt_f4', keys: 'Alt + F4', label: '关闭窗口' },
  { id: 'alt_space', keys: 'Alt + Space', label: '窗口菜单' },
  { id: 'alt_enter', keys: 'Alt + Enter', label: '属性/全屏' },
  { id: 'alt_left', keys: 'Alt + ←', label: '后退' },
  { id: 'alt_right', keys: 'Alt + →', label: '前进' },
  { id: 'win_d', keys: 'Win + D', label: '显示桌面' },
  { id: 'win_e', keys: 'Win + E', label: '资源管理器' },
  { id: 'win_l', keys: 'Win + L', label: '锁定电脑' },
  { id: 'win_r', keys: 'Win + R', label: '打开运行' },
  { id: 'win_i', keys: 'Win + I', label: '打开设置' },
  { id: 'win_s', keys: 'Win + S', label: '打开搜索' },
  { id: 'win_tab', keys: 'Win + Tab', label: '任务视图' },
  { id: 'win_v', keys: 'Win + V', label: '剪贴板历史' },
  { id: 'win_x', keys: 'Win + X', label: '快捷菜单' },
  { id: 'win_a', keys: 'Win + A', label: '快捷设置' },
  { id: 'win_n', keys: 'Win + N', label: '通知中心' },
  { id: 'win_p', keys: 'Win + P', label: '投影' },
  { id: 'win_m', keys: 'Win + M', label: '最小化所有窗口' },
  { id: 'win_home', keys: 'Win + Home', label: '最小化其他窗口' },
  { id: 'win_comma', keys: 'Win + ,', label: '预览桌面' },
  { id: 'win_period', keys: 'Win + .', label: '表情面板' },
  { id: 'win_space', keys: 'Win + Space', label: '切换输入法' },
  { id: 'win_left', keys: 'Win + ←', label: '窗口左贴靠' },
  { id: 'win_right', keys: 'Win + →', label: '窗口右贴靠' },
  { id: 'win_up', keys: 'Win + ↑', label: '窗口最大化' },
  { id: 'win_down', keys: 'Win + ↓', label: '窗口还原/最小化' },
  { id: 'win_shift_s', keys: 'Win + Shift + S', label: '截图' },
  { id: 'win_shift_m', keys: 'Win + Shift + M', label: '恢复最小化窗口' },
  { id: 'win_shift_left', keys: 'Win + Shift + ←', label: '窗口跨屏向左' },
  { id: 'win_shift_right', keys: 'Win + Shift + →', label: '窗口跨屏向右' },
  { id: 'win_shift_up', keys: 'Win + Shift + ↑', label: '垂直最大化窗口' },
  { id: 'win_shift_down', keys: 'Win + Shift + ↓', label: '垂直还原窗口' },
  { id: 'win_ctrl_d', keys: 'Win + Ctrl + D', label: '新建虚拟桌面' },
  { id: 'win_ctrl_left', keys: 'Win + Ctrl + ←', label: '切换到左侧虚拟桌面' },
  { id: 'win_ctrl_right', keys: 'Win + Ctrl + →', label: '切换到右侧虚拟桌面' },
  { id: 'win_ctrl_f4', keys: 'Win + Ctrl + F4', label: '关闭虚拟桌面' },
] as const satisfies readonly ShortcutDefinition[]

export type RecognizedShortcutId = (typeof RECOGNIZED_SHORTCUTS)[number]['id']

const shortcutById: ReadonlyMap<string, (typeof RECOGNIZED_SHORTCUTS)[number]> = new Map(
  RECOGNIZED_SHORTCUTS.map((shortcut) => [shortcut.id, shortcut]),
)

export const normalizeShortcutId = (shortcutId: string): string =>
  shortcutId.trim().toLowerCase().replace(/[+\-\s]+/g, '_')

export const isRecognizedShortcutId = (shortcutId: string): shortcutId is RecognizedShortcutId =>
  shortcutById.has(normalizeShortcutId(shortcutId))

export const getShortcutDefinition = (shortcutId: string): ShortcutDefinition | undefined =>
  shortcutById.get(normalizeShortcutId(shortcutId))

export const getShortcutLabel = (shortcutId: string): string | undefined => {
  const shortcut = getShortcutDefinition(shortcutId)
  return shortcut ? `${shortcut.keys} · ${shortcut.label}` : undefined
}
