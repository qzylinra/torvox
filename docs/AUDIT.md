# Torvox 审计报告

> 日期: 2026-05-29
> 测试: 225 全部通过 | Clippy: 零警告 | Android lint: 通过 | Android release build: 成功

## 一、当前完成状态

| 里程碑 | 状态 | 说明 |
|--------|------|------|
| P0.1-P0.6 | ✅ | 基础设施完成 |
| P1.1 VT解析器 | ✅ | vte 0.15, 76测试 |
| P1.2 PTY会话 | ✅ | crossbeam, 5集成测试 |
| P1.3 字体管线 | ✅ | fontdb+cosmic-text+swash+etagere |
| P1.4 GPU渲染 | ✅ | wgpu v29, WGSL着色器 |
| P1.5 Android Surface | ✅ | Rust surface.rs + Kotlin TerminalSurface.kt |
| P1.6 输入处理 | ✅ | Kitty+VT传统+鼠标SGR, 43测试 |
| P2.1 回滚缓冲UI | ⬜ | Grid scrollback已实现, Kotlin触摸滚动UI待完成 |

## 二、已知问题

### 严重 (影响正确性)
无。

### 重要 (影响质量)
1. **Kitty push/pop/restore 未实现** — keyboard.rs 仅编码基础 CSI u，缺少 push/pop 配置、save/restore 操作。依赖 Kitty 终端的应用可能行为不正确。
2. **VT 解析器无模糊测试** — cargo-fuzz 骨架已建 (torvox-fuzz)，但未在 CI 中运行。边缘输入可能 panic。
3. **无确定性回放测试** — PTY 输出序列化→回放→断言的回归测试未实现。

### 次要 (可改进)
1. **Unicode 手写宽度表** — unicode.rs 自建宽度表，可考虑 unicode-width crate 替代，但非必要。
2. **FontConfig 缺少 Default** — config.rs 中 FontConfig 无 Default 实现，size 与 RenderConfig.font_size 重复。
3. **ClipboardRequest(String) 语义不清** — event.rs 中该变体仅传递字符串，改为结构体更清晰。
4. **torvox-terminal/src/grid.rs 有 #[allow(dead_code)]** — 逐字段控制 dead_code 更精确。

## 三、代码质量

| 指标 | 值 |
|------|-----|
| 测试总数 | 225 (76+128+10+7+4) |
| proptest | 8策略 (5 terminal + 3 keyboard) |
| unsafe块 | 9个 (全部有SAFETY注释) |
| unwrap(库代码) | 0 (全部改为expect或if-let) |
| Clippy | 零警告 |
| 格式化 | 通过 |
| Android lint | 通过 |
| Android release build | 成功 (debug签名) |

## 四、TerminalEvent 变体数量

| 位置 | 数量 | 变体 |
|------|------|------|
| torvox-core (event.rs) | 9 | OutputReady, Bell, TitleChanged, ClipboardRequest, HyperlinkHover, ProcessExited, CursorChanged, SelectionChanged, DirtyRegion |
| bridge.rs (UniFFI) | 8 | Bell, TitleChanged, ClipboardRequest, HyperlinkHover, ProcessExited, DirtyRegion, CursorChanged, SelectionChanged |

OutputReady 不在 bridge 中 — 它是内部事件，不暴露给 Kotlin。

## 五、依赖审计

所有依赖为当前最佳选择，版本锁定于 ARCHITECTURE.md:
- vte 0.15 (VT解析), nix 0.31 (PTY), wgpu 29 (GPU), cosmic-text 0.19 (文本整形)
- swash 0.2.7 (光栅化, 内部含skrifa), etagere 0.3 (图集打包)
- postcard 1.1 (序列化, 替代已废弃bincode), thiserror 2 (库错误类型)
- uniffi 0.31 (Kotlin绑定), crossbeam 0.8 (无锁队列), proptest 1.11 (属性测试)

无已知安全漏洞 (cargo audit clean)。

## 六、下一步

1. **P2.1 回滚缓冲UI** — Grid scrollback已就绪，需Kotlin触摸滚动+fling手势
2. **Kitty push/pop/restore** — 完善键盘协议支持
3. **模糊测试** — 在CI nightly中运行cargo-fuzz
4. **确定性回放测试** — PTY输出序列化→回放→断言
