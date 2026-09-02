# 贡献指南

感谢你的贡献。提交 Issue 或 Pull Request 前，请保持本项目的核心隐私边界：

- 只保存固定 `key_id` / `shortcut_id` 的小时聚合计数；
- 不收集字符内容、原始按键事件、按键顺序、剪贴板、窗口标题、应用名称或设备标识；
- 快捷键必须是固定、明确列出的白名单，且不得拦截或改变系统输入。

提交前请运行：

```powershell
npm run test
npm run lint
npm run build
Set-Location src-tauri
cargo test --lib
```
