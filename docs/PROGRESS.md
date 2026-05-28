# Torvox 项目进度报告

> 生成时间: 2026-05-29 (深度审计更新)
> 代码行数: ~5,800 行 Rust
> 测试数量: 220 个 (78 core + 7 android + 10 renderer + 125 terminal)
> Clippy 状态: 零警告

---

## 一、当前进度总览

### 阶段 0: 基础设施 — ✅ 全部完成

| 里程碑 | 交付物 | 状态 | 测试数 |
|--------|--------|------|--------|
| P0.1 | Rust workspace 8 个 crate | ✅ | 0 |
| P0.2 | `torvox-core` 完整类型系统 (9 模块, no_std) | ✅ | 74 |
| P0.3 | Android 项目 + Kotlin + Hilt + Compose | ✅ | 0 |
| P0.4 | 文档 + CI + 质量门 | ✅ | 0 |
| P0.5 | PtyPair + W^X 多调用二进制 | ✅ | 4 |
| P0.6 | UniFFI 桥接验证 | ✅ | 7 |

### 阶段 1: 终端引擎 — 5/6 完成

| 里程碑 | 交付物 | 状态 | 测试数 |
|--------|--------|------|--------|
| P1.1 | VT 解析器 (vte 0.15) | ✅ | 76 (含 3 proptest) |
| P1.2 | PTY 会话集成 (crossbeam) | ✅ | 5 |
| P1.3 | 字体管线 (fontdb+cosmic-text+swash+etagere) | ✅ | 7 |
| P1.4 | GPU 渲染管线 (wgpu v29) | ✅ | 3 |
| P1.5 | Android Surface 渲染 | ⬜ | 0 |
| P1.6 | 输入处理 (Kitty 协议 + VT 传统编码) | ✅ | 43 |

---

## 二、测试如何保证正确性

### 2.1 测试金字塔

```
L0 编译时: cargo clippy --deny warnings (每次构建)
L1 单元测试: 214 个 #[test] (每个公共函数)
L2 属性测试: proptest 10K+ 用例 (VT 解析器、Grid、DirtyMask)
L3 集成测试: 5 个 Session 测试 (PTY 读写、echo、resize、exit)
L4 模糊测试: 待实现 (torvox-fuzz 空)
```

### 2.2 各模块测试覆盖

| 模块 | 测试数 | 覆盖内容 |
|------|--------|----------|
| `torvox-core/cell.rs` | 12 | Cell/Attrs/Color/DirtyMask 序列化、操作 |
| `torvox-core/grid.rs` | 12 | Grid 创建、标记、清空、resize |
| `torvox-core/line.rs` | 10 | Line 创建、访问、resize |
| `torvox-core/cursor.rs` | 8 | CursorState 移动、边界检查 |
| `torvox-core/ansi.rs` | 6 | ANSI 调色板、SGR 属性序列化 |
| `torvox-core/event.rs` | 6 | TerminalEvent 序列化 |
| `torvox-core/unicode.rs` | 10 | Unicode 宽度、CJK 检测 |
| `torvox-core/selection.rs` | 6 | Selection 范围、包含检查 |
| `torvox-core/config.rs` | 4 | TerminalConfig 序列化 |
| `torvox-terminal/terminal.rs` | 72 | CSI 光标/擦除/行操作/SGR、ESC/OSC |
| `torvox-terminal/terminal.rs` (proptest) | 3 | 随机字节不崩溃、Grid 维度、DirtyMask |
| `torvox-terminal/keyboard.rs` | 43 | Kitty/VT 编码、鼠标 SGR、括号粘贴 |
| `torvox-terminal/pty.rs` | 4 | PTY spawn/read/write/drop |
| `torvox-terminal/session.rs` | 5 | Session spawn/exit/echo/resize/write |
| `torvox-renderer/font.rs` | 7 | 字体发现、字形光栅化、图集填充 |
| `torvox-renderer/gpu.rs` | 3 | Instance 大小、投影矩阵、BufferLayout |
| `torvox-gui-android/bridge.rs` | 7 | UniFFI 类型转换、配置序列化 |

### 2.3 测试策略详解

**单元测试**: 每个公共函数至少 1 个测试。测试命名遵循 `功能_场景_预期结果` 模式。

**属性测试**: 使用 proptest 生成随机输入，验证:
- VT 解析器: 任意 0-255 字节序列不导致 panic
- Grid: 任意 dimensions 创建正确
- DirtyMask: mark/is_dirty/clear 一致性

**集成测试**: Session 模块通过真实 PTY 测试:
- spawn shell → echo hello → Grid 包含 "hello"
- spawn → write → read → verify output
- spawn → resize → verify dimensions
- spawn → exit → verify process terminated

**编译时保证**: `cargo clippy --deny warnings` 确保代码质量。

---

## 三、实现内容详解

### 3.1 torvox-core (no_std)

**Cell/Attrs/Color**: 终端单元格数据结构。`Attrs` 有 10 个 SGR 字段 (bold, dim, italic, underline, double_underline, reverse, strikethrough, blink, hidden, overline)。`Color` 支持 ANSI 256 和 TrueColor。

**DirtyMask**: `Vec<u64>` 分区位标志，每 u64 覆盖 64 行。支持任意终端高度。`mark(row)`, `is_dirty(row)`, `clear()`, `mark_all(rows)`, `resize(rows)`.

**Grid**: 二维单元格网格。`cell_mut(row, col)`, `scroll_up/down(top, bottom)`, `insert/delete_lines()`, `clear_cells()`, `fill_cells()`.

**CursorState**: 光标位置、样式 (Block/Underline/Bar)、可见性。

**Selection**: 字符/词/行/块选择模式。

**Unicode**: 宽度检测 (ASCII=1, CJK=2, 零宽=0)。

### 3.2 torvox-terminal

**VtParser**: 包装 `vte::Parser`，提供 `advance(handler, bytes)` 方法。

**TerminalState**: 完整 VT 状态机，实现 `vte::Perform` trait:
- CSI: 光标移动 (CUU/CUD/CUF/CUB/CUP/CHA)、擦除 (ED/EL/ICH/DCH/ECH)、行操作 (IL/DL/SU/SD)
- SGR: 0-107 编码 (bold/dim/italic/underline/reverse/hidden/strikethrough/overline + 256色 + TrueColor)
- ESC: 保存/恢复光标 (7/8)、RIS (c)、HTS (H)、RI (M)、字符集指定 ((/))
- OSC: 窗口标题 (0/2)
- 模式: DECSET/DECRST (光标可见、wraparound、origin mode、alt buffer)

**Session**: PTY 会话编排器:
- 读线程: dup(fd) → 非阻塞读 → crossbeam channel → 解析器
- 写线程: PtyPair.write() 直通
- 等待线程: waitpid() 子进程退出检测
- resize(): ioctl(TIOCSWINSZ) + Grid 调整

**InputEngine**: 键盘编码:
- Kitty 协议: `CSI {code};{modifiers} u`
- VT 传统: `ESC[A` (箭头), `ESC[H` (Home), `ESC OP` (F1)
- 鼠标 SGR: `ESC[<button;col;row M/m`
- 括号粘贴: `ESC[200~` / `ESC[201~`

### 3.3 torvox-renderer

**FontPipeline**: 字体管线:
- fontdb: 字体发现 (monospace 优先)
- cosmic-text: 字体加载和管理
- swash: 字形光栅化 (ScaleContext → Scaler → Render)
- etagere: 图集货架打包
- GlyphCache: HashMap<(glyph_id, pixel_size), GlyphInfo>

**GpuContext**: wgpu v29 GPU 管线:
- Instance/Device/Queue 创建
- Surface 配置 (Bgra8UnormSrgb, AutoVsync)
- Cell 渲染管线 (instanced quads)
- Atlas 纹理上传
- Orthographic 投影矩阵
- Instance buffer 构建

**WGSL 着色器**:
- cell.wgsl: 实例化四边形，每实例: 位置 + 图集 UV + 前景色 + 背景色 + 标志
- cursor.wgsl: 纯色矩形光标

### 3.4 torvox-gui-android

**bridge.rs**: UniFFI 桥接类型:
- BridgeCell, BridgeAttrs, Shell (Enum), TerminalConfig
- TerminalEvent (6 变体), TerminalError (detail)
- TorvoxBridge: ping, echo_cells, get_config
- From/Into 转换 core ↔ bridge 类型

### 3.5 torvox-exec

多调用二进制: argv[0] 检测身份，支持符号链接模式和直接调用模式。

---

## 四、可改进之处

### 4.1 代码质量

| 问题 | 严重度 | 建议 |
|------|--------|------|
| `terminal.rs` 1440 行 | 中 | 考虑拆分为 cursor_ops.rs, erase_ops.rs, sgr.rs |
| `keyboard.rs` 728 行 | 中 | 考虑拆分为 kitty.rs, legacy.rs, mouse.rs |
| ~~`FontPipeline::glyph_info` 每次创建新实例~~ | ~~高~~ | **已修复**: build_cell_instances 现在使用传入的 &mut FontPipeline |
| ~~`build_cell_instances` 内部创建临时 FontPipeline~~ | ~~高~~ | **已修复**: 不再为每个字符创建新 FontPipeline 实例 |
| `GpuContext::render_frame` 每帧创建 instance_buffer | 中 | 应使用 staging buffer 或 triple buffering |
| `pipeline.rs` 是空壳 | 低 | 应实现或删除 |
| `atlas.rs` 和 FontPipeline 重复 etagere | 低 | 应统一使用 FontPipeline 的 atlas |

### 4.2 测试覆盖

| 缺失 | 优先级 | 建议 |
|------|--------|------|
| proptest 覆盖 keyboard 编码 | 高 | 随机 key + modifier 不 panic |
| proptest 覆盖 session 输出解析 | 高 | 随机 shell 输出不 crash |
| 模糊测试目标 | 高 | VT 解析器、OSC 解析器、UTF-8 |
| GPU 管线测试 | 中 | shader 编译测试 (需 headless) |
| 集成测试覆盖 | 中 | 完整 PTY → Parser → Grid → Render 流程 |

### 4.3 架构改进

| 问题 | 建议 |
|------|------|
| TerminalState 和 Session 分离不清 | Session 应拥有 TerminalState，不应暴露 grid_mut |
| 缺少事件系统 | 应有统一的 Event enum 跨模块传递 |
| 缺少配置系统 | FontConfig/RenderConfig 未使用 |
| 缺少错误恢复 | PTY 断开后应自动重连或优雅退出 |

---

## 五、需要延伸之处

### 5.1 P1.5 Android Surface 渲染 (阻塞: 需要 Android 设备)

- TerminalSurface.kt: SurfaceView + SurfaceHolder.Callback
- ANativeWindow → raw_window_handle → wgpu Surface
- 渲染线程: 独立 Rust 线程运行 wgpu 事件循环
- Choreographer 同步 vsync
- 首次可见输出: 启动应用 → 看到 shell 提示符

### 5.2 阶段 2 功能

| 功能 | 复杂度 | 依赖 |
|------|--------|------|
| P2.1 回滚缓冲 | 中 | 环形缓冲 + 触摸滚动 |
| P2.2 选择 | 中 | Selection 手势 + 剪贴板 |
| P2.3 修饰键栏 | 中 | UI 组件 + 键绑定 |
| P2.4 字体+主题 | 低 | FontPipeline 扩展 |
| P2.5 设置 | 低 | Compose UI + DataStore |

### 5.3 阶段 3 功能

| 功能 | 复杂度 | 依赖 |
|------|--------|------|
| P3.1 vttest 100% | 高 | VT 解析器完善 |
| P3.2 现代扩展 | 高 | OSC 8/52/133, Sixel, Kitty 图形 |
| P3.3 性能优化 | 高 | PGO, atlas LRU, instance diff |
| P3.4 模糊测试 | 中 | cargo-fuzz 3 目标 |

---

## 六、潜在问题

### 6.1 已知风险

| 风险 | 影响 | 缓解 |
|------|------|------|
| COLRv1 emoji 渲染 | 部分 emoji 可能显示异常 | 捆绑 CBDT 字体回退 |
| Mali GLES 驱动崩溃 | 预算三星设备 | 优先 Vulkan 后端 |
| Android 16 前台服务 | Play Store 审核 | 正确声明 foregroundServiceType |
| wgpu 29 Surface API | display=Option | Android/Vulkan: display=None |
| swash 0.2.x 限制 | 部分字体特性不支持 | 已内部集成 skrifa |

### 6.2 技术债务

| 项目 | 说明 |
|------|------|
| `DirtyMask` 不再 `Copy` | 含 Vec 的类型不能 Copy，需 Clone |
| `Shell::Custom(String)` | TerminalConfig 失去 Copy |
| `no_std` 与 `std` 混用 | torvox-core 需要 `alloc` feature |
| UniFFI 错误字段名限制 | 不能用 `message`，改用 `detail` |
| `std::env::set_var` 在 Rust 1.95 是 unsafe | PTY 环境变量设置需 unsafe 块 |

---

## 七、注意事项

### 7.1 开发规范

1. **规范驱动开发 (SDD)**: 先写规范，再实现。不要 vibe coding。
2. **每步验证**: `cargo clippy -- -D warnings` + `cargo test --workspace`
3. **小步提交**: 每个逻辑步骤提交，不积累 10+ 文件变更
4. **类型先行**: 先定义类型，再实现行为
5. **文档同步**: 修改代码后更新 AGENTS.md、ROADMAP.md

### 7.2 技术约束

1. `torvox-core` 是 `no_std` — 不要引入需要 `std` 的功能
2. 不要使用 `portable-pty` — 不支持 Android
3. 不要使用 `bincode` — 已废弃 (RUSTSEC-2025-0141)
4. 不要使用 `rust-android-gradle` — AGP 9.0 不兼容
5. UniFFI 库模式只允许一个 `setup_scaffolding!()`

### 7.3 构建验证

```bash
cargo clippy -- -D warnings          # 零警告
cargo test --workspace               # 214 测试通过
cargo build -p torvox-core --no-default-features --features alloc  # no_std 验证
```

---

## 八、后续计划

### 8.1 短期 (P1.5)

完成 Phase 1 唯一剩余任务: Android Surface 渲染。
需要: Android 设备或模拟器，连接 wgpu 到 SurfaceView。

### 8.2 中期 (Phase 2)

- P2.1 回滚缓冲: 环形缓冲 50K 行 + 触摸滚动
- P2.2 选择: 字符/词/行/块选择 + 剪贴板
- P2.3 修饰键栏: 屏幕修饰键 + 粘滞模式
- P2.4 字体+主题: 字体选择器 + 10+ 内置主题
- P2.5 设置: Compose 设置屏幕 + DataStore

### 8.3 长期 (Phase 3-5)

- P3.1 vttest 100% 合规
- P3.2 现代扩展 (OSC 8/52/133, Sixel, Kitty 图形)
- P3.3 性能优化 (PGO, atlas LRU, instance diff)
- P3.4 模糊测试 (cargo-fuzz 1B+ 迭代)
- Phase 4: 前台服务、无障碍、国际化、MCP 服务器
- Phase 5: SSH、标签、分割面板、插件系统

---

## 九、如何继续

### 9.1 开发流程

```
1. 阅读 ROADMAP.md 确认当前阶段
2. 阅读相关 ADR 了解决策背景
3. 检查 AGENTS.md 了解当前状态
4. 实现 → 测试 → clippy → 提交
5. 更新文档 → 推送
```

### 9.2 质量门

每次提交前必须通过:
1. `cargo clippy -- -D warnings`
2. `cargo test --workspace`
3. `cargo fmt --check`
4. 更新 AGENTS.md 状态

### 9.3 分支策略

- `main`: 稳定版本
- `phase-N/*`: 阶段工作分支
- `fix/*`: 修复分支
- PR: Squash merge 到 main

---

## 十、度量

| 指标 | 当前值 | 目标 |
|------|--------|------|
| 代码行数 | 5,297 | - |
| 测试数 | 214 | 持续增长 |
| 审计文档 | docs/AUDIT-2026-05-29.md | 完整审计报告 |
| 测试通过率 | 100% | 100% |
| Clippy 警告 | 0 | 0 |
| Phase 1 完成度 | 83% (5/6) | 100% |
| vttest 合规 | ~50% | 100% (Phase 3) |
