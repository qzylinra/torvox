# Torvox 项目审查 — 发现的问题

> 审查日期: 2026-05-27
> 审查范围: 全部文档 + 源码 + CI + 配置

---

## 1. 源码问题

### 1.1 关键 (Critical)

| # | 位置 | 问题 | 建议 |
|---|------|------|------|
| C1 | `torvox-core/src/grid.rs` DirtyLine | `DirtyLine(u64)` 仅支持 0-63 行。行号 >= 64 时移位越界 (debug panic, release UB) | 使用 `BitVec` 或 `Vec<u64>` 支持任意行数 |
| C2 | `torvox-core/Cargo.toml` | `thiserror = { workspace = true }` 在 `no_std` 模式下无法编译 (thiserror 2.x 需要 std) | 使用 `thiserror-nostd` 或条件依赖 |
| C3 | `torvox-bridge-types/src/lib.rs` + `torvox-gui-android/src/lib.rs` | 两处调用 `uniffi::setup_scaffolding!()` 会导致链接时重复符号错误 | 仅在最终二进制 crate 调用一次 |
| C4 | `torvox-terminal/src/pty.rs` | `Pty::open()` 始终返回 Err，结构体无法实例化。但 Drop 实现 `kill(pid, SIGHUP)` 对 pid=0 会发信号给整个进程组 | 添加 `pid > 0` 守卫 |

### 1.2 重要 (Major)

| # | 位置 | 问题 | 建议 |
|---|------|------|------|
| M1 | `torvox-core/src/cell.rs` | `Cell` 缺少 `PartialEq, Eq` derive，无法比较。`default_wide()` 名字暗示 width=2 但实际 width=1 | 添加 derive；重命名为 `default_single()` 或修正 width |
| M2 | `torvox-core/src/ansi.rs` | `SgrAttribute` 缺少 `Serialize, Deserialize` derive，与其他类型不一致 | 添加 derive |
| M3 | `torvox-core/src/unicode.rs` | `unicode_width()` 粗略近似，不支持零宽字符 (ZWJ, VS)、组合标记、East Asian Ambiguous | 使用 `unicode-width` crate |
| M4 | `torvox-terminal/src/osc.rs` | OSC 8 解析不完整：未提取 `id=KEY:` 部分。OSC 52 未解析剪贴板名和 base64。`Unknown(u64, String)` 第一个字段始终为 0 | 修正解析逻辑 |
| M5 | `torvox-exec/src/main.rs` | `Command::new(other).exec()` 不传递命令行参数，实际使用时所有参数会丢失 | 添加 `.args(env::args().skip(1))` |
| M6 | 多个 Cargo.toml | 死依赖：torvox-terminal (crossbeam/postcard/serde 未用)、torvox-renderer (cosmic-text/swash/etagere/glyphon 未用)、torvox-gui-android (torvox-core/tokio/thiserror 未用)、torvox-bridge-types (torvox-core 未用)、torvox-exec (nix 未用) | 移除未使用的依赖 |

### 1.3 次要 (Minor)

| # | 位置 | 问题 | 建议 |
|---|------|------|------|
| m1 | `torvox-core/src/cell.rs` | `Cell` 缺少 `Copy` derive (所有字段均为 Copy) | 添加 Copy |
| m2 | `torvox-core/src/selection.rs` | `Selection` 缺少 `PartialEq` derive | 添加 derive |
| m3 | `torvox-core/src/config.rs` | `FontConfig` 缺少 Default 实现。`FontConfig.size` 与 `RenderConfig.font_size` 重复 | 添加 Default；统一 font_size |
| m4 | `torvox-core/src/event.rs` | `ClipboardRequest(String)` 语义不清 — 是剪贴板名还是内容？ | 改为 `ClipboardRequest { name: String, content: String }` |
| m5 | `torvox-terminal/src/grid.rs` | `#[allow(dead_code)]` 遮盖所有未实现方法的警告 | 逐字段控制 |

## 2. CI/工作流问题

| # | 位置 | 问题 | 建议 |
|---|------|------|------|
| CI1 | `.github/workflows/ci.yml` | `android-checks` job 引用 `android/` 目录，但仓库中不存在该目录 | 移除或标记为条件跳过 |
| CI2 | `.github/workflows/ci.yml` | `java-version: 25` 不存在 | 改为 `21` (LTS) |
| CI3 | `.github/workflows/nightly.yml` | fuzz matrix 列 `[vt_parser, osc_parser, utf8_parser]`，实际 target 名为 `fuzz_vt_parser`, `fuzz_osc_parse`, `fuzz_grid_resize` | 修正 matrix 名称 |
| CI4 | `.github/workflows/nightly.yml` | `cargo fuzz run` 需指定 `--fuzz-dir torvox-fuzz` | 添加 `--fuzz-dir` |
| CI5 | `.github/workflows/nightly.yml` | MIRI `-Zmiri-tag-raw-pointers` 已弃用 | 改为 `-Zmiri-strict-provenance` |
| CI6 | `.github/workflows/release.yml` | `cargo ndk -t x86_64-linux-android` 应为 `x86_64` (NDK ABI 名) | 修正 target 名 |
| CI7 | 所有 workflow | 使用 `@main`/`@master` 可变引用而非 SHA pin，有供应链风险 | Pin 到具体 SHA |
| CI8 | `.github/workflows/release.yml` | 无测试步骤，直接构建发布 | 添加测试门 |

## 3. 配置问题

| # | 位置 | 问题 | 建议 |
|---|------|------|------|
| O1 | `.opencode/opencode.jsonc` | `$schema` URL 使用 `opencode.ai/config.json`，需验证是否为有效 schema | 验证并修正 |
| O2 | `.opencode/opencode.jsonc` | formatter `shfmt` 扩展使用 `*.sh` 但 opencode formatter 扩展格式应为 `.sh` (无通配符) | 检查格式 |
| O3 | `.opencode/opencode.jsonc` | MCP `mcp-server-filesystem` 无 path 参数，可能无法访问项目目录 | 添加路径参数 |
| O4 | `.opencode/opencode.jsonc` | ADR 005 提到 `.opencode/config.jsonc` 但实际文件名为 `opencode.jsonc` | 统一引用 |
| O5 | `.opencode/opencode.jsonc` | agents 缺少 `torvox-ci` 的 files 中应包含的 `rust-toolchain.toml` | 添加 |

## 4. 文档不一致

| # | 位置 | 问题 |
|---|------|------|
| D1 | ROADMAP P0.1 步骤 5 | 列出 `skrifa 0.42` 作为 torvox-renderer 依赖，但 Cargo.toml 和 ADR 003 明确说明 swash 0.2.x 内部已包含 skrifa，无需单独声明 |
| D2 | ADR 005 §opencode 配置 | 提到 `.opencode/config.jsonc` 但实际文件名是 `.opencode/opencode.jsonc` |
| D3 | ADR 005 §配置文件层次 | 列出 `CLAUDE.md` 但仓库中不存在该文件 |
| D4 | DEVELOPMENT.md §Android SDK | NDK 路径写 `ndk/28.x.x` 但版本锁定表写 NDK r29 |
| D5 | ARCHITECTURE.md §Crate 架构 | 列出 `torvox-terminal/tests/` 目录，但实际测试在 `torvox-integration-tests/` |
