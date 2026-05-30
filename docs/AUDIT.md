# Torvox 审计跟踪 — 待确认问题

> 本文件记录 2026-05-30 全项目审计中**未确认**或**需要更多上下文**的问题。
> 已确认并已修复的问题已直接修复到代码中，不在此记录。
> 生成日期: 2026-05-30

---

## U1 — Android SurfaceView 未接入 Rust GPU Surface、PTY 或输入管线

**严重性**: 高  
**状态**: 待确认（需架构决策）  
**证据**: `TerminalSurface.surfaceCreated()` 和 `surfaceChanged()` 为空；没有创建 `TorvoxBridge`/`AndroidSurface`/native window/render thread/PTY session。`TerminalScreen` 仅创建 `TerminalSurface`，设置固定 24×80，ModifierBar 输入停留在 `pendingInput`，没有发送到 Rust/PTY。`TerminalViewModel.extractSelectedText()` 返回占位字符串。`TerminalSurface.consumePendingInput()` 没有调用点。

**影响**:
- P1.5 "Android Surface 渲染完成" 和 P1.6 "输入处理完成" 的状态不成立
- APK 显示 Compose 壳，但不会显示 shell 提示符，也不会回显命令
- UI 测试只验证 Activity/ContentView 存在，无法发现真实渲染缺失

**待决策**:
- 是否需要 `TerminalSessionController`/`TorvoxRuntime` 作为 Android native runtime 对象？
- surface lifecycle → Rust bridge 如何最小闭环？
- 是先做最小端到端闭环，还是先把所有 P2.4-P3.3 做完再统一接线？

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
**状态**: 待确认（需要测试验证兼容性）
**证据**:
- `boltffi 0.25` → `0.25.1` (patch)
- `flume 0.11` → `0.12.0` (minor)
- `Compose BOM 2026.05.00` → `2026.05.01` (patch)
- `JNA 5.17.0` → `5.18.1` (minor)
- `AndroidX runner/rules 1.6.2` → `1.7.0` (minor)
- `AGP 9.0.1` → `9.2.1` (minor)

**待决策**:
- 是否现在升级所有依赖？还是等下一个里程碑？
- flume 0.12 是否有 breaking changes？是否需要更新 API？

---

## U7 — 会话生命周期测试偶发失败

**严重性**: 低  
**状态**: 待确认（需要复现和根因分析）
**证据**: `cargo test --workspace` 复跑时出现 `session_spawn_and_write`/`session_spawn_and_exit` 偶发 `write failed: Closed`，说明 PTY session 测试和生命周期存在竞态或环境敏感性。

**待决策**:
- 是否需要添加测试隔离（每个测试独立临时目录）？
- 是否需要添加重试逻辑？
- 是否需要固定 shell 路径？
