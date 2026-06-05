# Torvox

<!-- AUDIT: 2026-06-04 — PTY code corrected (execvp→execve), env vars补充 TERM_PROGRAM_VERSION -->

Android 终端模拟器。Rust 引擎 + Kotlin UI。

---

# 一、架构

## Crate 依赖方向 (严格单向)

```
torvox-core (no_std, serde + unicode-width)  ← 数据模型、Grid、Cell、Event
    ↑
torvox-terminal (libghostty-vt + nix + flume) ← PTY、VT 解析、Session
    ↑
torvox-renderer (wgpu + cosmic-text + swash + guillotiere) ← GPU 渲染
    ↑
torvox-gui-android (boltffi)            ← Kotlin↔Rust 桥接
    ↑
torvox-app (Kotlin/Compose)             ← Android UI
```

## 目录结构

```
torvox/
├── torvox-core/src/          # lib.rs, cell.rs, grid.rs, line.rs, ansi.rs, config.rs, selection.rs, cursor.rs, unicode.rs, event.rs, snapshot.rs
├── torvox-terminal/src/      # lib.rs, pty.rs, ghostty_terminal.rs, keyboard.rs, session.rs
├── torvox-renderer/src/      # lib.rs, font.rs, gpu.rs
├── torvox-gui-android/src/   # lib.rs, bridge.rs (主要 FFI 导出), surface.rs, jni_bridge.rs
├── torvox-exec/src/          # W^X 多调用二进制
├── torvox-mcp/               # MCP 服务器 (JSON-RPC)
├── torvox-fuzz/              # 7 fuzz targets (3 active, 4 orphaned)
├── torvox-integration-tests/ # 78 集成测试
├── torvox-bench/             # 10 Criterion 基准
├── android/                  # Kotlin/Compose 应用
└── scripts/                  # quality-gate.nu, build-android-libs.nu
```

## 技术版本锁定

| 技术 | 版本 | 备注 |
|------|------|------|
| Rust | 1.95.0 (rust-toolchain.toml) | Edition 2024 |
| wgpu | 29 | Vulkan 后端; Surface 创建: `SurfaceTarget::Window` |
| cosmic-text | 0.19 | 文本成形, COLR/COLRv1 emoji |
| swash | 0.2.7 | 光栅化 (via zeno), scale feature 引入 skrifa |
| skrifa | 0.40 | Google 字体缩放库 |
| guillotière | 0.7 | 货架打包图集 |
| libghostty-vt | 0.1.1 (patch) | VT 解析器, channel-based |
| nix | 0.31 | Unix API (openpty, fork, ioctl) |
| flume | 0.12 | 无锁 SPSC 通道 |
| boltffi | 0.25 | 类型安全 Rust↔Kotlin 绑定 |
| rkyv | 0.8 | 会话持久化 |
| Kotlin | 2.3.21 | K2 编译器 |
| Compose BOM | 2026.05.00 | Material 3 |
| AGP | 9.0.1 | |
| Hilt | 2.59.2 | 依赖注入 |
| minSdk | 33 | Android 13 (Vulkan 1.3) |
| targetSdk / compileSdk | 36 | Android 16 |


## 已知风险

| 风险 | 缓解 |
|------|------|
| COLRv1 emoji | 捆绑 CBDT 字体回退 |
| Mali `glPushDebugGroup` SIGSEGV | `DISCARD_HAL_LABELS`, 优先 Vulkan |
| Android 16 前台服务 | `FOREGROUND_SERVICE_SPECIAL_USE` |
| wgpu 29 Surface API | Android/Vulkan: `SurfaceTarget::Window` |

## PTY 实现

`nix 0.31` crate (`openpty()` + `fork()`)。不用 `portable-pty`。

```rust
pub struct PtyPair {
    master: OwnedFd,
    child_pid: Pid,
}

impl PtyPair {
    pub fn spawn(shell: &str, rows: u16, cols: u16) -> Result<Self, PtyError> {
        let winsize = Winsize { ws_row: rows, ws_col: cols, ws_xpixel: 0, ws_ypixel: 0 };
        let pty = openpty(&winsize, None)?;
        let env = build_env(); // TERM, COLORTERM, TERM_PROGRAM, TERM_PROGRAM_VERSION
        match unsafe { fork() }? {
            ForkResult::Child => {
                setsid()?;
                ioctl(pty.slave, TIOCSCTTY, 0)?;
                close(pty.master)?;
                dup2(pty.slave, Stdio::stdin())?;
                dup2(pty.slave, Stdio::stdout())?;
                dup2(pty.slave, Stdio::stderr())?;
                if pty.slave > 2 { close(pty.slave)?; }
                configure_raw_mode(STDIN_FILENO);
                // execve with pre-built env (avoids malloc after fork)
                execve(shell, &[shell], &env)?;
                libc::_exit(1);
            }
            ForkResult::Parent { child } => {
                close(pty.slave)?;
                Ok(Self { master: pty.master, child_pid: child })
            }
        }
    }
}

impl Drop for PtyPair {
    fn drop(&mut self) {
        kill(self.child_pid, Signal::SIGHUP).ok();
        kill(self.child_pid, Signal::SIGCONT).ok();
        std::thread::sleep(Duration::from_millis(100));
        kill(self.child_pid, Signal::SIGKILL).ok();
        waitpid(self.child_pid, None).ok();
    }
}
```

**PTY 参数**:

| 参数 | 值 |
|------|-----|
| 接口 | `nix` 0.31 crate (`openpty()` + `fork()`) |
| 子进程 | `/system/bin/sh` (默认, 可配置) |
| Termios | Raw mode, UTF-8, 无流控, 无回显 |
| SIGWINCH | `ioctl(TIOCSWINSZ)` |
| W^X 变通 | `torvox-exec` 多调用二进制, `argv[0]` 的 `file_name()` 确定身份 |
| 进程组 | `setsid()` |
| kill 语义 | Drop 时 SIGHUP → SIGCONT → SIGKILL (递增式) |

**环境变量**: `TERM=torvox-direct`, `COLORTERM=truecolor`, `TERM_PROGRAM=torvox`, `TERM_PROGRAM_VERSION=<CARGO_PKG_VERSION>`

## 线程模型

```
PTY Reader (block read) → GhosttyTerminal (独占线程, flume channel) → Grid
Input Encoder ← Input Writer
Process Waiter
→ RenderThread (单线程, Condvar 唤醒) → wgpu v29 → Android SurfaceView
```

每会话 6-7 线程。热路径用 flume 不用 tokio。

## 渲染管线

```
PTY → flume → GhosttyTerminal → DirtyMask(Vec<u64>) → RenderThread
→ cosmic-text shape + swash/skrifa render → guillotiere pack
→ Atlas upload → Instance { position, uv, fg, bg, flags }
→ wgpu submission (1 draw call, 实例化四边形) → Android SurfaceView
```

- **cosmic-text**: 文本成形 + 字体回退 + BIDI
- **swash/skrifa**: 光栅化 + 彩色 emoji
- **guillotière**: 图集打包
- **DirtyMask**: Vec<u64> 分区位标志，每 u64 覆盖 64 行
- **实例化四边形**: 所有可见单元格打包到单个顶点缓冲区，1 次绘制调用

## 关键接口

```rust
pub struct Session {
    pty: PtyPair,
    terminal: GhosttyTerminal,
    output_rx: Receiver<Vec<u8>>,
    output_notify: Arc<(Mutex<bool>, Condvar)>,
    exited: Arc<AtomicBool>,
    reader_handle: Option<std::thread::JoinHandle<()>>,
    wait_handle: Option<std::thread::JoinHandle<()>>,
}
```

## 安全模型

1. PTY fork (`pty.rs`) 和 boltffi 桥接 (`bridge.rs`, `jni_bridge.rs`) 是主要 `unsafe` 边界
2. OSC 52 剪贴板: 事件类型已定义, 未接入
3. PTY 隔离: 每个会话独立进程组; `kill_on_drop`
4. 默认无网络: MCP 服务器仅限本地回环
5. 前台服务: `FOREGROUND_SERVICE_SPECIAL_USE`

---

# 二、技术规范

## 渲染规范

| 指标 | 目标 (模拟器) |
|------|------|
| 帧率 (空闲) | 0 FPS (无工作), CPU <0.5% |
| 帧率 (活跃 `find /`) | >60 FPS |
| 帧率 (突发 `cat /dev/zero`) | >30 FPS |
| 输入 → 像素 | <16ms P95 |
| 图集命中率 | >99% 稳态 |
| 空闲内存 | <10MB |
| 图集上限 | 64MB → LRU 驱逐 |
| GPU 驱动 | Adreno ✅, Mali ⚠️ (`DISCARD_HAL_LABELS`), PowerVR ❌ (<1%) |

字体管线: fontdb → cosmic-text → swash + skrifa → guillotière
捆绑字体: `JetBrainsMono-Nerd-Font.ttf`
成形 feature: `cursive`, `kern`, `liga`, `dlig`, `rlig`, `calt`
图集: 2048×2048 或 4096×4096, 货架打包, LRU 驱逐

## 性能目标 (模拟器)

| 指标 | 目标 |
|------|------|
| 冷启动 → shell | <2s |
| PTY 输入 → 回显 | <16ms P95 |
| 空闲 CPU | <1% |
| 空闲内存 | <10MB |

## 目标平台

minSdk 33 (Android 13), Vulkan 1.3, arm64-v8a + x86_64, targetSdk/compileSdk 36

## 会话持久化

| 数据 | 格式 | 位置 | 同步 |
|------|------|------|------|
| 终端状态 | rkyv 0.8 | `app/session_{id}.bin` | 每 60s + 前台丢失时 |
| 配置 | DataStore Preferences | Android DataStore | 变更时 |

---

# 三、工作流

## 环境

```bash
nix develop                           # 进入开发环境
cargo build && cargo nextest --workspace
cd android && ./gradlew assembleDebug
```

## 构建流程

```
[Rust 源码] → cargo ndk v4 → [aarch64-linux-android / x86_64-linux-android]
→ libtorvox_android.so → APK 打包 → System.loadLibrary("torvox_android")
```

## 测试

```bash
cargo nextest --workspace                              # 所有 Rust 测试
QUICKCHECK_TESTS=10000 cargo test --workspace          # 属性测试
cargo fuzz run --fuzz-dir torvox-fuzz/fuzz fuzz_vt_parser -- -max_total_time=3600
cargo geiger --all-features           # unsafe 统计 (torvox-core 应为零)
cargo miri test -p torvox-core        # 未定义行为检测
```

## 质量门

```bash
# 快速 (~5min)
cargo fmt --check && cargo clippy --deny warnings && cargo nextest --workspace

# 完整 (~15min)
scripts/quality-gate.nu
# = fmt → no_std build → clippy → test → mutants → kani → audit → geiger → machete → markdownlint → Android lint → Roborazzi → APK build
```

## 性能分析

```bash
adb shell dumpsys gfxinfo io.torvox    # 帧时间
adb shell dumpsys meminfo io.torvox    # 内存
cargo bench                            # 基准测试
```

## 提交规范

```
<type>(<scope>): <description>
```
类型: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`
规则: 原子提交、代码+文档同步、提交前 clippy+tests 通过、不混格式化变更。

## AI 工作流

1. 读 ROADMAP 当前阶段 + 相关章节
2. 规划 → 类型先行 → 小步提交 → 验证 (clippy+nextest) → 同步 (Rust→bridge.rs→Kotlin) → 更新文档

**反模式 (严格禁止)**: 无规范编码、忽略测试、批量修改 10+ 文件、抽象泄漏 (引用不存在的模块)

## 规范驱动开发 (SDD)

规范是唯一真相来源，代码必须符合规范。
工作流: 定义规范 → 生成代码 → 验证对齐 → 更新规范。
代码与规范冲突时以规范为准；代码变更后必须同步更新规范；每个验收标准必须可测试。

---

# 四、测试

## 测试金字塔

```
L4: 模糊测试 (cargo-fuzz)            ← 夜间 CI, 1B+ 迭代
L3: 集成测试 (78 tests), 变异测试 (≥70%), Kani 形式化验证 ← 每 PR
L2: 属性测试 (quickcheck 23 tests)  ← 每 PR, 10K+ 案例
L1: 单元测试 (437 Rust, 148 Kotlin) ← 每 PR
L0: 编译时 (clippy, geiger, MIRI, fmt) ← 每次构建
```

## 测试最佳实践

### 核心原则

1. **测试即规范**: 测试就是代码行为的规范。没有测试 = 没有规范。
2. **只测试公共 API**: 测试内部实现会将测试与实现耦合。
3. **修复实现，而非测试**: 永远不要为通过而削弱测试。
4. **一个测试 = 一个行为**: 每个测试验证一个不变量。
5. **杜绝不稳定测试**: 时序依赖用确定性同步机制。

### 命名规范

- ✅ `vt_write_cjk_then_snapshot_includes_char`
- ❌ `test_1`
- ✅ `grid_resize_preserves_dims`
- ❌ `resize_test`
- Kotlin: `fun \`shell exits when user types exit command\`()` { ... }

### 反模式

| 反模式 | 说明 |
|--------|------|
| 同义反复 | `assert!(x == 5)` — 断言在构造上恒为真 |
| 防御性测试 | `assert!(s.len() > 0)` — 测试语言本身 |
| 顺序依赖 | 测试依赖前一个测试的状态 |
| sleep 测试 | `thread::sleep` — 用通道/AtomicBool 代替 |
| 被忽略的测试 | `#[ignore = "TODO"]` — 重要就修复, 否则删除 |
| 脆弱快照 | 5000 行 diff 不是测试失败, 是代码审查 |

### 何时删除测试

- 被测行为不再需要
- 测试是同义反复
- 测试不稳定且无法修复
- 测试是已完成 TODO 的占位符
- 测试被更好的测试重复覆盖

**永远不要**因为测试失败而删除它。修复它，或修复代码。

---

# 五、编码规范

## Shell 脚本

所有 shell 脚本使用 **Nushell** (`.nu`)。禁止 bash/sh。
规范: `#!/usr/bin/env nu`, 所有外部命令检查退出码, `|` 在行尾, `snake_case`, `$env.VAR = "val"`, `^command` 调用。

## Nix 环境

所有环境管理使用 **Nix**。禁止系统 shell 直接构建。
规范: 始终 `nix develop`, `nix develop --command cargo build`, `nix fmt`, `nix flake check`。

## GitHub Actions

- Action 版本: 默认分支 (`@main`/`@master`), 不用标签
- Step `name`: 不设置
- `run` 合并: 相邻步骤合并为多行块
- `||` 禁止
- Job 命名: 短横线 (`rust-checks`)
- 权限: 显式声明 `permissions:`

## Nix 表达式

- 不用 `let in` / `rec`
- 限制 `with`
- 不缩写变量名
- `flake.nix`: 不添加 `description`/`shellHook`; `checks` 移到 nushell 脚本; `formatter` 用 `pkgs.nixfmt-tree.override`

## 通用

- 变量命名: 不缩写单词
- 中间变量: 尽可能 inline
- 每个主题只维护一份文档，避免重复

---

# 六、行为准则

## 1. 先理解再动手

**不确定就问。有多种方案就全部列出。**

在实现之前:
- 写出你的假设。不确定的部分直接问。
- 有多种解读时，列出每种的优劣，不要默默选一种。

**禁止跳过**:
- 写代码前没读 ROADMAP 当前阶段
- 改 crate 前没读本文件相关章节
- 引入新依赖前没查技术版本锁定

## 2. 简单优先

**只写解决问题所需的代码。不写投机性代码。**

- 不实现未被要求的功能。不为单次使用的代码创建抽象。
- 不添加未被请求的"灵活性"。不为不可能发生的场景编写错误处理。
- 如果你写了 200 行而其实 50 行就够了，重写。

**具体到本项目**:
- `torvox-core` 是 `no_std` — 不要引入需要 `alloc` 的功能除非绝对必要
- 不要提前实现未来阶段的骨架 — ROADMAP 说了何时做

## 3. 精确修改

**只动必须动的。只清自己造成的混乱。**

- 不"改进"相邻代码、注释、或格式。匹配已有风格。
- 修改 `torvox-core` 类型时，同步更新 `bridge.rs` 的桥接类型
- 修改 Cargo.toml 依赖版本时，确认与本文件技术版本锁定一致

## 4. 每步验证

1. 写代码 → `cargo clippy -- -D warnings` 零警告
2. 写测试 → `cargo nextest -p <crate>` 通过
3. 提交前 → `cargo nextest --workspace` 全量通过
4. 完成阶段 → `scripts/quality-gate.nu` 通过

## 5. 不确定时停下

**编造 API 比承认不知道更危险。** 文档冲突、API 可见性、crate 归属、依赖审批 — 停下来问。

---

# 七、约束与已知陷阱

## 绝对禁止

```
【语言与依赖】
✗ 添加 Java 文件 — 仅 Kotlin
✗ 依赖 Termux — 独立项目
✗ 使用 portable-pty — 用 nix 0.31 openpty()+fork()
✗ 使用 bincode — 用 rkyv 0.8
✗ 使用 rust-android-gradle — 用 scripts/build-android-libs.nu
✗ 在库 crate 中使用 anyhow — 用 thiserror 2
✗ boltffi Error 用 `message` 字段 — 改用 `detail`

【架构与安全】
✗ 在 torvox-core 中添加 `unsafe` — 零 unsafe crate
✗ 多 crate 使用 setup_scaffolding!() — 只允许一个
✗ 使用 Canvas.drawText 逐单元格 — 仅 GPU 渲染
✗ FFI 传递原始字节 — 传递结构化事件
✗ 用 /proc/self/exe — 用 argv[0] file_name()

【开发流程】
✗ 无规范编码 — 必须先写规范 (SDD)
✗ 忽略测试 — 每个公共函数需要单元测试
✗ 一次修改 10+ 文件 — 分步修改
```

## 已知陷阱

| # | 陷阱 | 教训 |
|---|------|------|
| 1 | `Shell::Custom(u8)` | u8 太小 → `String`，失去 Copy |
| 2 | `DirtyLine` 枚举 | 改为 `DirtyMask { partitions: Vec<u64> }` |
| 3 | thiserror 2.x + no_std | 设为 optional，std feature 启用 |
| 4 | boltffi 多 crate 导出 | 只允许一个导出位置 |
| 5 | boltffi message 字段 | 与 Kotlin Throwable.message 冲突 |
| 6 | cargo-ndk 仅 cdylib | torvox-exec 用 CARGO_TARGET_*_LINKER |
| 7 | boltffi CLI 不生成桥接 | 改用 JNA 手动绑定 (TorvoxBridge.kt) |
| 8 | libghostty-vt API | `scrollback_rows()` 非 `history_size()`; `resize(rows, cols)` 两参数 |
| 9 | jniLibs 缺失 | Gradle 不会自动编译 Rust; `cargo ndk` 后需手动复制 .so 到 `android/app/src/main/jniLibs/{x86_64,arm64-v8a}/` |
| 10 | APK 缺 .so 不报构建错 | `./gradlew assembleDebug` 成功但 native lib 缺失时 APK 仅 kotlin 字节码; 运行时 `UnsatisfiedLinkError` |
| 11 | libghostty-rs 需手动 clone | `git clone --depth 1 https://github.com/Uzaaft/libghostty-rs.git` 且需 Zig 0.15 编译 Ghostty C 库 |
| 12 | Zig 版本必须 0.15.x | Ghostty build.zig 检查版本; 0.16 不兼容; 通过 `nix shell nixpkgs#zig_0_15` 获取 |

---

# 八、提交检查清单

**开发阶段**:
- [ ] 类型在哪个 crate? → 是否通过 boltffi 导出? → bridge.rs 同步?
- [ ] Kotlin 绑定重新生成? → serde 格式破坏? → 测试更新?
- [ ] 是否需要更新 AGENTS.md / ROADMAP.md?

**验证命令**:
1. [ ] `cargo nextest --workspace` 通过
2. [ ] `cargo clippy -- -D warnings` 通过
3. [ ] `cargo fmt --check` 通过
4. [ ] `cargo geiger`: 无新 unsafe (torvox-core)
5. [ ] 新公共函数有单元测试?
6. [ ] `QUICKCHECK_TESTS=10000 cargo nextest run -p torvox-core --test property_tests` 通过
7. [ ] `cargo mutants --timeout 120` 突变分数≥70%
8. [ ] `cargo kani --manifest-path torvox-core/kani/Cargo.toml` 形式化验证通过
9. [ ] (如有 Android) `./gradlew lint` + `testDebugUnitTest` + `roborazziDebug` 通过
10. [ ] (如有 Maestro) `maestro test maestro/` E2E 测试通过
11. [ ] (如有 bridge 变更) boltffi 绑定已重新生成
12. [ ] AGENTS.md 已更新

---

# 附录: 术语表

| 术语 | 含义 |
|------|------|
| SDD | 规范驱动开发 |
| VT | Video Terminal — VT100/220/ECMA-48 |
| PTY | Pseudo-Terminal |
| W^X | Write XOR Execute — Android 安全策略 |
| SGR | Select Graphic Rendition |
| CSI | Control Sequence Introducer |
| OSC | Operating System Command |
| DirtyMask | Vec<u64> 分区位标志 |
| SPA | Single Point of Authority |

# 附录: 关键代码文件

| 文件 | 用途 |
|------|------|
| `torvox-core/src/cell.rs` | Cell, Attrs, Color, DirtyMask (`no_std`) |
| `torvox-core/src/grid.rs` | Grid, Scrollback (VecDeque) |
| `torvox-terminal/src/pty.rs` | PtyPair — 唯一允许 fork unsafe |
| `torvox-terminal/src/session.rs` | Session 编排器 (线程+通道) |
| `torvox-gui-android/src/bridge.rs` | boltffi 导出 — 唯一导出位置 |
| `scripts/quality-gate.nu` | 13 步质量门 |
| `docs/ROADMAP.md` | 当前阶段、里程碑、退出标准 |
| `docs/MCP.md` | MCP 服务器规范 |
