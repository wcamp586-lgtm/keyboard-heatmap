# 键力（Keyboard Heatmap）

一个开源、仅支持 Windows 的本地桌面工具：用 ANSI 104 键物理布局显示按键使用热度。

> 当前版本：v0.1.6。仅在本机聚合统计按键使用习惯，不尝试还原输入内容。

## 安装

Windows x64 安装包见 [GitHub Releases](https://github.com/wcamp586-lgtm/keyboard-heatmap/releases)。也可按下面步骤从源码构建。V1 仅支持 Windows；安全桌面、锁屏、UAC、受保护软件中的按键不承诺统计。升级前请从系统托盘退出旧版本，安装后检查左上角版本号。

累计更新见 [CHANGELOG.md](CHANGELOG.md)。v0.1.6 包含 v0.1.3 首次开源之后的本地迭代，未为中间状态补造发布标签。

## V1 边界

- 统计：概览、排行、记录三个页面共享日期范围（今天、昨天、本周、本月、近 30 天、全部、自定义）。概览保留热力图和 Top 10；完整排行覆盖 104 个物理按键、85 个内置快捷键，支持搜索、排序、键区与零次项筛选，每页 20 项。名次和占比始终基于所选范围的全部同类计数。
- 记录：按日期倒序，每页 30 个日历日；包括日总量、使用键数、最常用键与峰值小时，可展开小时明细，也可从排行查看单个按键或内置快捷键的历史。趋势图仅展示当前页，具体数值可读下方表格。零次数表示“无已保存记录”，不代表未打字、未使用电脑或应用一直运行。
- 版本：品牌旁显示当前运行版本，桌面窗口标题也包含版本号。桌面版从 Tauri 读取实际运行版本，读取失败会明确提示；浏览器版从构建版本读取。没有联网检查更新功能，版本号不等于“已确认最新版”。
- 快捷键：内置目录覆盖常用 Ctrl、Alt、Win 及其明确列出的修饰组合，例如 `Win + D`、`Alt + Tab`、`Alt + F4`、`Ctrl + C`。它们只按小时汇总总次数；键盘热力图仍只显示每个实体键的使用量。
- 不统计：字符内容、任意按键顺序、未列入固定目录的组合、剪贴板、窗口标题、应用名称、设备标识。
- 不提供：回放、导出、多布局、开机启动、账号、云同步或跨设备同步。
- 存储：SQLite 仅保存 `hour_start_utc`、固定 `key_id`/`shortcut_id`、`press_count` 的小时聚合数据；不会写入原始按键事件。

## 日期与刷新规则

日期范围按本机时区解析，包含起止日期；本周从周一开始，“全部”从现有最早聚合日期开始（空库显示今天）。拒绝无效、倒序及未来日期。小时桶按其 UTC 起点对应的本地日期归属：半小时时区会出现 `00:30` 等标签；夏令时日期可能有 23 或 25 小时，重复小时通过 UTC 偏移区分。今天不填充尚未到达的小时。

今天的本机数据约每 3 秒刷新，后台隐藏时停止查询；历史范围在进入、切换筛选或手动刷新时更新。切换范围或单项后，旧请求返回不会覆盖当前结果。历史查询只读取当前页，小时详情按需读取；完整排行在 SQLite 内按固定 ID 汇总，避免把全部历史明细传给前端。以上功能复用现有两个聚合表，不增加数据库迁移或新的采集内容。

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
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

首次 Rust 构建需要从 crates.io 下载依赖。Windows 安装包输出到 `src-tauri\target\release\bundle\nsis`。

Vite 启动与构建会检查 `package.json`、Tauri 配置与 `Cargo.toml` 的版本一致性。测试覆盖完整排行、共享占比分母、日期验证、分页空白日、单项小时合计、旧库兼容及异步刷新。夏令时测试使用 [Chrono-TZ](https://docs.rs/chrono-tz/latest/chrono_tz/) 的时区数据，仅为开发测试依赖，不增加应用运行时数据采集。

## 隐私实现

Windows Raw Input 只监听键盘的物理扫描位置，立即映射为固定 `KeyId` 并按小时聚合，用于热力图。它不进行字符转换、不拦截输入，也不会影响前台软件的正常键盘输入。

快捷键是另一条严格受限的观察通道：Windows 低级键盘回调只识别内置目录中的固定组合（包括 `Win + D`、`Alt + Tab`），命中后只向本进程的消息窗口投递一个数字 `shortcut_id`。回调立即把事件交还给 Windows，不抢占、不修改系统快捷键；它不把虚拟键码、扫描码、修饰键状态、按键顺序或文字交给 SQLite。数据库仍只保存时间桶、固定 ID 和次数。系统安全桌面、锁屏、UAC 或受保护软件中的按键不承诺可统计。

暂停时，程序会先关闭内存计数门，再停止固定快捷键观察和 Raw Input 注册；恢复时两类监听都重新成功注册后才继续计数。关闭主窗口会隐藏到通知区域；左键或双击托盘图标可重新显示窗口，只有托盘菜单中的“退出”会退出程序。重复启动应用时，第二次启动只会唤醒已有窗口，不会创建第二套采集器。

除上述两个 SQLite 小时聚合表外，程序只会在本机写入一个生命周期诊断日志：UTC 时间与固定事件标签（如启动、隐藏、暂停、监听失败）。启动时检查日志，若已达到 2 MiB 则轮转，并只保留一份旧日志；单次长期运行期间不实时限制日志大小。该日志不含按键、文本、按键顺序、窗口/应用信息或设备标识。当前源码没有账号、同步、服务端 API 或遥测功能。

## 作者与致谢

- 项目作者与维护者：[wcamp586-lgtm](https://github.com/wcamp586-lgtm)。
- 协作开发：OpenAI Codex（AI-assisted）。

## 贡献

欢迎提交 Issue 和 Pull Request。新增功能必须保留项目的隐私边界：只保存固定 ID 的小时聚合；不得加入原始按键事件、字符内容、按键顺序、窗口/应用信息、设备标识或任意组合键记录。提交前请通过前端测试、Rust 测试、lint 和生产构建。

## 许可证

本项目采用 [MIT License](LICENSE) 开源。详见 [AUTHORS.md](AUTHORS.md)。
