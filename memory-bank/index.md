# Torvox 历史经验

## 目录

| # | 分类 | 标题 |
|--->-----|------|
| 1 | [Bridge/FFI](lessons/01-bridge-ffi.md) | Boltffi 桥接层字段对齐 — Wire Format 静默损坏 |
| 2 | [GPU/Render](lessons/02-gpu-render.md) | Render Thread 生命周期管理 — Surface 销毁后线程死亡 |
| 3 | [Android](lessons/03-android-pitfalls.md) | GPU Surface 未释放导致 VK_ERROR_NATIVE_WINDOW_IN_USE_KHR |
| 4 | [Android](lessons/03-android-pitfalls.md) | JNA 不支持 Array\<ByteArray\> — 需原生内存手动管理 |
| 5 | [Android](lessons/03-android-pitfalls.md) | Keyboard Jelly Effect + Coroutine 泄漏 + 设置默认值不同步 |
| 6 | [VT/Terminal](lessons/04-vt-terminal.md) | CSI cursor_position 1-indexed 未转换 + DEC 模式路由绕过 |
| 7 | [Build/CI](lessons/05-build-ci.md) | Ghostty Android 动态链接的反复尝试 |
| 8 | [Build/CI](lessons/05-build-ci.md) | CARGO_TARGET 环境变量命名 — Nushell str replace 非全局替换 |
| 9 | [Testing](lessons/06-testing.md) | 测试质量审计 — 82 个无效测试的清除 |
| 10 | [VT/Terminal](lessons/04-vt-terminal.md) | Keyboard Encoding — SS3 vs CSI Modifier Encoding 12 Bugs |
| 11 | [VT/Terminal](lessons/04-vt-terminal.md) | erase_in_display/erase_in_line 错误移动了光标位置 |
| 12 | [VT/Terminal](lessons/04-vt-terminal.md) | SGR 属性积累 vs 替换 |
| 13 | [Build/CI](lessons/05-build-ci.md) | Mesa Lavapipe 替代 SwiftShader — 30分钟构建降到即时 |
| 14 | [Testing](lessons/06-testing.md) | 删除78个derive宏测试后被Revert |
| 15 | [Testing](lessons/06-testing.md) | scrollbackLine()返回null导致搜索失效 — GhosttyTerminal API 陷阱 |
| 16 | [Testing](lessons/06-testing.md) | Android像素验证 → Rust端内部状态验证 |

## 分类说明

| 分类 | 文件 | 内容 |
|------|------|------|
| Bridge/FFI | `01-bridge-ffi.md` | boltffi, JNA, FFI 桥接层陷阱 |
| GPU/Render | `02-gpu-render.md` | wgpu, Vulkan, Surface 管理, GPU pipeline |
| Android | `03-android-pitfalls.md` | Android 特定: SurfaceView, JNA, Activity 生命周期 |
| VT/Terminal | `04-vt-terminal.md` | VT 解析, CSI/OSC 处理, Ghostty 集成 |
| Build/CI | `05-build-ci.md` | Nix, Gradle, cargo-ndk, 交叉编译 |
| Testing | `06-testing.md` | 测试策略, 审计经验 |
