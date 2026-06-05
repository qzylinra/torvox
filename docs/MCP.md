<!-- AUDIT: 2026-06-02 — Verified 8 tools, 10 tests + 1 quickcheck (was claimed 11). All architecture claims accurate. -->
# Torvox MCP 服务器

Model Context Protocol (MCP) 服务器，用于 AI 代理检查 Torvox 终端会话。

## 快速开始

```bash
# 构建
cargo build -p torvox-mcp --release

# 启动（只读）
./target/release/torvox-mcp --socket /tmp/torvox-mcp.sock

# 启动并启用写入权限（危险 — 仅在需要时使用）
./target/release/torvox-mcp --socket /tmp/torvox-mcp.sock --mcp-allow-write
```

## 使用 Python 测试

```python
import socket, json

s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.connect("/tmp/torvox-mcp.sock")

def call(req):
    s.sendall((json.dumps(req) + "\n").encode())
    buf = b""
    while b"\n" not in buf:
        chunk = s.recv(4096)
        if not chunk:
            break
        buf += chunk
    return json.loads(buf.decode().split("\n")[0])

# 初始化
print(call({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}))

# 列出工具
print(call({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}))

# 调用工具
print(call({
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
        "name": "list_sessions",
        "arguments": {}
    }
}))
```

## 可用工具

| 工具 | 描述 | 所需权限 |
|------|------|----------|
| `list_sessions` | 列出所有活动终端会话 | 读 |
| `read_grid` | 读取网格单元格（文本 + 属性） | 读 |
| `read_scrollback` | 读取最后 N 行回滚 | 读 |
| `read_cursor` | 读取光标行、列、可见性 | 读 |
| `read_selection` | 读取当前选中文本 | 读 |
| `read_title` | 读取会话标题 (OSC 0/2) | 读 |
| `send_input` | 向 PTY 写入文本 | 写 (`--mcp-allow-write`) |
| `send_signal` | 向子进程发送信号 | 写 (`--mcp-allow-write`) |

## 线路协议

基于换行分隔 JSON 的 JSON-RPC 2.0。每行一个 JSON 对象。

```text
客户端 → 服务端:  {"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
服务端 → 客户端:  {"jsonrpc":"2.0","id":1,"result":{...}}

客户端 → 服务端:  {"jsonrpc":"2.0","id":2,"method":"tools/call","params":{...}}
服务端 → 客户端:  {"jsonrpc":"2.0","id":2,"result":{...}}
```

错误遵循 JSON-RPC 2.0 约定，包含 `code`、`message`、可选 `data`。

## 架构

```
┌─────────────┐
│ AI 代理     │   Claude Code, Open Interpreter 等
└──────┬──────┘
       │ stdio / socket
       ▼
┌─────────────┐
│ torvox-mcp  │   独立进程 (JSON-RPC 服务器)
└──────┬──────┘
       │ Unix 套接字（可配置路径）
       ▼
┌─────────────┐
│  Torvox GUI │   Android (Kotlin) 或桌面端
│  (进程)      │   持有实际会话 (PTY + 终端 + 解析器)
└─────────────┘
```

`torvox-mcp` **不**持有任何终端状态。它将请求中继到 GUI 进程，由 GUI 进程持有实际的 `torvox-terminal::Session` 实例。

### 为什么是独立进程？

| 方案 | 拒绝/选择原因 |
|------|-------------|
| A. 集成在 GUI 进程中 | 增加主进程攻击面；MCP bug 影响终端；难复用 |
| B. 共享库 (cdylib) + CLI 包装 | 桌面平台才需要；增加构建复杂度 |
| **C. Unix 套接字 (选择)** | **进程隔离；桌面/Android 均适用；协议简单；易测试** |

## 安全

1. **写权限**: 双层验证 — 启动时 `--mcp-allow-write` flag (用户意图) + 运行时 `write_consent` 字段 (GUI 层)
2. **套接字权限**: 默认 0o755，所有本地用户可连接。未来添加 `--socket-mode 0o600` 限制
3. **无认证**: 假定本地用户已认证。未来可添加 `--auth-token` + challenge-response
4. **无加密**: 本地 IPC，不需要 TLS
5. **多用户系统**: 应限制套接字目录权限

## 集成

在 GUI 进程中，实现 `SessionStore` trait 并将 `Arc<dyn SessionStore>` 传递给 `torvox_mcp::serve_unix`：

```rust
use torvox_mcp::{SessionStore, serve_unix};

struct MyStore { /* ... */ }

impl SessionStore for MyStore {
    fn read(&self, req: ReadRequest) -> Result<ReadResponse, String> {
        // 转接到你的会话管理器
    }
    fn write(&self, session_id: u32, data: Vec<u8>) -> Result<(), String> {
        // 写入 PTY
    }
    fn signal(&self, session_id: u32, sig: SignalKind) -> Result<(), String> {
        // 向子进程发送信号
    }
}

let store: Arc<dyn SessionStore> = Arc::new(MyStore { /* ... */ });
serve_unix(socket_path, store, write_consent)?;
```

## 未来扩展

- `subscribe_*` 工具: 订阅 grid 变化流 (SSE 风格)
- `eval_lua`/`eval_python`: 在终端上下文中执行脚本 (危险，需要更多同意层)
- 跨设备: 通过 SSH 隧道暴露套接字 (用户明确)
- 桌面专用: 启动 `torvox-mcp` 作为 system service (systemd / launchd)

## 测试

```bash
cargo test -p torvox-mcp
```

10 个单元测试 + 1 个 quickcheck 属性测试覆盖：初始化、工具列表、写入同意、错误封装、Cell 转换，以及 serde 往返测试。

## 相关规范

- [Model Context Protocol 规范](https://modelcontextprotocol.io/)
- JSON-RPC 2.0 规范
