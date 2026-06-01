# Torvox

本文件是 AI 编码智能体 (opencode, Claude Code, Cursor 等) 的持久上下文。
每个会话从这里开始。不要跳过。不要略读。

> **核心原则**: 将智能体视为需要清晰上下文的有能力初级工程师，而非魔法代码生成器。
> 它需要明确的指令、已知的约束、和可验证的目标才能正确工作。
> 缺少上下文时，停止并询问 — 不要猜测。

---

# 第一部分: 行为准则

以下准则偏重谨慎而非速度。对于琐碎任务，运用判断力。

## 1. 动手之前先思考

**不要假设。不要隐藏困惑。展示权衡。**

在实现之前:
- 明确陈述你的假设。如果不确定，先问。
- 如果存在多种解读，全部呈现 — 不要默默选择一种。
- 如果有更简单的方法，说出来。适当时予以反驳。
- 如果有什么不清楚的，停下来。指出困惑之处。问。
- 阅读 ADR。它们存在是为了防止错误决策。不要略读。

**危险信号**:
- 你准备写代码但还没读过 ROADMAP 当前阶段
- 你准备修改某个 crate 但没读过相关 ADR
- 你准备引入新依赖但没检查 ARCHITECTURE.md 的技术版本锁定
- 你准备实现功能但 ROADMAP 中该阶段尚不可达

## 2. 简单优先

**解决问题的最少代码。没有投机性代码。**

- 不实现未被要求的功能。
- 不为单次使用的代码创建抽象。
- 不添加未被请求的"灵活性"或"可配置性"。
- 不为不可能发生的场景编写错误处理。
- 不"以防万一"添加配置项、特征门、或泛型参数。
- 如果你写了 200 行而其实 50 行就够了，重写。

自问: "高级工程师会说这过度工程化了吗?" 如果是，简化。

**具体到本项目**:
- `torvox-core` 是 `no_std` — 不要引入需要 `alloc` 的功能除非绝对必要
- 不要提前实现未来阶段的骨架 — ROADMAP 说了何时做
- 不要为了"一致性"给一个 crate 加 feature flag 如果只有一种使用模式

## 3. 外科手术式修改

**只触碰你必须触碰的。只清理你自己造成的混乱。**

编辑已有代码时:
- 不"改进"相邻代码、注释、或格式。
- 不重构没有坏的东西。
- 匹配已有风格，即使你会用不同方式。
- 如果注意到无关的死代码，提及它 — 不要删除它。
- 不要在同一 PR 中混合功能变更和风格变更。

当你的修改产生孤儿代码时:
- 移除 YOUR 修改导致不再使用的 imports/变量/函数。
- 不要移除预先存在的死代码除非被要求。

**检验标准**: 每一行变更都应该能追溯到用户的请求。

**具体到本项目**:
- 修改 `torvox-core` 类型时，同步更新 `bridge.rs` 的桥接类型
- 修改 Rust 公共 API 时，检查 boltffi 生成绑定是否需要重新生成
- 修改 Cargo.toml 依赖版本时，确认与 ARCHITECTURE.md 版本锁定一致

## 4. 目标驱动执行

**定义成功标准。循环直到验证通过。**

将任务转化为可验证的目标:
- "添加校验" → "为无效输入写测试，然后让测试通过"
- "修复 bug" → "写一个能重现它的测试，然后让测试通过"
- "重构 X" → "确保重构前后测试都通过"

多步任务，陈述简要计划:
```
1. [步骤] → 验证: [检查项]
2. [步骤] → 验证: [检查项]
3. [步骤] → 验证: [检查项]
```

强成功标准让你能独立循环。弱标准 ("让它工作") 需要不断确认。

**每步验证流程**:
1. 写代码 → `cargo clippy -- -D warnings` 零警告
2. 写测试 → `cargo nextest -p <crate>` 通过
3. 提交前 → `cargo nextest --workspace` 全量通过
4. 完成阶段 → `nu scripts/quality-gate.nu` 通过

## 5. 不确定时停下

**编造 API 比承认不知道更危险。**

以下情况必须停止并询问:
- 文档之间信息冲突 (如 ARCHITECTURE.md 与代码不一致)
- 不确定某个函数是否应该 `pub` 还是 `pub(crate)`
- 不确定新类型该放在哪个 crate
- 不确定某个依赖是否已被项目批准
- 不确定修改是否会破坏 boltffi 绑定

---

**这些准则有效的标志**: diff 中不必要的变更更少，过度工程化导致的重写更少，
澄清问题出现在实现之前而非错误之后。

---

# 第二部分: 项目标识

- **名称**: Torvox — Android 终端模拟器
- **定位**: 从零构建的现代终端 — GPU 渲染、Rust 引擎、AI 优先
- **语言**: Rust (引擎) + Kotlin (Android UI)
- **架构**: 库优先分层。Rust crate 在 `torvox-*` 下，Kotlin 应用在 `android/app/`
- **目标平台**: Android 13+ (minSdk 33, targetSdk 36)，arm64-v8a + x86_64
- **阅读者**: 仅作者和 AI，无第三方贡献者
- **开发方法**: 规范驱动开发 (SDD) — 先写规范，再 AI 生成实现
- **许可证**: UNLICENSED — 仅作者和 AI 使用，不添加 license 声明

## 技术栈

| 层 | 技术 |
|----|------|
| 核心引擎 | Rust (stable, Edition 2024, MSRV 1.95) |
| GPU 渲染 | wgpu 29 (Vulkan) |
| VT 解析器 | libghostty-vt (GhosttyTerminal, channel-based) |
| FFI 绑定 | boltffi 0.25 (JNA 桥接, TorvoxBridge.kt) |
| Android UI | Kotlin 2.3.21 + Compose BOM 2026.05.00 + Material 3 |
| DI | Hilt 2.59.2 + KSP 2.3.9 |
| 持久化 | DataStore Preferences |
| 最低 SDK | Android 13 (API 33, Vulkan 1.3) |

完整版本锁定见 `docs/ARCHITECTURE.md §技术版本锁定`。

## 设计哲学 (摘要，详见 VISION.md)

1. **库优先** — `torvox-core` 零 Android 依赖，可嵌入任何应用
2. **GPU 从第一天起** — wgpu v29，不存在 CPU 回退路径
3. **用户与像素之间无 VM** — Rust 热路径，无 GC 暂停，无 JIT 预热
4. **AI 代理作为一等公民** — 结构化 JSON-RPC 会话协议
5. **精确优于兼容** — 顶级 VT 标准合规
6. **规范先行，AI 生成其次** — 先写规范再写代码

---

# 第三部分: 关键文件

## 必读文档

每个新会话必读。按重要性排序:

| 文件 | 重要性 | 用途 | 何时阅读 |
|------|--------|------|----------|
| `AGENTS.md` (本文件) | ★★★★★ | AI 智能体首要上下文 | 每个会话开始 |
| `docs/WORKFLOW.md` | ★★★★★ | SDD 工作流、状态管理、提交规范、质量门禁 | 每个会话开始 |
| `docs/ROADMAP.md` | ★★★★★ | 当前阶段、里程碑步骤、退出标准 | 开始任何工作之前 |
| `docs/ARCHITECTURE.md` | ★★★★★ | 完整架构、crate 结构、数据流、线程模型、技术版本锁定表 | 修改任何 crate 时 |
| `docs/SPECIFICATION.md` | ★★★★★ | VT 标准覆盖、性能目标、合规测试 | 实现 VT 功能或优化时 |
| `docs/ADR/004-pty-implementation.md` | ★★★★★ | 为什么 nix crate forkpty、不用 portable-pty、W^X 方案 | 修改 PTY 时 |
| `docs/ADR/006-testing-strategy.md` | ★★★★★ | 五层测试策略、模糊测试、属性测试、MIRI | 写任何测试时 |
| `docs/VISION.md` | ★★★★★ | 项目愿景和设计哲学 | 首次接触项目时 |

## 按需阅读文档

| 文件 | 重要性 | 用途 | 何时阅读 |
|------|--------|------|----------|
| `docs/ADR/001-language-choice.md` | ★★★☆☆ | 为什么 Rust+Kotlin 混合 | 质疑语言选择时 |
| `docs/ADR/002-architecture-pattern.md` | ★★★☆☆ | 为什么库优先分层、事件驱动、flume 而非 tokio | 修改架构时 |
| `docs/ADR/003-rendering-pipeline.md` | ★★★☆☆ | 为什么 wgpu v29 + cosmic-text + swash/skrifa + 实例化四边形 | 修改渲染时 |
| `docs/ADR/005-ai-workflow-and-tooling.md` | ★★★☆☆ | AI 工作流、SDD 方法论 | 规划 AI 协作流程时 |
| `docs/DEVELOPMENT.md` | ★★★☆☆ | 构建步骤、命令、CI/CD、开发工作流 | 构建或调试问题时 |

## 关键代码文件

| 文件 | 用途 | 关键注意 |
|------|------|----------|
| `Cargo.toml` (workspace root) | 所有 Rust crate 依赖 | 版本变更须同步 ARCHITECTURE.md |
| `android/settings.gradle.kts` | 所有 Android 模块 | 新模块须在此注册 |
| `torvox-core/src/cell.rs` | Cell, Attrs (10 个 SGR 字段), Color, DirtyMask | `no_std` 兼容 |
| `torvox-core/src/config.rs` | TerminalConfig, Shell (SystemDefault/Custom(String)), RenderConfig, FontConfig | Shell 是 Clone 不是 Copy |
| `torvox-core/src/grid.rs` | Grid, DirtyMask (Vec<u64> 分区位标志) | 任意行数 |
| `torvox-terminal/src/pty.rs` | PtyPair (spawn, resize, Read/Write, Drop) | 唯一允许 fork unsafe 的位置 |
| `torvox-gui-android/src/bridge.rs` | boltffi 导出类型 + TorvoxBridge | 唯一允许导出的位置 |
| `scripts/build-android-libs.nu` | cargo-ndk 交叉编译 + torvox-exec 构建 | 替代 rust-android-gradle |
| `scripts/generate-bindings.nu` | boltffi Kotlin 绑定生成 | 替代旧 generate-bindings.sh |
| `scripts/quality-gate.nu` | 8 步质量门 | 提交前必须通过 |
| `flake.nix` | 完整 Nix 开发环境 | 使用 flake-parts，需要 allowUnfree |

---

# 第四部分: 当前状态

- **阶段**: 3 完成。P4 进行中 (P4.1 前台服务+持久化, P4.4 MCP 服务器, P4.5 hyperlink 追踪+渲染 已完成)。详见 `docs/ROADMAP.md`。
- **下一步**: P4.3 i18n, P4.2 无障碍, P4.5 APK 构建发布
- **审计修复**: 44 项已修复, 5 项仍部分修复 (P3.2 图像协议 + 真实设备性能)

---

# 第五部分: 关键约束

## 绝对禁止

```
┌────────────────────────────────────────────────────────────────────────────┐
│ 以下约束源自 ADR 决策或已验证的技术限制。违反意味着架构错误。              │
│ 不要"只是试试"违反它们。如果你认为某个约束过时了，先提 ADR 再改。        │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│ 【语言与依赖】                                                             │
│ ✗ 添加 Java 文件 — 仅 Kotlin                                              │
│ ✗ 依赖 Termux — Torvox 是独立项目                                         │
│ ✗ 使用 portable-pty — 不支持 Android，用 nix 0.31 crate                  │
│ ✗ 使用 bincode — 已废弃 (RUSTSEC-2025-0141)，用 serde derive            │
│ ✗ 使用 rust-android-gradle — AGP 9.0+ 移除 AppExtension，不兼容          │
│   用 scripts/build-android-libs.nu (cargo-ndk v4) 代替                    │
│ ✗ 在库 crate 中使用 anyhow — 库用 thiserror 2，仅二进制可用 eyre         │
│ ✗ 在 boltffi Error 枚举中使用 `message` 字段名 — 与 Kotlin               │
│   Throwable.message 冲突。改用 `detail`                                   │
│ ✗ 添加 license 声明 — 本项目 UNLICENSED，仅作者和 AI 使用                 │
│                                                                            │
│ 【架构与安全】                                                             │
│ ✗ 在 VT 解析器中添加 `unsafe` — 需要类型安全解析器                        │
│ ✗ 在 torvox-core 中添加 `unsafe` — 零 unsafe crate                       │
│ ✗ 在多个 crate 中使用 setup_scaffolding!() — boltffi 库模式              │
│   只允许一个。所有 boltffi 类型放 torvox-gui-android/src/bridge.rs       │
│ ✗ 使用 Canvas.drawText 逐单元格 — 仅 GPU 渲染 (wgpu v29)                │
│ ✗ 在 FFI 边界传递原始字节 — 传递结构化事件 (boltffi #[data]/#[error])  │
│ ✗ 使用 /proc/self/exe 确定多调用二进制身份 — 用 argv[0] 的               │
│   file_name()。/proc/self/exe 解引用符号链接，丢失名称                    │
│                                                                            │
│ 【开发流程】                                                               │
│ ✗ 无规范编码 (vibe coding) — 必须先写规范再实现 (SDD)                     │
│ ✗ 忽略测试 — 每个公共函数需要单元测试                                     │
│ ✗ 一次修改 10+ 文件 — 分步修改，每步验证                                  │
│ ✗ 略读 ADR — 阅读它们。它们存在是为了防止错误决策                         │
│ ✗ 使用 torvox-bridge-types crate — 已删除，类型在 bridge.rs 中           │
│                                                                            │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│ 【必须遵守】                                                               │
│ ✓ 每个有意义步骤后 cargo clippy -- -D warnings                            │
│ ✓ 每个阶段完成后 cargo nextest --workspace                                │
│ ✓ 保持 AGENTS.md 更新 (新约定、新工具、状态变化)                         │
│ ✓ 逐阶段优先于逐函数 — 完成阶段退出标准再往下走                          │
│ ✓ 不确定时询问 — 不要编造 API、版本号、或 crate 结构                     │
│ ✓ 技术版本见 ARCHITECTURE.md 技术版本锁定表 — 不要自行升级               │
│ ✓ 新 crate → 更新 ARCHITECTURE.md + Cargo.toml workspace                 │
│ ✓ 新 Android 模块 → 更新 settings.gradle.kts                             │
│ ✓ 修改 Rust 公共类型 → 检查 bridge.rs 是否需要同步更新                   │
│ ✓ 修改 bridge.rs → 重新生成 Kotlin 绑定                                  │
│ ✓ 修改 Cargo.toml 依赖版本 → 确认与 ARCHITECTURE.md 版本锁定一致         │
│ ✓ 新 unsafe 块 → 在注释中记录安全不变量                                  │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

## 已知陷阱与教训

以下是从过往会话中学到的具体技术教训。避免重蹈覆辙:

| # | 陷阱 | 教训 | 来源 |
|---|------|------|------|
| 1 | `Shell::Custom(u8)` 存储路径 | `u8` 太小，改为 `Shell::Custom(String)`。Shell 失去 `Copy`，TerminalConfig 也失去 `Copy` | P0.2 审计 |
| 2 | `DirtyLine` 枚举 vs 位掩码 | ARCHITECTURE.md 指定位掩码。实现为 `DirtyMask { partitions: Vec<u64> }`，每 u64 分区 64 行，支持任意行数 | P0.2 审计 → C4 → 本次修复 |
| 3 | `thiserror 2.x` 与 `no_std` | `torvox-core` 是 `no_std`。`thiserror` 需要 `std` feature。方案: `thiserror` 设为 optional，`std` feature 启用它；`serde/postcard` 也需 `default-features = false` + `alloc` feature | P0.2 审计 |
| 4 | boltffi `setup_scaffolding!()` 多 crate 冲突 | boltffi 库模式只允许一个导出位置。跨 crate derive 导致 Kotlin 重复脚手架 | P0.6 |
| 5 | boltffi Error `message` 字段与 Kotlin 冲突 | Kotlin `Throwable.message` 与 boltffi 生成的 `message` 字段冲突。改用 `detail` | P0.6 |
| 6 | `libtorvox_core.so` 命名冲突 | lib 名与 `torvox-core` Rust crate 冲突。改为 `libtorvox_android.so` | C1 审计 |
| 7 | `/proc/self/exe` 不保留符号链接名 | 解引用到真实二进制。用 `argv[0]` 的 `file_name()` 代替 | ADR 004 |
| 8 | `rust-android-gradle` 与 AGP 9.0 不兼容 | AGP 9.0 移除了 `AppExtension`。用 `scripts/build-android-libs.nu` 代替 | P0.3 |
| 9 | `std::env::set_var` 在 Rust 1.95 是 unsafe | 需要 `unsafe` 块包裹 | 构建 |
| 10 | `nix::fcntl::fcntl` 在 nix 0.31 接受 `AsFd` 而非 `RawFd` | API 变更，注意类型转换 | P0.5 |
| 11 | `OwnedFd::from_raw_fd` + `mem::forget` 模式 | 用于在借用 fd 上操作 termios，防止 drop 关闭 fd | P0.5 |
| 12 | 一个枚举中多个 `#[from] nix::errno::Errno` 冲突 | 产生冲突的 `From` impl。用手动 `From` impl 代替 | P0.5 |
| 13 | `cargo-ndk` 仅支持 cdylib | `torvox-exec` 是 `[[bin]]`，不能通过 cargo-ndk 构建。用 `CARGO_TARGET_*_LINKER` 环境变量直接构建 | P0.5 |
| 14 | `Result<T, String>` 不被 boltffi 支持 | 必须使用 `#[error]` 枚举 | P0.6 |
| 15 | `#[error]` 枚举在 boltffi 中仅适用于枚举 | 不适用于结构体 | P0.6 |
| 16 | `DirtyMask(Vec<u64>)` 不再是 `Copy` | 含 Vec 的类型不能 Copy。DirtyMask 失去 Copy，需 Clone。Grid 的 Clone 仍可用 | P0.2→本次修复 |
| 17 | bridge.rs `shell: String` 无法表达 "系统默认" | 改为 `Shell` 枚举 (SystemDefault/Custom)，与 core 的 Shell 对齐，避免空字符串 hack | 本次修复 |
| 18 | boltffi `#[export]` 在 impl 块上要求 `pub` 方法 | 方法必须是 `pub` 才能生成 C 导出函数。私有方法不会产生 `boltffi_torvox_bridge_*` 符号 | 本次修复 |
| 19 | UniFFI 与 boltffi 绑定不兼容 | UniFFI 生成 `uniffi_*` 函数名，boltffi 生成 `boltffi_*` 函数名。切换 FFI 框架后 Kotlin 绑定必须重写 | 本次修复 |
| 20 | `AndroidSurface.write_to_pty` 调用 `vt_write` 而非 PTY | Surface 需拥有 `Session` (PTY+terminal+parser)，而非独立的 GhosttyTerminal。`write_to_pty` 必须走 `Session::write()` → PTY master fd | P3 修复 |
| 21 | Grid scrollback `Vec::remove(0)` O(n) | 改为 `VecDeque<Line>`，`push_back` + `pop_front` 均 O(1)。`drain(..excess)` 也替换为循环 `pop_front()` | P3 修复 |
| 22 | 手写 unicode.rs 缺少 ZWJ/variation selector | 替换为 `unicode-width 0.2` crate (no_std + CJK feature)，完整 UAX #11 实现 | P3 修复 |
| 23 | Nightly CI 缺少 Zig + libghostty-vt patch | fuzz/miri/bench/geiger job 都依赖 libghostty-vt-sys 编译，需要 Zig + `git clone + git apply` | P3 修复 |
| 24 | `GhosttyTerminal.history_size()` 不存在 | libghostty-vt Terminal API 是 `scrollback_rows()` 而非 `history_size()`。API 命名与自建 TerminalState 不同 | P3 修复 |
| 25 | GhosttyTerminal channel-based 后 `resize()` 签名变更 | `resize` 只需 `(rows, cols)` 两参数，不再需要 `(cell_width, cell_height)` — libghostty-vt 内部管理 | P3 修复 |
| 26 | `boltffi generate kotlin` 不生成桥接方法 | boltffi 0.25.2 CLI 对复杂依赖树 (wgpu, cosmic-text) 静默失败，只生成 WireReader/WireWriter/Native 基础设施，不检测 `#[boltffi::export]` 方法。已验证三种命名变体 (TorvoxGuiAndroid.kt, Torvox.kt, TorvoxAndroid.kt) 均无效。解决方案：JNA 手动绑定 (TorvoxBridge.kt，已验证 24 个符号全部导出)。由 `nm` 确认 `libtorvox_android.so` 正确导出所有 `boltffi_torvox_bridge_*` 符号。 | P4 |
| 27 | libghostty-vt hyperlink URI 不走 Style API | 超链接 URI 通过 `GridRef::hyperlink_uri(buf)` 获取，而非 `point.style()`。需要先 `cell.has_hyperlink()` 判断，再单独调用 `hyperlink_uri()`。Style 无 hyperlink 字段。 | P4 |
| 28 | rkyv `to_bytes` 需要 ArrayVec 容量 | `rkyv::api::high::to_bytes::<T, 4096>(&val)` 需要 const generic 容量参数 (4096 bytes 通常足够)。`from_bytes::<T, rancor::Error>(&bytes)` 需 `std` + `bytecheck` 特性。rkyv 0.8 通过 `rkyv::rancor` 导出 rancor。 | P4 |

---

# 第六部分: 构建命令

```bash
# ── 日常开发最常用 ────────────────────────────────────
cargo build                                        # Debug 构建
cargo nextest --workspace                          # 全部测试
cargo clippy -- -D warnings                        # 零警告 (必须)
cargo fmt --check                                  # 格式检查
nu scripts/quality-gate.nu                          # 8 步全量质量门
```

完整命令见 `docs/DEVELOPMENT.md`。

---

# 第七部分: 技术版本锁定

完整版本锁定表见 `docs/ARCHITECTURE.md §技术版本锁定`。**不要自行升级版本**。如需变更，先更新 ARCHITECTURE.md 版本锁定表，再改 Cargo.toml / build.gradle.kts。

---

# 第八部分: 约定

**所有编码规范 (Rust, Kotlin, Git, Nix, Nushell, GitHub Actions) 见 `docs/WORKFLOW.md §九`。**

---

# 第九部分: 测试要求

完整策略见 `docs/ADR/006-testing-strategy.md`。

## 五层测试金字塔

| 层 | 工具 | 频率 | 覆盖目标 |
|----|------|------|----------|
| L0 编译时 | clippy, geiger, MIRI, fmt | 每 PR | unsafe 审计、类型安全、格式 |
| L1 单元 | `cargo nextest` + 内联 `#[test]` | 每 PR | 每个公共函数 |
| L2 属性 | `proptest 1.11` (10K+ 用例) | 每 PR | VT 解析器、Grid、PTY 编码 |
| L3 集成 | `torvox-integration-tests` | 每 PR | 跨 crate 交互、会话生命周期 |
| L4 模糊 | `cargo-fuzz` (4 目标, 1B+ 迭代/夜) | 每夜 | 零崩溃 |

## 具体测试要求

- **每个公共函数**: 必须有单元测试
- **VT 解析器**: proptest (10K+ 用例) + fuzz (每夜 1B+ 迭代)
- **Grid/DirtyMask**: proptest (不变量: mark 后 is_dirty 为 true, clear 后 any_dirty 为 false)
- **序列化**: 每个可序列化类型须有 serde roundtrip 测试
- **PTY**: 非阻塞读写、resize、kill_on_drop (Linux 单元测试, Android 集成测试)
- **boltffi 桥接**: Kotlin 调用 Rust 函数，返回值正确 (端到端测试)

---

# 第十部分: 架构关键点

完整架构见 `docs/ARCHITECTURE.md`。智能体必须记住:

- **Crate 依赖方向 (严格单向)**: `torvox-core` → `torvox-terminal` → `torvox-renderer` → `torvox-gui-android`。依赖只能从下往上。
- **关键不变量**: torvox-core 需 `alloc` (no_std)；terminal 拥有所有 PTY I/O (fork 是唯一 unsafe)；renderer 单线程；FFI 传递结构化事件；热路径用 flume 不用 tokio。
- **渲染管线**: fontdb → cosmic-text → swash → guillotiere → 实例化四边形 → 单次 draw call。详见 ARCHITECTURE.md §渲染管线。

---

# 第十一部分: AI 工作流

## 会话开始流程

1. **阅读本文件** — 完整阅读，不要略读
2. **阅读 ROADMAP.md 当前阶段** — 确认当前要做什么
3. **阅读相关 ADR** — 如果修改 PTY 读 ADR 004，修改渲染读 ADR 003，以此类推
4. **检查当前代码状态** — 确认文档描述与实际代码一致

## 实现流程

1. **规划**: 列出本次会话要完成的具体步骤，包含验证标准
2. **类型先行**: 先定义类型，再实现行为
3. **小步提交**: 每个逻辑步骤提交，不积累 10+ 文件变更
4. **验证**: 每步 `cargo clippy -- -D warnings && cargo nextest -p <affected-crate>`
5. **同步**: 修改 Rust 类型 → 检查 bridge.rs → 重新生成 Kotlin 绑定
6. **更新**: 完成后更新 AGENTS.md 和相关文档

## 修改 Rust 类型时检查清单

```
[ ] 类型定义在哪个 crate? (core → terminal → renderer → gui-android)
[ ] 该类型是否通过 boltffi 导出? (检查 bridge.rs)
[ ] bridge.rs 的桥接类型是否需要同步更新?
[ ] boltffi Kotlin 绑定是否需要重新生成?
[ ] 此类型变更是否影响 serde 序列化格式? (破坏性变更?)
[ ] 相关单元测试是否需要更新?
```

## 添加新函数时检查清单

```
[ ] 公共还是内部? 默认 pub(crate)，仅必要时 pub
[ ] 是否需要单元测试? 每个公共函数必须
[ ] 是否需要 proptest? VT 解析器和 Grid 必须
[ ] 错误类型用 thiserror，不用 anyhow (库 crate)
[ ] no_std 兼容? (如果在 torvox-core 中)
[ ] unsafe? (如果在 torvox-core 中，不允许)
```

---

# 第十二部分: 会话结束检查清单

每次工作会话后，逐项检查:

## Rust 质量

1. [ ] `cargo nextest --workspace` 通过
2. [ ] `cargo clippy -- -D warnings` 通过
3. [ ] `cargo fmt --check` 通过
4. [ ] `cargo geiger` 检查: 无新的 unsafe 引入 (torvox-core 零 unsafe)
5. [ ] 测试覆盖率: 新公共函数都有单元测试?

## Android 质量 (如有 Android 变更)

6. [ ] `cd android && ./gradlew lint` 通过
7. [ ] `cd android && ./gradlew test` 通过
8. [ ] `./scripts/build-android-libs.nu` 成功
9. [ ] boltffi 绑定已重新生成 (如 bridge.rs 有变更)

## 文档同步

10. [ ] AGENTS.md 更新了新约定/工具/状态变化
11. [ ] 重大决策创建了 ADR
12. [ ] 添加了新 crate? → 更新 ARCHITECTURE.md 和 Cargo.toml workspace
13. [ ] 添加了新 Android 模块? → 更新 settings.gradle.kts
14. [ ] 修改了依赖版本? → 确认 ARCHITECTURE.md 版本锁定表同步

## 序列化兼容性

15. [ ] 修改了 serde 序列化的类型? → 评估是否破坏已保存状态兼容性

---

# 第十三部分: 已知问题与待办

以下问题汇总自项目审计 (原 docs/AUDIT.md) 和开发历史，是 AGENTS.md 作为单一权威源的已知问题跟踪。
已修复的标记 ✅，部分修复的标记 ⚠️，仍待修复的标记 🔲。

### ✅ 已修复 (46 项)

| # | 问题 | 修复 |
|---|------|------|
| 1 | `torvox-bench` 基准骨架 | criterion benchmarks 已添加 |
| 2 | CI `@main` 引用 | 所有 actions 固定到 v4/v3/v2 |
| 3 | Grid scrollback `Vec::remove(0)` O(n) | 改为 `VecDeque<Line>`，`push_back` + `pop_front` O(1) |
| 4 | 手写 unicode.rs 缺少 ZWJ/variation selector | 替换为 `unicode-width 0.2` crate (no_std + CJK) |
| 5 | boltffi `#[export]` 要求 `pub` 方法 | 所有 bridge 方法已改为 pub |
| 6 | CI 模拟器测试需 Rust 交叉编译 | CI 已添加 cargo-ndk 步骤 |
| 7 | `echo_cells()` 空操作 | 已移除 |
| 8 | `AndroidSurface.write_to_pty` 调用 vt_write 而非 PTY | 改走 `Session::write()` → PTY master fd |
| 9 | GhosttyTerminal `history_size()` 不存在 | 改用 `scrollback_rows()` API |
| 10 | GhosttyTerminal channel-based 后 `resize()` 签名变更 | 只需 `(rows, cols)`，去掉 cell_width/cell_height |
| 11 | Nightly CI 缺少 Zig + libghostty-vt patch | 添加 Zig + `git clone + git apply` 步骤 |
| 12 | build-android-libs.nu env var bug | str replace 修复 |
| 13 | GhosttyTerminal unsafe Send/Sync | channel-based 架构消除 |
| 14 | release APK debug 签名 | 添加 release signing config |
| 15 | CI 缺少 boltffi 绑定校验 | 添加 bindings diff step |
| 16 | Grid::get_mut 幽灵脏位 | 先检查行存在再标脏 |
| 17 | GpuContext 硬编码 1080×1920 | 参数化 width/height |
| 18 | 双 VT 引擎策略 | GhosttyTerminal 唯一引擎 |
| 19 | 渲染线程跨会话共享 | 每会话独立线程 |
| 20 | LRU cache O(n log n) sort | 改用 `lru` crate O(1) |
| 21 | ROADMAP "etagere" 过时引用 | 改为 "guillotiere" |
| 22 | ROADMAP .sh → .nu 脚本引用 | 已修正 |
| 23 | quality-gate.nu 使用 cargo test | 改为 cargo nextest |
| 24 | JNA 5.17.0 → 5.18.1 | 已升级 |
| 25 | ARCHITECTURE.md bridge 路径 | 修正为 TorvoxBridge.kt |
| 26 | CI Zig 步骤无注释 | 添加注释 |
| 27 | libghostty-vt commit hash 记录 | patch + build.rs |
| 28 | pollster → futures | 改用 `futures::executor::block_on` |
| 29 | proptest → quickcheck | 改用 `quickcheck 1.1` |
| 30 | raw-window-handle → wgpu re-export | 添加 raw-window-handle 直接依赖 (wgpu 29 不再 re-export) |
| 31 | GpuUniforms 硬编码 cell_size [8, 16] | 添加 FontPipeline::cell_metrics() 基于真实字体度量 |
| 32 | build_cell_instances 重复代码 | 移除 `build_cell_instances_from_flat` 死代码 |
| 33 | ANativeWindow 生命周期风险 | 添加 AndroidSurface::Drop + GpuContext::has_surface() |
| 34 | quality-gate.nu cargo audit 仅 warn | 改用 `--json` + 检查 `vulnerabilities.found` |
| 35 | Grid 分散分配 | Line 改用 `Box<[Cell]>`，添加 `row_cells()` 直接访问器 |
| 36 | lru 0.12.5 RUSTSEC-2026-0002 unsound | 升级到 lru 0.16 (修补版本) |
| 37 | P3.1 DEC 2026 / DECCRA / DECERA 缺失测试 | 添加 3 个集成测试 (libghostty-vt 处理) |
| 38 | P3.3.1 PGO 缺失 | 添加 `scripts/build-pgo.nu` (3 阶段流程) |
| 39 | P3.3.5 <10MB 内存预算无验证 | 添加 `torvox-core/examples/memory_check.rs`，45KB idle PASS |
| 40 | P3.4 fuzz 目标只有 4 个 | 添加 3 个 (fuzz_grid_ops, fuzz_selection, fuzz_attrs) |
| 41 | P4.1 foreground service 无 wake lock / 计数 | 添加 wake lock + session count + onTaskRemoved + 通知点击 |
| 42 | P4.4 MCP server 缺失 | 添加 `torvox-mcp` crate (8 工具，11 测试，端到端验证) |
| 43 | CI quickcheck 属性测试 | 添加 `QUICKCHECK_TESTS=10000 cargo test --workspace` |
| 44 | P4.2 会话持久化缺失 | 添加 save_session/restore_session/has_saved_session 到 bridge、surface、Drop handler、Android lifecycle |
| 45 | P4.5 OSC 8 hyperlink 追踪缺失 | 添加 uri: Option<String> 到 CellSnapshot, populate_uri 在 build_snapshot/build_dumped_grid, GridSnapshot::uri_at |
| 46 | P4.5 OSC 8 hyperlink 渲染缺失 | 添加 mouse position 跟踪, shader blue tint flag (bit 4), WGSL hyperlink 着色, get_hovered_url bridge |

### ⚠️ 部分修复

| # | 问题 | 已修复 | 剩余 |
|---|------|--------|------|
| 1 | render_frame 全量实例构建 | snapshot 支持 dirty_rows | dirty_rows 需 libghostty-vt 暴露行级脏标记 |
| 2 | release.yml 缺少 GitHub Release | 添加 `softprops/action-gh-release@v2` | 需 `v*` tag 触发 |
| 3 | `extractSelectedText()` 返回占位符 | 使用 `bridge.scrollbackLine()` | 实际可用 |
| 4 | P3.2 Sixel/Kitty/iTerm2 图像协议 | 无 | libghostty-vt 不支持 + 需要新 GPU 纹理管线 |
| 5 | P3.3.2 5ms 延迟 / P3.3.3 120 FPS / P3.3.5 真实设备 | 内存 PASS | 需要真实 Android 设备性能分析 |

# 附录: 术语表

| 术语 | 含义 |
|------|------|
| SDD | Specification-Driven Development — 规范驱动开发 |
| VT | Video Terminal — 视频终端 (VT100/220/320/ECMA-48 标准) |
| PTY | Pseudo-Terminal — 伪终端 |
| W^X | Write XOR Execute — Android 安全策略，限制同时可写可执行 |
| SGR | Select Graphic Rendition — VT 终端文本属性控制序列 |
| CSI | Control Sequence Introducer — VT 控制序列引导 |
| OSC | Operating System Command — VT 操作系统命令序列 |
| DirtyMask | Vec<u64> 分区位标志，每 u64 覆盖 64 行，支持任意行数 |
| SPA | Single Point of Authority — 单一权威源 (如 ARCHITECTURE.md 是版本的 SPA) |
| PIE | Position-Independent Executable — 位置无关可执行文件 (Android 要求) |
