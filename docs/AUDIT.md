# Torvox 审计报告

> 日期: 2026-05-30
> 测试: 155 全部通过 | Clippy: 零警告 | 格式化: 通过

## 一、当前完成状态

| 里程碑 | 状态 | 说明 |
|--------|------|------|
| P0.1-P0.6 | ✅ | 基础设施完成 |
| P1.1 VT解析器 | ✅ | libghostty-vt 0.1, SIMD优化 |
| P1.2 PTY会话 | ✅ | flume通道, 5集成测试 |
| P1.3 字体管线 | ✅ | fontdb+cosmic-text+swash+guillotière |
| P1.4 GPU渲染 | ✅ | wgpu v29, WGSL着色器, FlatGrid |
| P1.5 Android Surface | ✅ | Ghostty VT集成 + RenderState |
| P1.6 输入处理 | ✅ | Kitty+VT传统+鼠标SGR |

## 二、依赖迁移状态

| # | 迁移 | 从 | 到 | 状态 |
|---|------|----|----|------|
| 1 | VT解析器 | vte 0.15 | libghostty-vt 0.1 | ✅ 完成 |
| 2 | 通道库 | crossbeam 0.8 | flume 0.11 | ✅ 完成 |
| 3 | 图集打包 | etagere 0.3 | guillotière 0.7 | ✅ 完成 |
| 4 | 序列化 | postcard 1.1 | dev-dependency | ✅ 完成 |
| 5 | Rust-Kotlin绑定 | UniFFI 0.31 | boltffi 0.25 | ✅ 完成 |

**UniFFI → boltffi 迁移已完成**: 所有 UniFFI 引用已更新为 boltffi 等效项。
boltffi 0.25 使用 `#[data]`/`#[error]`/`#[boltffi::export]` 注解，不需要 `uniffi.toml` 配置文件或 `setup_scaffolding!()`。

## 三、代码质量

| 指标 | 值 |
|------|-----|
| 测试总数 | 155 (76 core + 7 gui + 4 integration + 10 renderer + 58 terminal) |
| unsafe块 | 有 (全部有SAFETY注释) |
| unwrap(库代码) | 0 (全部改为expect或if-let) |
| Clippy | 零警告 |
| 格式化 | 通过 |

## 四、依赖审计

| 依赖 | 版本 | 用途 | 状态 |
|------|------|------|------|
| libghostty-vt | 0.1 | VT解析+终端状态 | ✅ (SIMD优化, VT100-520) |
| nix | 0.31.3 | PTY (Unix API) | ✅ |
| wgpu | 29.0.3 | GPU渲染 (WebGPU) | ✅ |
| cosmic-text | 0.19 | 文本整形 | ✅ |
| swash | 0.2.7 | 字体光栅化 | ✅ |
| guillotiere | 0.7 | 图集打包 | ✅ |
| flume | 0.11 | 无锁通道 | ✅ |
| thiserror | 2.0.18 | 库错误类型 | ✅ |
| boltffi | 0.25 | Kotlin绑定 | ✅ |
| proptest | 1.11 | 属性测试 | ✅ |
| postcard | 1.1.3 | 测试序列化 | ✅ (dev-dependency) |

## 五、构建依赖

Ghostty VT 构建需要 zig 0.15.2。已通过 nix flake 配置：
- `devShells.default` 包含 `pkgs.zig_0_15`
- `checks` native-dependencies 包含 `pkgs.zig_0_15`
- CI 环境通过 nix develop 获取 zig

## 六、已决策事项

| 决策 | 选择 | 理由 |
|------|------|------|
| VT解析器 | libghostty-vt 0.1 | SIMD优化，VT100-520完整兼容 |
| 通道库 | flume 0.11 | 更快，无unsafe |
| 图集打包 | guillotière 0.7 | 相同作者，更现代的算法 |
| 序列化 | postcard → dev-dep | 仅测试用，不进入生产代码 |
| Rust-Kotlin绑定 | boltffi 0.25 | 类型安全绑定，不需要配置文件 |
| GPU API | wgpu 29 | 跨平台，WebGPU标准 |
| 渲染管线 | FlatGrid + build_cell_instances_from_flat | 适配 Ghostty VT RenderState |

## 七、下一步

1. **实现渲染循环** — Rust 专用线程 + ANativeWindow
2. **实现 cursor.wgsl** — 光标 GPU 渲染 (Block/Underline/Bar)
3. **P2.3 修饰键栏** — 底部固定栏 (Ctrl/Alt/Esc/Tab)
4. **P2.4 字体+主题** — 字体大小调整，主题支持
5. **P2.5 设置** — Jetpack Compose 设置屏幕
6. **优化 RenderState 渲染** — 避免每帧重建 FlatGrid，使用增量渲染
