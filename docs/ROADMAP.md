<!-- AUDIT: 2026-06-04 — Full re-verification. Roborazzi 9→4, quickcheck 19→23, OSC 8 shader tint 未实现. -->
# Torvox 路线图

## 当前状态

| 阶段 | 状态 |
|------|------|
| P0: 基础设施 | ✅ 完成 |
| P1: 终端引擎 | ✅ 完成 |
| P2: 交互式终端 | ✅ 完成 |
| P3: 合规性 & 性能 | ✅ 完成 |
| P4: 打磨 & 发布 | 🔄 进行中 — P4.1, P4.4, P4.5 已完成 |

---

## 阶段 4: 打磨 & 发布

> ⚠️ **进行中。** P4.1, P4.4, P4.5 已完成。

**退出标准**: v1.0.0-beta.1 — GitHub Releases 上的签名 APK。

### P4.1 — 前台服务 + 持久化 ✅

- ✅ `FOREGROUND_SERVICE_SPECIAL_USE` 前台服务带通知
- ✅ 跨死亡/OOM 的会话持久化 (rkyv 序列化, save/restore on bridge + surface.rs + Drop autosave)
- ✅ Wake lock 管理 (30 分钟最大超时, 非 ref-counted)
- ⬜ 自动恢复上次会话 — restore_session() 接口已实现, 需启动时自动调用

### P4.2 — 多会话管理 ✅

- ✅ 每个会话独立 PTY (TorvoxRuntime.kt 管理 per-session bridges)
- ✅ 会话创建/切换/关闭 (TerminalViewModel + SessionDrawer)
- ✅ 侧边会话面板 (SessionDrawer.kt: 会话列表, 添加/关闭, 设置入口)

### P4.3 — MCP 服务器 ✅

- ✅ JSON-RPC MCP 服务器 (8 工具: list_sessions, read_grid, read_scrollback, read_cursor, read_selection, read_title, send_input, send_signal)
- ✅ Unix 域套接字, 默认关闭 (需 `--socket <path>`)
- ✅ 写入同意提示 — `--mcp-allow-write` flag

### P4.4 — Hyperlink 追踪 ✅

- ✅ OSC 8 hyperlink 追踪 (CellSnapshot.uri, GridSnapshot::uri_at, populate_uri)
- ✅ OSC 8 hyperlink 渲染 (鼠标悬停检测, get_hovered_url bridge; shader 蓝色 tint 未实现)

### P4.5 — 主题模式 ✅

- ✅ 日间/夜间/跟随系统 三种模式 (ThemeMode 枚举, SettingsScreen ThemeModeSelector)
- ✅ 独立日间/夜间主题选择 (分别设置 dayThemeName/nightThemeName)
- ✅ Material You 动态取色 (Android 12+ dynamicDarkColorScheme/dynamicLightColorScheme)

### P4.6 — 无障碍 (待实现)

1. TalkBack 终端内容朗读
2. 操作描述 (AccessibilityNodeInfo)
3. 响铃声音 (BEL → 系统 notification sound)
4. 高对比度模式

### P4.7 — 国际化 (待实现)

1. i18n 框架 (Compose string resources)
2. 中/英/日/韩/德/法 初始翻译
3. RTL 布局支持 (阿拉伯语, 希伯来语)

### P4.8 — Beta 发布 (待实现)

1. GitHub Release (签名 APK + AAB)
2. F-Droid 提交
3. 崩溃报告 (用户同意)
4. 更新日志

---

## 已完成阶段归档

### 阶段 0: 基础设施 ✅

| 里程碑 | 交付物 | 状态 |
|--------|--------|------|
| P0.1 | Rust workspace 脚手架 (9 workspace members) | ✅ |
| P0.2 | `torvox-core` 完整类型系统 (Cell, Grid, Selection, Cursor, Config, Event) | ✅ |
| P0.3 | Android 外壳 (Kotlin 2.3.21 + Compose BOM 2026.05.00 + Hilt DI) | ✅ |
| P0.4 | 文档 + CI (AGENTS.md, 3 GHA workflows, quality-gate.nu, test-all.nu) | ✅ |
| P0.5 | PTY 验证 (nix 0.31 openpty + fork + exec, W^X torvox-exec) | ✅ |
| P0.6 | boltffi 桥接验证 (Kotlin 调用 Rust, 端到端测试) | ✅ |

### 阶段 1: 终端引擎 ✅

| 里程碑 | 交付物 | 状态 |
|--------|--------|------|
| P1.1 | VT 解析器 (GhosttyTerminal, libghostty-vt) | ✅ |
| P1.2 | PTY 会话集成 (Session 编排器, flume 通道, 脏区域通知) | ✅ |
| P1.3 | 字体管线 (cosmic-text 0.19 + swash 0.2.7 + guillotiere 0.7) | ✅ |
| P1.4 | GPU 渲染管线 (wgpu v29, 实例化四边形, DirtyMask) | ✅ |
| P1.5 | Android Surface 渲染 (SurfaceView, wgpu Surface) | ✅ |
| P1.6 | 输入处理 (Kitty 键盘协议, SGR 鼠标, 触摸手势) | ✅ |

**VT 合规性:**

| 标准 | 状态 | 备注 |
|------|------|------|
| VT100 | ✅ 完整 | DEC AWB, CKM, DECCOLM, 原点模式, 滚动区域 |
| VT220 | ⚠️ 部分 | GhosttyTerminal VT 引擎, DECSC/DECRC, DECSTBM |
| xterm | ⚠️ 部分 | 256 色 ✅, 真彩色 ✅, 括号粘贴 ✅; DECSET/DECRST 仅少数 |

| 扩展 | 状态 |
|------|------|
| Kitty 键盘协议 | ✅ |
| OSC 8 超链接 | ✅ |
| 真彩色 (24-bit) | ✅ |
| 256 调色板 | ✅ |
| 粗体/斜体/下划线/删除线 | ✅ |
| 括号粘贴模式 | ✅ (DEC 2004) |
| 鼠标跟踪 (SGR 模式) | ✅ |
| OSC 52 剪贴板 | ❌ 未实现 |
| OSC 133 Shell 集成 | ❌ 未实现 |
| 同步输出 (DEC 2026) | ❌ 未实现 |
| 焦点事件 | ❌ 未实现 |
| 双宽/双高 (DECDWL/DECDHL) | ❌ 未实现 |
| 矩形区域 (DECCRA/DECERA/DECFRA) | ❌ 未实现 |
| 选择性擦除 (DECSEL/DECSED) | ❌ 未实现 |
| DECRPM 模式参数报告 | ❌ 未实现 |
| Sixel 图形 | ❌ 未实现 |
| Kitty 图形协议 | ❌ 未实现 |
| iTerm2 图像协议 | ❌ 未实现 |
| OSC 7 CWD | ❌ 未实现 |

**Unicode 支持:**

| 特性 | 状态 | 备注 |
|------|------|------|
| UTF-8 编码 | ✅ | Rust 原生 |
| CJK 宽字符 | ⚠️ | East Asian Width 部分处理 |
| 字形簇聚类 | ⬜ | UAX#29 — cosmic-text 基本聚类, 完整未验证 |
| Emoji (ZWJ) | ⬜ | 通过 cosmic-text + swash/skrifa, ZWJ 未测试 |
| 双向文本 | ❌ | |
| 组合字符 | ⬜ | 未测试 |

### 阶段 2: 交互式终端 ✅

**P2.1 — 回滚缓冲:**
- ✅ VecDeque 回滚 (50K 行默认, 可配置) — Grid.scrollback
- ✅ 触摸滚动 (fling 手势)
- ✅ 滚动位置指示器
- ✅ 滚动时锁定键盘输入
- ✅ 搜索功能 (在回滚中查找文本)

**P2.2 — 选择:**
- ✅ 字符/词/行/块选择模式
- ✅ 放大镜精确选择 (Android Magnifier)
- ✅ 复制到剪贴板 (Android ClipboardManager)
- ✅ 检测到链接时打开 URL (Intent.ACTION_VIEW)
- ✅ OSC 8 超链接悬停高亮 — 通过 P4.4 hyperlink 渲染实现
- ⬜ 语义选择 (OSC 133 Shell 集成)

**P2.3 — 修饰键栏:**
- ✅ 屏幕修饰键 (Ctrl/Alt/Esc/Tab/方向键)
- ✅ 粘滞模式 (双击重置修饰键)
- ✅ 可配置布局 (用户自定义键)
- ✅ 滑动手势 (左滑=Esc, 右滑=Tab)
- ⬜ Nerd Font 字形用于键标签

**P2.4 — 字体 + 主题:**
- ✅ 字体大小调整 (DataStore 持久化, 8sp–32sp)
- ✅ 字体选择器 (12 个字体/字体族选项, JetBrains Mono Nerd Font 为首)
- ✅ 10 内置主题 (全部暗色: Catppuccin Mocha, Dracula, Solarized Dark, Nord, Tokyo Night, Gruvbox Dark, One Dark, Monokai, GitHub Dark, Rose Pine)
- ✅ Material You 动态取色 (Android 12+)
- ✅ 自定义主题 (key=value 格式, #hex/rgb 颜色)
- ✅ 24-bit TrueColor 支持 (vim 语法高亮)

**P2.5 — 设置:**
- ✅ Jetpack Compose 设置屏幕
- ✅ Shell 选择 (自由输入框, 默认 /system/bin/sh)
- ✅ 字体/字号配置
- ✅ 配色方案选择 (含预览)
- ✅ 主题模式 (日间/夜间/跟随系统)
- ✅ 触摸行为配置 (右键粘贴/中键粘贴/无)
- ✅ 会话管理 (创建/关闭)
- ✅ DataStore 持久化

### 阶段 3: 合规性 & 性能 ✅

**P3.1 — VT 功能 (由 libghostty-vt 提供):**
- ✅ DSR 5/6, DA, SpecialGraphics, ENQ answerback (libghostty-vt)
- ✅ REP, CHT, LNM, IRM (libghostty-vt)
- ✅ DECCKM 光标键应用模式

**P3.2 — 现代扩展:**
- ✅ OSC 8 超链接 (URI 追踪, 查询响应)
- ⬜ Sixel 图形 — 需渲染器支持
- ⬜ Kitty 图形协议 — 需渲染器支持
- ⬜ iTerm2 图像协议 — 需渲染器支持

**P3.3 — 性能优化:**
- ✅ PGO 构建 (`scripts/build-pgo.nu`)
- ✅ 图集 LRU 驱逐策略 — `FontPipeline.glyph_cache.pop_lru()`
- ✅ 空闲内存 <10MB — `torvox-core/examples/memory_check.rs`
- ✅ flume 通道调优 — command 256, output 128
- ✅ 实例缓冲区 flags 编码
- ✅ 着色器预热 — `GpuContext::warmup()`
- ⬜ 亚 5ms 输入→像素延迟
- ⬜ `find /` 下 120 FPS

**P3.4 — 测试基础设施:**
- ✅ 模糊测试 (7 cargo-fuzz 目标, nightly CI)
- ✅ `cargo geiger` 零 unsafe (torvox-core 0/0)
- ✅ MIRI 通过 (无未定义行为)
- ✅ `quickcheck` 属性测试 23 个 (10K+ 案例)
- ✅ `cargo-mutants` 配置 (`mutants.toml`, 目标 ≥70%)
- ✅ Kani 形式化验证 (4 个证明函数)
- ✅ Roborazzi 视觉回归测试 (4 个 Kotlin screenshot 测试)
- ✅ Maestro E2E 流程 (4 个流程: smoke-test, full-test, modifier-keys, settings-navigation)
- ✅ 统一测试脚本 (`scripts/test-all.nu`, 16 个检查, 7 阶段)
- ⬜ Android UI/Espresso — 测试文件存在, 需设备验证
- ⬜ 变异测试 CI — `mutants.toml` 已配置, 需 CI 集成
- ⬜ Kani CI — 证明代码已完成, 需 nightly CI 集成

---

## 排除项

| 功能 | 理由 |
|------|------|
| **桌面环境** | 超出终端模拟器范围。Android 有 Taskbar/Launcher 等方案。 |
| **云同步** | 本地优先。需要远程访问时用 SSH。 |
| **文件浏览器/编辑器** | 终端内用 `ls`/`vim`/`nano`/`nnn` 即可。包管理器提供这些工具。 |
| **AI 集成** | 与终端核心功能无关。可以通过 SSH 连接到 AI 服务器。 |
| **多架构 QEMU 模拟** | PRoot 支持 QEMU 用户模式, 但模拟 x86 程序在 ARM 设备上极慢。 |
| **Android 原生隔离** | Landlock/seccomp 内核版本依赖严重, PRoot (ptrace) 兼容性最好。 |
