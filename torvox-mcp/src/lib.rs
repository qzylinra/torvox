//! Model Context Protocol (MCP) server for Torvox.
//!
//! Implements a JSON-RPC 2.0 server over Unix domain sockets, exposing
//! terminal session state to AI agents for inspection and structured interaction.
//!
//! ## Architecture
//!
//! ```text
//! AI Agent  <--stdio/JSON-RPC-->  torvox-mcp  <--Unix socket-->  torvox-gui-android
//! ```
//!
//! ## Wire protocol
//!
//! JSON-RPC 2.0 over newline-delimited JSON. Each line is a complete
//! request, response, or notification.
//!
//! ## Tools exposed
//!
//! - `list_sessions`: list all active terminal sessions
//! - `read_grid`: read current grid state of a session (rows × cols)
//! - `read_scrollback`: read last N lines of scrollback
//! - `read_cursor`: read cursor position and visibility
//! - `read_selection`: read selected text (if any)
//! - `send_input`: write text to PTY (requires write consent)
//! - `send_signal`: send signal to child process (SIGINT/SIGTERM/SIGHUP)

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::sync::Arc;

use flume::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;
use torvox_core::cell::Cell;

/// Errors produced by the MCP server.
#[derive(Error, Debug)]
pub enum McpError {
    #[error("invalid JSON-RPC request: {0}")]
    InvalidRequest(String),
    #[error("unknown method: {0}")]
    UnknownMethod(String),
    #[error("invalid parameters: {0}")]
    InvalidParams(String),
    #[error("session not found: {0}")]
    SessionNotFound(u32),
    #[error("internal error: {0}")]
    Internal(String),
}

impl McpError {
    pub fn to_json_rpc_error(&self, id: &Value) -> Value {
        let (code, message) = match self {
            Self::InvalidRequest(m) => (-32600, m.clone()),
            Self::UnknownMethod(m) => (-32601, m.clone()),
            Self::InvalidParams(m) => (-32602, m.clone()),
            Self::SessionNotFound(_) => (-32001, self.to_string()),
            Self::Internal(m) => (-32603, m.clone()),
        };
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": code, "message": message },
        })
    }
}

/// A session handle exposed via MCP. Real session is owned by the GUI;
/// this is a snapshot/proxy view.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: u32,
    pub title: String,
    pub rows: u32,
    pub cols: u32,
    pub shell: String,
    pub pid: Option<u32>,
    pub is_exited: bool,
}

/// A grid snapshot for MCP consumers. Only the text content + attributes,
/// not the full Cell binary form.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GridCellData {
    pub row: u32,
    pub col: u32,
    pub codepoint: u32,
    pub fg_r: u8,
    pub fg_g: u8,
    pub fg_b: u8,
    pub bg_r: u8,
    pub bg_g: u8,
    pub bg_b: u8,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub reverse: bool,
    pub dim: bool,
    pub strikethrough: bool,
    pub blink: bool,
    pub hidden: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GridSnapshotData {
    pub rows: u32,
    pub cols: u32,
    pub cells: Vec<GridCellData>,
    pub cursor_row: u32,
    pub cursor_col: u32,
    pub cursor_visible: bool,
}

impl From<&Cell> for GridCellData {
    fn from(c: &Cell) -> Self {
        Self {
            row: 0,
            col: 0,
            codepoint: c.char as u32,
            fg_r: c.fg.r,
            fg_g: c.fg.g,
            fg_b: c.fg.b,
            bg_r: c.bg.r,
            bg_g: c.bg.g,
            bg_b: c.bg.b,
            bold: c.attrs.bold,
            italic: c.attrs.italic,
            underline: c.attrs.underline,
            reverse: c.attrs.reverse,
            dim: c.attrs.dim,
            strikethrough: c.attrs.strikethrough,
            blink: c.attrs.blink,
            hidden: c.attrs.hidden,
        }
    }
}

/// Request from a client to perform a session read.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ReadRequest {
    Sessions,
    Grid { session_id: u32 },
    Scrollback { session_id: u32, max_lines: u32 },
    Cursor { session_id: u32 },
    Selection { session_id: u32 },
    Title { session_id: u32 },
}

/// Response to a read request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ReadResponse {
    Sessions(Vec<SessionInfo>),
    Grid(GridSnapshotData),
    Scrollback(Vec<String>),
    Cursor { row: u32, col: u32, visible: bool },
    Selection(Option<String>),
    Title(String),
}

/// Commands sent from the MCP server to the GUI's session manager.
#[derive(Debug)]
pub enum McpCommand {
    Read(ReadRequest, Sender<Result<ReadResponse, String>>),
    Write {
        session_id: u32,
        data: Vec<u8>,
        reply: Sender<Result<(), String>>,
    },
    Signal {
        session_id: u32,
        signal: SignalKind,
        reply: Sender<Result<(), String>>,
    },
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignalKind {
    Interrupt,
    Terminate,
    Hangup,
    Quit,
}

/// Server-side abstraction over session storage. The real implementation
/// (in the GUI) owns the actual sessions; this is a trait for testability.
pub trait SessionStore: Send + Sync {
    fn read(&self, req: ReadRequest) -> Result<ReadResponse, String>;
    fn write(&self, session_id: u32, data: Vec<u8>) -> Result<(), String>;
    fn signal(&self, session_id: u32, sig: SignalKind) -> Result<(), String>;
}

/// JSON-RPC 2.0 request envelope.
#[derive(Clone, Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
    pub id: Value,
}

/// JSON-RPC 2.0 response envelope.
#[derive(Clone, Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    pub id: Value,
    pub result: Value,
}

/// MCP server that handles JSON-RPC 2.0 requests.
pub struct McpServer {
    store: Arc<dyn SessionStore>,
    write_consent: bool,
}

impl McpServer {
    pub fn new(store: Arc<dyn SessionStore>) -> Self {
        Self {
            store,
            write_consent: false,
        }
    }

    /// Enable write permission (called by user via --mcp-allow-write flag).
    pub fn with_write_consent(mut self) -> Self {
        self.write_consent = true;
        self
    }

    /// Handle a single JSON-RPC request and produce a response.
    pub fn handle(&self, req: &JsonRpcRequest) -> Result<Value, McpError> {
        match req.method.as_str() {
            "initialize" => self.handle_initialize(),
            "tools/list" => Ok(self.list_tools()),
            "tools/call" => self.handle_tool_call(&req.params, &req.id),
            "ping" => Ok(json!({})),
            "notifications/initialized" => Ok(json!({})),
            _ => Err(McpError::UnknownMethod(req.method.clone())),
        }
    }

    fn handle_initialize(&self) -> Result<Value, McpError> {
        Ok(json!({
            "protocolVersion": "2024-11-05",
            "serverInfo": {
                "name": "torvox-mcp",
                "version": env!("CARGO_PKG_VERSION"),
            },
            "capabilities": {
                "tools": {}
            },
        }))
    }

    fn list_tools(&self) -> Value {
        let tools = vec![
            (
                "list_sessions",
                "List all active terminal sessions",
                empty_schema(),
            ),
            (
                "read_grid",
                "Read current grid state of a session (rows × cols cells)",
                schema_required(&["session_id"]),
            ),
            (
                "read_scrollback",
                "Read last N lines of scrollback",
                schema_required(&["session_id", "max_lines"]),
            ),
            (
                "read_cursor",
                "Read cursor position (row, col) and visibility",
                schema_required(&["session_id"]),
            ),
            (
                "read_selection",
                "Read currently selected text (if any)",
                schema_required(&["session_id"]),
            ),
            (
                "read_title",
                "Read the session title (from OSC 0/2)",
                schema_required(&["session_id"]),
            ),
            (
                "send_input",
                "Write text to PTY (requires --mcp-allow-write consent)",
                schema_required(&["session_id", "data"]),
            ),
            (
                "send_signal",
                "Send signal to child process (SIGINT/SIGTERM/SIGHUP/SIGQUIT)",
                schema_required(&["session_id", "signal"]),
            ),
        ];
        let tools_json: Vec<Value> = tools
            .into_iter()
            .map(|(name, desc, schema)| {
                json!({
                    "name": name,
                    "description": desc,
                    "inputSchema": schema,
                })
            })
            .collect();
        json!({ "tools": tools_json })
    }

    fn handle_tool_call(&self, params: &Value, _id: &Value) -> Result<Value, McpError> {
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("missing 'name'".into()))?;
        let args = params.get("arguments").cloned().unwrap_or(json!({}));

        match name {
            "list_sessions" => {
                let resp = self
                    .store
                    .read(ReadRequest::Sessions)
                    .map_err(McpError::Internal)?;
                Ok(json!({ "content": [{ "type": "json", "data": resp }] }))
            }
            "read_grid" => {
                let sid = args
                    .get("session_id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| McpError::InvalidParams("session_id required".into()))?
                    as u32;
                let resp = self
                    .store
                    .read(ReadRequest::Grid { session_id: sid })
                    .map_err(McpError::Internal)?;
                Ok(json!({ "content": [{ "type": "json", "data": resp }] }))
            }
            "read_scrollback" => {
                let sid = args
                    .get("session_id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| McpError::InvalidParams("session_id required".into()))?
                    as u32;
                let max = args
                    .get("max_lines")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| McpError::InvalidParams("max_lines required".into()))?
                    as u32;
                let resp = self
                    .store
                    .read(ReadRequest::Scrollback {
                        session_id: sid,
                        max_lines: max,
                    })
                    .map_err(McpError::Internal)?;
                Ok(json!({ "content": [{ "type": "json", "data": resp }] }))
            }
            "read_cursor" => {
                let sid = args
                    .get("session_id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| McpError::InvalidParams("session_id required".into()))?
                    as u32;
                let resp = self
                    .store
                    .read(ReadRequest::Cursor { session_id: sid })
                    .map_err(McpError::Internal)?;
                Ok(json!({ "content": [{ "type": "json", "data": resp }] }))
            }
            "read_selection" => {
                let sid = args
                    .get("session_id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| McpError::InvalidParams("session_id required".into()))?
                    as u32;
                let resp = self
                    .store
                    .read(ReadRequest::Selection { session_id: sid })
                    .map_err(McpError::Internal)?;
                Ok(json!({ "content": [{ "type": "json", "data": resp }] }))
            }
            "read_title" => {
                let sid = args
                    .get("session_id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| McpError::InvalidParams("session_id required".into()))?
                    as u32;
                let resp = self
                    .store
                    .read(ReadRequest::Title { session_id: sid })
                    .map_err(McpError::Internal)?;
                Ok(json!({ "content": [{ "type": "json", "data": resp }] }))
            }
            "send_input" => {
                if !self.write_consent {
                    return Err(McpError::InvalidParams(
                        "send_input requires --mcp-allow-write consent flag".into(),
                    ));
                }
                let sid = args
                    .get("session_id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| McpError::InvalidParams("session_id required".into()))?
                    as u32;
                let data = args
                    .get("data")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::InvalidParams("data required".into()))?
                    .as_bytes()
                    .to_vec();
                self.store.write(sid, data).map_err(McpError::Internal)?;
                Ok(json!({ "content": [{ "type": "text", "text": "wrote to PTY" }] }))
            }
            "send_signal" => {
                let sid = args
                    .get("session_id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| McpError::InvalidParams("session_id required".into()))?
                    as u32;
                let sig_str = args
                    .get("signal")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::InvalidParams("signal required".into()))?;
                let sig = match sig_str {
                    "SIGINT" | "INT" | "interrupt" => SignalKind::Interrupt,
                    "SIGTERM" | "TERM" | "terminate" => SignalKind::Terminate,
                    "SIGHUP" | "HUP" | "hangup" => SignalKind::Hangup,
                    "SIGQUIT" | "QUIT" | "quit" => SignalKind::Quit,
                    _ => {
                        return Err(McpError::InvalidParams(format!(
                            "unknown signal: {sig_str}"
                        )));
                    }
                };
                self.store.signal(sid, sig).map_err(McpError::Internal)?;
                Ok(json!({ "content": [{ "type": "text", "text": "sent signal" }] }))
            }
            _ => Err(McpError::UnknownMethod(name.into())),
        }
    }
}

fn empty_schema() -> Value {
    json!({
        "type": "object",
        "properties": {},
        "additionalProperties": false,
    })
}

fn schema_required(required: &[&str]) -> Value {
    let mut properties = BTreeMap::new();
    for r in required {
        match *r {
            "session_id" => {
                properties.insert(
                    "session_id".to_string(),
                    json!({ "type": "integer", "minimum": 0 }),
                );
            }
            "max_lines" => {
                properties.insert(
                    "max_lines".to_string(),
                    json!({ "type": "integer", "minimum": 1, "maximum": 100000 }),
                );
            }
            "data" => {
                properties.insert("data".to_string(), json!({ "type": "string" }));
            }
            "signal" => {
                properties.insert(
                    "signal".to_string(),
                    json!({
                        "type": "string",
                        "enum": ["SIGINT", "SIGTERM", "SIGHUP", "SIGQUIT"]
                    }),
                );
            }
            _ => {}
        }
    }
    let mut reqs: Vec<String> = required.iter().map(|s| s.to_string()).collect();
    reqs.sort();
    json!({
        "type": "object",
        "properties": properties,
        "required": reqs,
        "additionalProperties": false,
    })
}

/// Run a JSON-RPC server over a Unix socket at `socket_path`.
#[cfg(unix)]
pub fn serve_unix(
    socket_path: PathBuf,
    store: Arc<dyn SessionStore>,
    write_consent: bool,
) -> std::io::Result<()> {
    use std::os::unix::net::UnixListener;

    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let _ = std::fs::remove_file(&socket_path);
    let listener = UnixListener::bind(&socket_path)?;

    let mut server = McpServer::new(store as Arc<dyn SessionStore>);
    if write_consent {
        server = server.with_write_consent();
    }
    let server = Arc::new(server);

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let server = Arc::clone(&server);
                std::thread::spawn(move || {
                    let mut s = s;
                    let reader = match s.try_clone() {
                        Ok(cloned) => std::io::BufReader::new(cloned),
                        Err(_) => return,
                    };
                    for line in reader.lines().map_while(Result::ok) {
                        let line = line.trim().to_string();
                        if line.is_empty() {
                            continue;
                        }
                        let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
                            Ok(req) => match server.handle(&req) {
                                Ok(result) => json!({
                                    "jsonrpc": "2.0",
                                    "id": req.id,
                                    "result": result,
                                }),
                                Err(e) => e.to_json_rpc_error(&req.id),
                            },
                            Err(e) => json!({
                                "jsonrpc": "2.0",
                                "id": Value::Null,
                                "error": {
                                    "code": -32700,
                                    "message": format!("parse error: {e}"),
                                },
                            }),
                        };
                        let _ = writeln!(s, "{response}");
                        let _ = s.flush();
                    }
                });
            }
            Err(e) => {
                eprintln!("mcp: accept failed: {e}");
            }
        }
    }
    Ok(())
}

/// Channel bridge for in-process integration. The GUI side sends commands
/// through `tx`; the MCP server reads through `rx`.
pub fn channel_bridge(rx: Receiver<McpCommand>) {
    while let Ok(cmd) = rx.recv() {
        match cmd {
            McpCommand::Read(req, reply) => {
                let _ = reply.send(Err("no sessions connected".into()));
                let _ = req;
            }
            McpCommand::Write { reply, .. } => {
                let _ = reply.send(Err("write not supported in headless mode".into()));
            }
            McpCommand::Signal { reply, .. } => {
                let _ = reply.send(Err("signal not supported in headless mode".into()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockStore {
        sessions: Mutex<Vec<SessionInfo>>,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                sessions: Mutex::new(vec![SessionInfo {
                    id: 1,
                    title: "test".into(),
                    rows: 24,
                    cols: 80,
                    shell: "/bin/sh".into(),
                    pid: Some(1234),
                    is_exited: false,
                }]),
            }
        }
    }

    impl SessionStore for MockStore {
        fn read(&self, req: ReadRequest) -> Result<ReadResponse, String> {
            match req {
                ReadRequest::Sessions => Ok(ReadResponse::Sessions(
                    self.sessions.lock().unwrap().clone(),
                )),
                ReadRequest::Cursor { session_id } => {
                    if session_id == 1 {
                        Ok(ReadResponse::Cursor {
                            row: 5,
                            col: 10,
                            visible: true,
                        })
                    } else {
                        Err(format!("session {session_id} not found"))
                    }
                }
                _ => Err("not implemented in mock".into()),
            }
        }
        fn write(&self, _: u32, _: Vec<u8>) -> Result<(), String> {
            Ok(())
        }
        fn signal(&self, _: u32, _: SignalKind) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn handle_initialize() {
        let store = Arc::new(MockStore::new());
        let server = McpServer::new(store);
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            method: "initialize".into(),
            params: json!({}),
            id: json!(1),
        };
        let result = server.handle(&req).unwrap();
        assert_eq!(result["serverInfo"]["name"], "torvox-mcp");
    }

    #[test]
    fn list_tools() {
        let store = Arc::new(MockStore::new());
        let server = McpServer::new(store);
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            method: "tools/list".into(),
            params: json!({}),
            id: json!(1),
        };
        let result = server.handle(&req).unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert!(tools.iter().any(|t| t["name"] == "list_sessions"));
        assert!(tools.iter().any(|t| t["name"] == "read_grid"));
        assert!(tools.iter().any(|t| t["name"] == "send_input"));
    }

    #[test]
    fn unknown_method() {
        let store = Arc::new(MockStore::new());
        let server = McpServer::new(store);
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            method: "nonexistent".into(),
            params: json!({}),
            id: json!(1),
        };
        let result = server.handle(&req);
        assert!(matches!(result, Err(McpError::UnknownMethod(_))));
    }

    #[test]
    fn list_sessions_tool() {
        let store = Arc::new(MockStore::new());
        let server = McpServer::new(store);
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            method: "tools/call".into(),
            params: json!({
                "name": "list_sessions",
                "arguments": {}
            }),
            id: json!(1),
        };
        let result = server.handle(&req).unwrap();
        let content = &result["content"][0]["data"]["Sessions"];
        let arr = content.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["id"], 1);
    }

    #[test]
    fn write_requires_consent() {
        let store = Arc::new(MockStore::new());
        let server = McpServer::new(store);
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            method: "tools/call".into(),
            params: json!({
                "name": "send_input",
                "arguments": { "session_id": 1, "data": "ls" }
            }),
            id: json!(1),
        };
        let result = server.handle(&req);
        assert!(matches!(result, Err(McpError::InvalidParams(_))));
    }

    #[test]
    fn write_with_consent() {
        let store = Arc::new(MockStore::new());
        let server = McpServer::new(store).with_write_consent();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            method: "tools/call".into(),
            params: json!({
                "name": "send_input",
                "arguments": { "session_id": 1, "data": "ls" }
            }),
            id: json!(1),
        };
        let result = server.handle(&req).unwrap();
        assert_eq!(result["content"][0]["text"], "wrote to PTY");
    }

    #[test]
    fn signal_validation() {
        let store = Arc::new(MockStore::new());
        let server = McpServer::new(store).with_write_consent();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            method: "tools/call".into(),
            params: json!({
                "name": "send_signal",
                "arguments": { "session_id": 1, "signal": "INVALID" }
            }),
            id: json!(1),
        };
        let result = server.handle(&req);
        assert!(matches!(result, Err(McpError::InvalidParams(_))));
    }

    #[test]
    fn error_envelope_serializes() {
        let e = McpError::SessionNotFound(42);
        let env = e.to_json_rpc_error(&json!(1));
        assert_eq!(env["error"]["code"], -32001);
        assert!(env["error"]["message"].as_str().unwrap().contains("42"));
    }

    #[test]
    fn cell_conversion() {
        use torvox_core::cell::Cell;
        let c = Cell::default();
        let d: GridCellData = (&c).into();
        assert_eq!(d.codepoint, c.char as u32);
    }

    #[test]
    fn session_info_roundtrip() {
        let s = SessionInfo {
            id: 7,
            title: "zsh".into(),
            rows: 30,
            cols: 100,
            shell: "/bin/zsh".into(),
            pid: Some(999),
            is_exited: false,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: SessionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, 7);
        assert_eq!(back.cols, 100);
    }

    #[quickcheck_macros::quickcheck]
    fn session_info_serde_id(sid: u32) -> bool {
        let s = SessionInfo {
            id: sid,
            title: String::new(),
            rows: 24,
            cols: 80,
            shell: String::new(),
            pid: None,
            is_exited: false,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: SessionInfo = serde_json::from_str(&json).unwrap();
        back.id == sid
    }
}
