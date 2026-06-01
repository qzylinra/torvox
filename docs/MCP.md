# Torvox MCP Server

Model Context Protocol (MCP) server for AI agent inspection of Torvox
terminal sessions.

## Quick start

```bash
# Build
cargo build -p torvox-mcp --release

# Start (read-only)
./target/release/torvox-mcp --socket /tmp/torvox-mcp.sock

# Start with write permission (DANGEROUS — only when needed)
./target/release/torvox-mcp --socket /tmp/torvox-mcp.sock --mcp-allow-write
```

## Test with Python

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

# Initialize
print(call({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}))

# List tools
print(call({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}))

# Call a tool
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

## Available tools

| Tool | Description | Required permission |
|------|-------------|---------------------|
| `list_sessions` | List all active terminal sessions | read |
| `read_grid` | Read grid cells (text + attributes) | read |
| `read_scrollback` | Read last N lines of scrollback | read |
| `read_cursor` | Read cursor row, col, visibility | read |
| `read_selection` | Read currently selected text | read |
| `read_title` | Read session title (OSC 0/2) | read |
| `send_input` | Write text to PTY | write (--mcp-allow-write) |
| `send_signal` | Send signal to child process | write (--mcp-allow-write) |

## Wire protocol

JSON-RPC 2.0 over newline-delimited JSON. One JSON object per line.

```text
client → server:  {"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
server → client:  {"jsonrpc":"2.0","id":1,"result":{...}}

client → server:  {"jsonrpc":"2.0","id":2,"method":"tools/call","params":{...}}
server → client:  {"jsonrpc":"2.0","id":2,"result":{...}}
```

Errors follow JSON-RPC 2.0 conventions with `code`, `message`, optional `data`.

## Security

- The Unix socket is created with default permissions (0o755). All local users can connect.
- Use `--mcp-allow-write` only when you need write tools (`send_input`, `send_signal`).
- Never expose this socket to the network (it's a local IPC only).
- For multi-user systems, restrict the socket directory permissions.

## Architecture

```
┌─────────────┐
│ AI Agent    │   Claude Code, Open Interpreter, etc.
└──────┬──────┘
       │ stdio / socket
       ▼
┌─────────────┐
│ torvox-mcp  │   Independent process (JSON-RPC server)
└──────┬──────┘
       │ Unix socket (configurable path)
       ▼
┌─────────────┐
│  Torvox GUI │   Android (Kotlin) or desktop
│  (process)  │   Owns actual sessions (PTY + terminal + parser)
└─────────────┘
```

`torvox-mcp` does NOT own any terminal state. It relays requests to the GUI
process, which has the actual `torvox-terminal::Session` instances.

## Integration

In the GUI process, implement the `SessionStore` trait and pass an
`Arc<dyn SessionStore>` to `torvox_mcp::serve_unix`:

```rust
use torvox_mcp::{SessionStore, serve_unix};

struct MyStore { /* ... */ }

impl SessionStore for MyStore {
    fn read(&self, req: ReadRequest) -> Result<ReadResponse, String> {
        // translate to your session manager
    }
    fn write(&self, session_id: u32, data: Vec<u8>) -> Result<(), String> {
        // write to PTY
    }
    fn signal(&self, session_id: u32, sig: SignalKind) -> Result<(), String> {
        // send signal to child
    }
}

let store: Arc<dyn SessionStore> = Arc::new(MyStore { /* ... */ });
serve_unix(socket_path, store, write_consent)?;
```

## Testing

```bash
cargo test -p torvox-mcp
```

11 unit tests cover: initialization, tool listing, write consent, error
envelopes, Cell conversion, and serde roundtrip with quickcheck properties.
