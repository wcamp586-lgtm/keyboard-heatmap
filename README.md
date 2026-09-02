# 键力（Keyboard Heatmap）

一个开源、仅支持 Windows 的本地桌面工具：用 ANSI 104 键物理布局显示按键使用热度。

> 当前版本：v0.1.3。仅在本机聚合统计按键使用习惯，不尝试还原输入内容。

## 安装

Windows 安装包发布后可从本仓库的 Releases 下载并运行；当前也可按下面的源码运行步骤本地构建。V1 仅支持 Windows；安全桌面、锁屏、UAC、受保护软件中的按键不承诺统计。

## V1 边界

- 统计：实时热力图、今天/本周/本月、Top 10 单键、每小时柱状图、本地历史、托盘暂停，以及固定目录内的常用快捷键总次数。
- 快捷键：内置目录覆盖常用 Ctrl、Alt、Win 及其明确列出的修饰组合，例如 `Win + D`、`Alt + Tab`、`Alt + F4`、`Ctrl + C`。它们只按小时汇总总次数；键盘热力图仍只显示每个实体键的使用量。
- 不统计：字符内容、任意按键顺序、未列入固定目录的组合、剪贴板、窗口标题、应用名称、设备标识。
- 不提供：回放、导出、多布局、开机启动、账号、云同步或跨设备同步。
- 存储：SQLite 仅保存 `hour_start_utc`、固定 `key_id`/`shortcut_id`、`press_count` 的小时聚合数据；不会写入原始按键事件。

## 从源码运行

先安装 Node.js、Rust MSVC 工具链、Visual Studio C++ Build Tools 和 WebView2 Runtime。

```powershell
git clone https://github.com/wcamp586-lgtm/keyboard-heatmap.git
Set-Location keyboard-heatmap
npm ci
npm run dev
```

运行桌面端：

```powershell
npm run tauri -- dev
```

## 验证

```powershell
npm run test
npm run lint
npm run build
npm run tauri -- build --bundles nsis
Set-Location src-tauri
cargo test --lib
```

首次 Rust 构建需要从 crates.io 下载依赖。Windows 安装包输出到 `src-tauri\target\release\bundle\nsis`。

## 隐私实现

Windows Raw Input 只监听键盘的物理扫描位置，立即映射为固定 `KeyId` 并按小时聚合，用于热力图。它不进行字符转换、不拦截输入，也不会影响前台软件的正常键盘输入。

快捷键是另一条严格受限的观察通道：Windows 低级键盘回调只识别内置目录中的固定组合（包括 `Win + D`、`Alt + Tab`），命中后只向本进程的消息窗口投递一个数字 `shortcut_id`。回调立即把事件交还给 Windows，不抢占、不修改系统快捷键；它不把虚拟键码、扫描码、修饰键状态、按键顺序或文字交给 SQLite。数据库仍只保存时间桶、固定 ID 和次数。系统安全桌面、锁屏、UAC 或受保护软件中的按键不承诺可统计。

暂停时，程序会先关闭内存计数门，再停止固定快捷键观察和 Raw Input 注册；恢复时两类监听都重新成功注册后才继续计数。关闭主窗口会隐藏到通知区域；左键或双击托盘图标可重新显示窗口，只有托盘菜单中的“退出”会退出程序。

除上述两个 SQLite 小时聚合表外，程序只会在本机写入一个生命周期诊断日志：UTC 时间与固定事件标签（如启动、隐藏、暂停、监听失败）。该日志不含按键、文本、按键顺序、窗口/应用信息或设备标识。当前源码没有账号、同步、服务端 API 或遥测功能。

## 作者与致谢

- 项目作者与维护者：[wcamp586-lgtm](https://github.com/wcamp586-lgtm)。
- 协作开发：OpenAI Codex（AI-assisted）。

## 贡献

欢迎提交 Issue 和 Pull Request。新增功能必须保留项目的隐私边界：只保存固定 ID 的小时聚合；不得加入原始按键事件、字符内容、按键顺序、窗口/应用信息、设备标识或任意组合键记录。提交前请通过前端测试、Rust 测试、lint 和生产构建。

## 许可证

本项目采用 [MIT License](LICENSE) 开源。详见 [AUTHORS.md](AUTHORS.md)。
