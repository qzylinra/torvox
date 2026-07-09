# Torvox 项目全面审查报告

## 1. 报告概述

- **调查日期**: 2026-07-04
- **范围**: 全项目代码库、构建系统、CI/CD、测试基础设施、代码质量、文档
- **方法**: 三阶段深度审查——(1) 构建系统审查 (96项发现)、(2) 测试基础设施审查 (22项发现)、(3) 代码质量与文档审查 (31项发现)

三阶段总计约149项发现，覆盖 Rust/Kotlin 跨语言项目的全生命周期。

---

## 2. 关键发现汇总

### A. 构建系统问题

| 编号 | 问题 | 严重度 | 位置 |
|------|------|--------|------|
| C-01 | 18个GitHub Actions中17个使用 `@main`/`@master` 未固定版本 | **高风险** | 全部3个workflow文件 |
| C-07 | 无 Dependabot 配置，依赖无自动更新 | **高风险** | `.github/dependabot.yml` 缺失 |
| N-01 | `flake.nix:31-32` fenix overlay 重复定义（复制粘贴遗留） | **高风险** | `flake.nix` |
| X-01 | `CARGO_INCREMENTAL=0` 未穿透 nix develop（验证确认） | **中风险** | `rust-checks.yml → check-rust.nu` |
| X-04 | CI 缓存键不区分 host/交叉编译目标 | **中风险** | `rust-checks.yml:29-33` |
| CV-04 | 覆盖率基础设施完整配置但CI从未生成 | **中风险** | `codecov.yml` + `coverage-ratchet.toml` |
| S-01 | 脚本目录9个文件，AGENTS.md仅记载8个（`download-rapidocr-models.nu` 未记录） | **低风险** | `scripts/` |
| C-03 | release.yml 每日cron触发完整构建（~60分钟/天） | **低风险** | `release.yml:7` |
| A-05 | `gradle.properties` 仅1行，无缓存/并行/AndroidX配置 | **低风险** | `android/gradle.properties` |

### B. 测试基础设施问题

| 编号 | 问题 | 严重度 | 位置 |
|------|------|--------|------|
| TI-001 | Rust↔Kotlin 跨语言 wire format 无共享fixture测试，无版本鉴别器 | **高风险** | `bridge.rs` ↔ `WireCodecTest.kt` |
| TI-002 | GPU device loss 完全无处理和测试（Android常见场景） | **高风险** | `gpu.rs`, `surface.rs` |
| TI-003 | Session crash recovery 从未测试（mock PTY从不返回错误） | **高风险** | `session_state_machine.rs` |
| TI-004 | 全过程E2E测试缺失（FFI边界从未连通测试） | **高风险** | `bridge.rs` ↔ Kotlin |
| TI-019 | 50个 FFI 导出函数无 `catch_unwind`（同CQ-002，此处列为高风险） | **高风险** | `bridge.rs` |
| TI-005 | 10个 CSI proptest 仅断言"不崩溃"，无行为验证 | **中风险** | `proptest_csi.rs:410-567` |
| TI-009 | 代码覆盖率在CI中从未生成（`cargo llvm-cov` 从未调用） | **中风险** | `rust-checks.yml`, `check-rust.nu` |
| TI-012 | 35个 shuttle 并发测试需 nightly 且从未在 CI 运行 | **中风险** | `shuttle_concurrent.rs` |
| TI-018 | 插桩测试+Maestro E2E 仅在 release cron 运行，PR不触发 | **中风险** | CI 工作流矩阵 |
| TI-007 | TESTING.md 未反映实际测试覆盖（仅部分列举，缺少大量测试文件记录） | **中风险** | `docs/standards/TESTING.md` |
| TI-014 | 21个 Maestro E2E 流程完全未在 TESTING.md 记录 | **低风险** | `maestro/flows/` |

### C. 代码质量与文档问题

| 编号 | 问题 | 严重度 | 位置 |
|------|------|--------|------|
| CQ-001 | `unsafe impl Send+Sync for NativeWindow` 无 `// SAFETY:` 注释 | **严重** | `surface.rs:61-64` |
| CQ-002 | 50个FFI导出函数无 `catch_unwind`（同TI-019） | **严重** | `bridge.rs` |
| CQ-004 | 13处 `panic!` 位于 `Copy` trait 实现中（逻辑炸弹） | **严重** | `control.rs`, `event.rs`, `sgr.rs`, `vt_types.rs` |
| CQ-003 | 81个 unsafe 块中75个（93%）无 `// SAFETY:` 注释 | **高风险** | 全代码库 |
| CQ-005 | PTY fork 路径中3个 `expect()` 可 panic | **高风险** | `pty.rs:86,89,92` |
| CQ-017 | zh-rCN 翻译缺失31个键值，含搜索/光标/USB串口等关键功能 | **高风险** | `values-zh-rCN/strings.xml` |
| CQ-019 | 3处 Kotlin `catch (_: Exception)` 静默吞异常 | **高风险** | `TerminalScreen.kt`, `ToolbarPreferences.kt`, `SecondStageRunner.kt` |
| CQ-007 | `scroll_offset` 跨线程使用 `Relaxed` 内存序 | **中风险** | `bridge.rs:705` |
| CQ-009 | 17处 `CString::into_raw` 所有权文档化不足 | **中风险** | `bridge.rs` 多处 |
| CQ-015 | `pharaoh.toml` 中 `required_links` 为空（需求追踪未启用） | **中风险** | `pharaoh.toml` |
| CQ-016 | `torvox-mcp` 和 `torvox-exec` 无需求文档（MCP涉及文件系统访问） | **低风险** | `docs/requirements/` |
| CQ-014 | `docs/index.rst` 仅10行，缺少架构/API/贡献文档 | **中风险** | `docs/index.rst` |
| CQ-020 | Rust中25+处 `.ok()` / `let _ =` 静默丢弃错误 | **中风险** | `bridge.rs`, `pty.rs`, `session.rs` |

---

## 3. 未完成计划清单

| 位置 | 待办内容 | 状态 |
|------|----------|------|
| `pharaoh.toml:25` | `required_links` 空数组——TODO注释存在，未执行 pharaoh-setup | 未完成 |
| `TorvoxRuntime.kt:473` | `// TODO(U9): GPU pipeline sharing` | 未完成 |
| `proptest_csi.rs:410-567` | 10个占位符proptest（XXX标签）无行为断言 | 未完成 |
| `coverage-ratchet.toml` | 低阈值（core=45%, terminal=30%）需要提高 | 待优化 |
| `deny.toml` | cargo-deny 无配置文件，although工具已在devShell中 | 缺失 |
| CI 覆盖率 | codecov.yml + coverage-ratchet.toml 配置完毕但从未执行 | 未执行 |
| CI 基准测试 | `torvox-bench` 和 `gpu_benchmark.rs` 从未在CI运行 | 未执行 |
| Kani 形式化验证 | `torvox-core/kani` 元crate存在但 `cargo-kani` 未装 | 无效代码 |
| shuttle 并发测试 | 35个测试需要nightly，从未在CI执行 | 未执行 |

---

## 4. 风险分级

| 严重度 | 数量 | 主要问题 |
|--------|------|----------|
| **严重** | 3 | `unsafe impl Send+Sync` 无安全注释、FFI边界无 `catch_unwind`、`Copy` trait中 `panic!`（13处） |
| **高风险** | 19+ | 未固定Actions、Dependabot缺失、wire format漂移、GPU loss未处理、session崩溃恢复、i18n缺失31键、Kotlin静默吞异常、93% unsafe无SAFETY注释、PTY fork expect()、OCR flaky |
| **中风险** | 30+ | 10个占位proptest、Relaxed内存序、覆盖率未生成、需求追踪未启用、E2E测试缺失、无 deny.toml、缓存键混叠、CString所有权、错误静默丢弃等 |
| **低风险** | 18+ | 脚本数不匹配、GRADLE缓存未配置、每日cron浪费、kani无效、docs/index.rst 10行stub、zh/zh-rCN分裂等 |
| **信息** | 15+ | no_std合规、workspace成员数正确、lock ordering正确、thiserror使用正确、cargo machete干净等 |

---

## 5. 改进路线图

### 紧急（1-2天，高影响）
1. **固定GitHub Actions版本** — 所有18个action使用SHA或 `@v4` 标签，添加Dependabot
2. **补全 `// SAFETY:` 注释** — 75个unsafe块，优先 `NativeWindow`, `bridge.rs` FFI函数
3. **修复脚本计数偏差** — 在AGENTS.md中添加 `download-rapidocr-models.nu` 或移除孤立脚本
4. **修复缩写flags** — `-p`→`--package`, `-l`→长flag（`check-rust.nu`, `build-apk.nu`）
5. **优化 Gradle 配置** — 添加 `org.gradle.parallel=true`, `caching=true` 等

### 短期（1-2周）
6. **跨语言 wire format fixture 测试** — Rust 编码二进制→Kotlin解码，添加版本鉴别器
7. **GPU device loss 处理** — `SurfaceError::Lost` 恢复循环 + 模拟测试
8. **Session 崩溃恢复测试** — 扩展 `TestPty` 支持错误模式，测试 broken pipe/signal exit
9. **补全 zh-rCN 翻译** — 31个缺失键值，确保与English base同步
10. **添加 deny.toml + cargo-audit 到 CI** — 许可证/安全漏洞检查
11. **CI 启用覆盖率生成** — check-rust.nu 中添加 `cargo llvm-cov --lcov`，上传 Codecov

### 中期（1-3个月）
12. **FFI 边界添加 `catch_unwind`** — 50个导出函数，使用 `with_catch` 辅助宏
13. **修复 CSI proptest** — 添加光标位置/单元格内容/滚动量行为断言
14. **修复 `scroll_offset` 内存序** — `Release`/`Acquire` 替代 `Relaxed`
15. **添加 E2E 测试** — bridge→terminal→grid→渲染→Kotlin 全过程
16. **重写 TESTING.md** — 完整测试清单（82+ Rust文件 + 513 Kotlin单元 + 233插桩 + 21 Maestro）
17. **扩展文档** — `docs/index.rst` 架构文档、API参考、contributing guide
18. **添加 shuttle 并发测试到 CI** — 配置nightly jobs

---

## 6. 结论

Torvox 是一个架构设计优秀、技术选型现代的 GPU 加速 Android 终端模拟器。项目的核心数据层（`torvox-core`）实现了严格的 `no_std` + `forbid(unsafe_code)` 内存安全保证，workspace 分层清晰符合单向依赖原则，错误处理统一使用 `thiserror` 替代 `anyhow`。

**主要优势**:
- 清晰的6层单向依赖架构，静态验证通过
- `torvox-core` 零 unsafe，符合设计承诺
- 丰富的测试覆盖（513 Kotlin单元 + 233插桩 + 82 Rust测试 + 21 Maestro E2E + 16 Cucumber feature）
- 6种语言的 i18n 支持（中日韩法德英）
- CI 全自动化（Rust checks + Android build + emulator test）

**核心弱点**:
- **安全债务**：93% unsafe 块缺少安全注释、FFI 边界无 `catch_unwind`、`Copy` trait 中 `panic!` 构成逻辑炸弹
- **构建供应链风险**：全部 GitHub Actions 未固定版本、无 Dependabot、外部 kudzu 依赖
- **测试盲区**：跨语言 wire format 漂移、GPU device loss、session 崩溃恢复完全无测试覆盖
- **文档漂移**：TESTING.md 与实际严重脱节、AGENTS.md 脚本数不匹配、zh-rCN 翻译落后
- **CI 缺口**：覆盖率从未生成、基准测试不运行、shuttle 并发测试夜间阻塞

项目处于"功能完备但安全加固不足"的阶段。建议优先解决严重（CRITICAL）级别问题（3项），然后在1-2周内修复高风险供应链和跨语言测试问题，最后在1-3个月内系统性完善文档、内存序和测试基础设施。

**总体评分**: 架构设计 = A, 测试覆盖 = B, 代码安全 = D, 文档 = C, CI成熟度 = C
