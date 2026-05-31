# Torvox 审计跟踪 — 待确认问题

> 本文件记录 2026-05-30 全项目审计中**未确认**或**需要更多上下文**的问题。
> 已确认并已修复的问题已直接修复到代码中，不在此记录。
> 生成日期: 2026-05-30

---

## U1 — Android SurfaceView 未接入 Rust GPU Surface、PTY 或输入管线

**严重性**: 高  
**状态**: 已修复（核心接线完成）  
**证据**: 
- `TorvoxRuntime.kt` 创建 `TorvoxBridge`，调用 `setNativeWindow`, `render`, `resize`
- `TerminalSurface.surfaceCreated()` → `vm.runtime.start()`
- `TerminalSurface.surfaceChanged()` → `vm.runtime.resize()`
- `writeToPty()` 现已通过 `runtime.writeToPty()` 接通到 PTY
- `extractSelectedText()` 仍返回占位字符串（非阻塞）

**修复方案**: 
- `TorvoxRuntime` 作为统一控制器管理 Surface→PTY→render 生命周期
- `createBridge(config)` 使用 boltffi JNA 桥接

---

## U2 — CI/Release 没有构建 torvox-exec，也没有重新生成/校验 boltffi Kotlin 绑定

**严重性**: 中  
**状态**: 待确认（需验证 CI 实际行为）
**证据**: release workflow 只用 `cargo ndk` 构建 `torvox-gui-android` cdylib，没有运行 `scripts/build-android-libs.nu`，没有为 `torvox-exec` 生成 assets。CI 不校验 Kotlin 绑定是否由当前 `bridge.rs` 生成。

**影响**:
- W^X 多调用二进制可能缺失，真实 PTY shell 启动可能失败
- Rust FFI 类型变更后 Kotlin 绑定可陈旧

**待决策**:
- CI/release 是否统一调用 `scripts/build-android-libs.nu`？
- 是否新增 "generate then diff" 绑定校验步骤？

---

## U3 — Unicode 宽度实现不适合作为终端长期基础

**严重性**: 中  
**状态**: 待确认（需要长期策略）
**证据**: `torvox-core/src/unicode.rs` 手写 East Asian width 范围，覆盖有限：emoji ZJW 序列、variation selector、regional indicator、Unicode 版本更新、ambiguous width 策略都没有正式处理。

**待决策**:
- 是否引入 `unicode-width` crate（注意 no_std/alloc）？
- 是否定义 width policy：CJK ambiguous width、emoji presentation、combining cluster？
- Grid 写入是否从 char-based 升级到 grapheme/cluster-aware？

---

## U4 — Grid 性能问题

**严重性**: 中  
**状态**: 待确认（需要 benchmark 验证）
**证据**:
- `Grid::get_mut(row)` 先标脏再检查 row 是否存在
- `Line` 是 `Vec<Cell>`，每行单独分配，对 120 FPS 热路径可能有 cache miss
- scrollback 使用 `lines.remove(0)`/`insert(...)`，50k 行上限下应考虑 ring buffer

**待决策**:
- 是否改用 flat `Vec<Cell>` 或 gap/ring layout？
- scrollback 是否改用 ring buffer？

---

## U5 — Renderer 不是生产级

**严重性**: 中  
**状态**: 待确认（需要实际 Android 设备验证）
**证据**:
- Android surface 初始配置写死 1080×1920，没有公开 `resize_surface` 接口
- `render_frame()` 每帧 clear 全屏并为全部 visible cells build instances，DirtyMask 没用于减少 CPU build
- `FontPipeline::new()` 对负数没有防御，`(atlas_width * atlas_height * 4) as usize` 可能溢出
- glyph cache key 只有 glyph_id + pixel_size，没有 font_id/style/subpixel/variation
- atlas 满时没有 LRU 驱逐或 atlas re-pack
- `upload_atlas()` 似乎只在 surface 设置时上传一次，运行时 rasterize 新 glyph 后没有增量 upload

**待决策**:
- 是否需要 LRU 驱逐策略？
- 是否需要增量 atlas upload？
- 是否需要 resize_surface 接口？

---

## U6 — 依赖更新

**严重性**: 低  
**状态**: 部分完成  
**证据**:
- `boltffi 0.25` → `0.25.1` ✅ 已升级
- `flume 0.11` → `0.12.0` ✅ 已升级
- `Compose BOM 2026.05.00` → `2026.05.01` — 待验证
- `JNA 5.17.0` → `5.18.1` — 待验证
- `AGP 9.0.1` → `9.2.1` — 待验证

**待决策**:
- 是否现在升级剩余依赖？还是等下一个里程碑？

---

## U7 — 会话生命周期测试偶发失败

**严重性**: 低  
**状态**: 待确认（需要复现和根因分析）
**证据**: `cargo test --workspace` 复跑时出现 `session_spawn_and_write`/`session_spawn_and_exit` 偶发 `write failed: Closed`，说明 PTY session 测试和生命周期存在竞态或环境敏感性。

**待决策**:
- 是否需要添加测试隔离（每个测试独立临时目录）？
- 是否需要添加重试逻辑？
- 是否需要固定 shell 路径？

---

## U8 — Kotlin 绑定由 UniFFI 生成，未用 boltffi 重新生成

**严重性**: 高  
**状态**: 已修复  
**证据**: 
- `bridge.rs` 使用 `#[boltffi::data]`, `#[boltffi::export]`, `#[boltffi::error]` 宏
- `Cargo.toml` 依赖 `boltffi = "0.25"`
- 旧 `torvox_android.kt` 由 UniFFI 生成，与 boltffi .so 不兼容
- **修复**: 删除旧 UniFFI 绑定，创建 `TorvoxBridge.kt` 使用 JNA 直接调用 boltffi C ABI
- 发现 `#[boltffi::export]` 在 impl 块上要求方法为 `pub` 才能生成 FFI 导出
- 所有 18 个方法现在通过 boltffi C ABI 导出

**修复方案**: 
1. `bridge.rs` 所有方法改为 `pub`
2. 新 `TorvoxBridge.kt`: 自包含 JNA 桥接，使用 boltffi wire format
3. `writeToPty()` 现已接通到实际 PTY

---

## 待决策问题 (2026-05-30 审计)

### Q1: fontdb 版本不一致
**已解决**: Cargo.toml 和 ARCHITECTURE.md 都是 0.23，保持不变。

### Q2: flume 版本不一致
**已解决**: Cargo.toml 升级到 0.12，与 ARCHITECTURE.md 一致。

### Q3: boltffi vs UniFFI 绑定重新生成
**已解决**: 用 boltffi JNA 桥接替代 UniFFI (见 U8)。

### Q4: CI Actions 引用 @main
- A: 固定到 v4 tag + Dependabot
- B: 固定到 commit SHA
- C: 保持 @main

### Q5: no_std 构建加入 CI
- A: 加入 CI (~30s)
- B: 仅本地验证

### Q6: 子 crate workspace 继承
- A: 使用 version.workspace = true 等 (Cargo 最佳实践)
- B: 保持当前重复定义

### Q7: torvox-exec 加入 CI release
- A: 加入构建 (W^X 方案完整)
- B: 保持当前

### Q8: 是否创建 TorvoxRuntime 控制器
- A: 创建统一 runtime (4-8h，解决端到端断裂)
- B: 逐步接线 (每次改动小)
