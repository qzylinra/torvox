# Torvox

本文件是 AI 编码智能体 (opencode, Claude Code, Cursor 等) 的持久上下文。

> **原则**: 将智能体视为需要清晰上下文的有能力初级工程师，而非魔法代码生成器。每个会话从这里开始。

# CLAUDE.md

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.

---

## 项目标识

- **名称**: Torvox — Android 终端模拟器
- **语言**: Rust (引擎) + Kotlin (Android UI)
- **架构**: 库优先分层。Rust crate 在 `torvox-*` 下, Kotlin 应用在 `android/app/`
- **阅读者**: 仅作者和 AI，无第三方贡献者
- **开发方法**: 规范驱动开发 (SDD) — 先写规范, 再 AI 生成实现

## 关键文件

| 文件 | 重要性 | 用途 |
|------|--------|------|
| `AGENTS.md` (本文件) | ★★★★★ | AI 智能体首要上下文文件 |
| `docs/VISION.md` | ★★★★★ | 项目愿景和设计哲学。首先阅读。 |
| `docs/ARCHITECTURE.md` | ★★★★★ | 完整架构, crate 结构, 数据流, 线程模型, 技术版本锁定。 |
| `docs/SPECIFICATION.md` | ★★★★★ | 详细标准覆盖, 性能目标, 合规测试。 |
| `docs/ROADMAP.md` | ★★★★★ | 当前阶段, 里程碑步骤, 退出标准。 |
| `docs/ADR/001-language-choice.md` | ★★★☆☆ | 为什么 Rust+Kotlin 混合。 |
| `docs/ADR/002-architecture-pattern.md` | ★★★☆☆ | 为什么库优先分层。 |
| `docs/ADR/003-rendering-pipeline.md` | ★★★☆☆ | 为什么 wgpu v29 + cosmic-text + swash/skrifa。 |
| `docs/ADR/004-pty-implementation.md` | ★★★★★ | 为什么 nix crate forkpty, 不用 portable-pty。W^X 方案。 |
| `docs/ADR/005-ai-workflow-and-tooling.md` | ★★★☆☆ | AI 工作流, 规范驱动开发方法论。 |
| `docs/ADR/006-testing-strategy.md` | ★★★★★ | 五层测试策略, 模糊测试, 属性测试, MIRI。 |
| `docs/DEVELOPMENT.md` | ★★★☆☆ | 构建步骤, 命令, CI/CD, 开发工作流。 |
| `Cargo.toml` (workspace root) | ★★★★★ | 所有 Rust crate 依赖。 |
| `android/settings.gradle.kts` | ★★★★★ | 所有 Android 模块。 |

## 当前状态

- **阶段**: 0→1 (基础设施→终端引擎) — P0.1 完成, P0.2 核心类型完成
- **下一步**: P0.3 Android 外壳

## 关键约束

```
┌─────────────────────────────────────────────────────────────────┐
│ 禁止:                                                           │
│ - 添加 Java 文件。仅 Kotlin。                                  │
│ - 依赖 Termux。Torvox 是独立项目。                              │
│ - 使用 Canvas.drawText 逐单元格。仅 GPU 渲染。                 │
│ - 在 VT 解析器中添加 `unsafe`。需要类型安全解析器。            │
│ - 使用 portable-pty。它不支持 Android。用 nix 0.31 crate。     │
│ - 使用 bincode。它已废弃 (RUSTSEC-2025-0141)。用 postcard 1.1。│
│ - 略读 ADR。阅读它们。它们存在是为了防止错误决策。            │
│ - 添加 license 声明。本项目仅作者和 AI 使用。                  │
│ - 无规范编码 (vibe coding)。必须先写规范再实现。               │
│ - 忽略测试。每个公共函数需要单元测试。                         │
│ - 一次修改 10+ 文件。分步修改, 每步验证。                      │
│                                                                 │
│ 必须: │
│ - 每个有意义步骤后 cargo clippy --deny warnings。 │
│ - 每个阶段后 cargo nextest --workspace。 │
│ - 保持 AGENTS.md 更新。 │
│ - 逐阶段优先于逐函数。 │
│ - 不确定时询问。不要编造 API。 │
│ - 技术版本见 ARCHITECTURE.md 技术版本锁定。 │
│ - 关键约束见上方 "禁止" 栏。 │
└─────────────────────────────────────────────────────────────────┘
```

## 构建命令

```bash
# Nix 开发环境 (推荐)
nix develop                    # 进入 devshell
nix develop --command cargo build --workspace

# 直接构建 (需已安装 Rust stable + cargo-nextest)
cargo build                    # Debug 构建
cargo nextest --workspace      # 全部测试
cargo clippy -- -D warnings    # 零警告
./scripts/quality-gate.sh      # 质量门 (全量)
```

完整命令见 `docs/DEVELOPMENT.md`。

## Nix DevShell

项目包含 `flake.nix`, 提供完整开发环境:

| 工具 | 版本 |
|------|------|
| Rust (fenix stable) | 1.95 |
| cargo-nextest | 0.9.136 |
| cargo-fuzz | 0.13 |
| cargo-geiger | 0.13 |
| cargo-audit | 0.22 |
| rust-analyzer | latest |

```bash
nix develop                           # 进入 shell
nix develop --command cargo nextest   # 直接运行
```

## 约定

### Rust

| 约定 | 规则 |
|------|------|
| Edition | 2024 |
| 格式化 | `cargo fmt` 强制 |
| Clippy | `--deny warnings` 每 PR 必需 |
| `unsafe` | `torvox-core` 中零。仅在 `torvox-gui-android` FFI 桥接和 `torvox-terminal::pty` 中。记录每个块。 |
| 错误处理 | `thiserror 2` + `eyre` 用于二进制。库 crate 中无 `anyhow`。 |
| 序列化 | `postcard 1.1` (不用 bincode, 已废弃) |
| 测试 | `cargo nextest` 替代 `cargo test`。内联单元测试。集成测试在 `torvox-integration-tests`。 |
| 属性测试 | `proptest 1.11` — VT 解析器和 CellGrid 必须有 |
| 命名 | 函数/变量 `snake_case`, 类型 `PascalCase`, 常量 `SCREAMING_SNAKE` |
| 导出 | 每个 crate 最小公共 API 表面。用 `pub(crate)` 隐藏内部。 |

### Kotlin

| 约定 | 规则 |
|------|------|
| Kotlin | 2.3.21+, Compose BOM 2026.05.01 |
| DI | Hilt |
| 架构 | MVVM with StateFlow/SharedFlow |
| UI | Jetpack Compose, Material 3 |
| 命名 | 函数/变量 `camelCase`, 类 `PascalCase` |
| 可空性 | 默认非空。`?` 仅在真正可空处。 |
| 格式化 | `ktfmt` 强制 |
| 渲染 | SurfaceView 宿主 Rust wgpu v29 Surface, 不用 Canvas |
| 前台服务 | `FOREGROUND_SERVICE_SPECIAL_USE` |

### Git

| 约定 | 规则 |
|------|------|
| 提交 | Conventional Commits (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`) |
| 分支 | `phase-N/*` 用于阶段工作, `fix/*` 用于修复 |
| PR | Squash merge 到 `main`。每个 PR 一个逻辑提交。 |

## 测试要求

完整策略见 `docs/ADR/006-testing-strategy.md`。摘要:

| 层 | 工具 | 频率 |
|----|------|------|
| Rust 单元+属性 | `cargo nextest` + `proptest` | 每 PR |
| Rust 模糊 | `cargo-fuzz` | 每夜 |
| Rust clippy/geiger/MIRI | 安全检查 | 每 PR/每夜 |
| Kotlin 单元+lint | `JUnit` + `ktlint` | 每 PR |

## 架构提醒

完整架构见 `docs/ARCHITECTURE.md`。关键点:

- **torvox-core 不分配**。它在 `no_std` 环境工作。
- **torvox-terminal 拥有所有 PTY I/O** 在专用线程中。
- **torvox-renderer 是单线程** (wgpu 设备在自己线程上)。
- **FFI 边界传递结构化事件**, 不是原始字节。

## AI 工作流

1. **阅读**: 本文件 → 当前阶段 ROADMAP.md → 相关 ADR
2. **规划**: 列出本次会话要完成的具体步骤
3. **类型先行**: 先定义类型, 再实现行为
4. **小步提交**: 每个逻辑步骤提交, 不积累 10+ 文件变更
5. **验证**: 每步 `cargo clippy --deny warnings && cargo nextest`
6. **更新**: 完成后更新 AGENTS.md 和相关文档

## 会话结束检查清单

每次工作会话后:

1. [ ] `cargo nextest --workspace` 通过
2. [ ] `cargo clippy --deny warnings` 通过
3. [ ] `./gradlew lint` 通过 (如有 Android 变更)
4. [ ] AGENTS.md 更新了新约定/工具
5. [ ] 重大决策创建了 ADR
6. [ ] 添加了新 crate? → 更新 ARCHITECTURE.md 和 Cargo.toml workspace
7. [ ] 添加了新 Android 模块? → 更新 settings.gradle.kts
8. [ ] 测试覆盖率: 新公共函数都有单元测试?
9. [ ] `cargo geiger` 检查: 无新的 unsafe 引入?
