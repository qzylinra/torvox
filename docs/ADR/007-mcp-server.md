# ADR 007: MCP Server for AI Agent Inspection

- **状态**: 已接受
- **日期**: 2026-06-01
- **决策者**: 项目作者

## 背景

Torvox 是一个终端模拟器。AI 代理（如 Claude Code、Open Interpreter）需要能够：

1. **观察** 当前终端状态 (grid, scrollback, cursor, selection, title)
2. **结构化地读取** 状态 (而不是解析 ANSI 转义序列)
3. **写入** PTY (在用户同意下，发送输入)
4. **发送信号** (Ctrl-C, SIGHUP 等)

传统的 stdio 包装器 (如 fabric 模式) 不可行，因为我们不想用 LLM 替换 shell — 我们要增强现有的 shell 体验。

## 决策

实现 [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) 服务器作为独立 crate (`torvox-mcp`)：

- **传输**: Unix 域套接字 (本地 IPC，非网络)
- **协议**: JSON-RPC 2.0 / MCP 2024-11-05
- **默认**: 关闭 — 必须显式启用 (`--socket <path>` flag)
- **写入**: 双重保护 — `--mcp-allow-write` flag (用户层) + GUI SessionStore 实现层
- **架构**: 双进程模型 — `torvox-mcp` 进程作为桥，GUI 进程通过共享抽象暴露 session 状态

## 关键工具

| 名称 | 用途 | 权限 |
|------|------|------|
| `list_sessions` | 列出所有活动会话 | 读 |
| `read_grid` | 读取当前 grid (rows × cols cells) | 读 |
| `read_scrollback` | 读取最后 N 行回滚 | 读 |
| `read_cursor` | 读取光标位置/可见性 | 读 |
| `read_selection` | 读取当前选择 | 读 |
| `read_title` | 读取会话标题 (OSC 0/2) | 读 |
| `send_input` | 写入 PTY | 写 (需 `--mcp-allow-write`) |
| `send_signal` | 发送信号 (SIGINT/TERM/HUP/QUIT) | 写 (需 `--mcp-allow-write`) |

## 备选方案

### A. 集成在 GUI 进程中

将 MCP 服务器嵌入 `torvox-gui-android`，通过 JNI 暴露。**拒绝原因**：
- 增加主进程的攻击面 (MCP 服务器任何 bug 影响终端)
- 难以在桌面平台复用

### B. 共享库 (cdylib) + CLI 包装

将 MCP 服务器作为共享库，CLI thin wrapper 启动它。**拒绝原因**：
- 桌面平台才需要 (Android 通过 JNI 启动 binary)
- 增加构建复杂度 (无明显收益)

### C. 通过 Unix 套接字 (已选)

独立进程 + IPC。**接受原因**：
- 进程隔离 (MCP 崩溃不影响终端)
- 适用于桌面 (开发) 和 Android (通过 `adb shell` 或本地 socket)
- 简单的协议 (JSON-RPC 2.0)
- 易于测试 (Python/Node 客户端可立即连接)

## 安全考虑

1. **写权限**: 必须双层验证
   - 启动时 `--mcp-allow-write` flag (用户明确意图)
   - 运行时 `send_input`/`send_signal` 检查 `write_consent` 字段
2. **套接字权限**: Unix 套接字默认权限 0o755 — 任何本地用户可连接
   - 未来: 添加 `--socket-mode 0o600` 限制为当前用户
3. **无认证**: 假定本地用户已认证
   - 未来: 可选 `--auth-token` + challenge-response
4. **无加密**: 本地 IPC，不需要 TLS

## 未来扩展

- `subscribe_*` 工具: 订阅 grid 变化流 (SSE 风格)
- `eval_lua`/`eval_python`: 在终端上下文中执行脚本 (危险，需要更多同意层)
- 跨设备: 通过 SSH 隧道暴露套接字 (用户明确)
- 桌面专用: 启动 `torvox-mcp` 作为 system service (systemd / launchd)

## 实施时间表

- P4.4 (MCP 服务器) ✅ 已实现骨架
- 真实设备测试: 阻塞于 GUI 集成
- 安全加固 (socket 模式, auth): P5 后

## 相关

- P4.4 路线图条目
- [Model Context Protocol 规范](https://modelcontextprotocol.io/)
- JSON-RPC 2.0 规范
- 现有 ADR 002 (架构), 004 (PTY)
