//! torvox-mcp binary entry point.
//!
//! Usage:
//!   torvox-mcp --socket /path/to/socket
//!   torvox-mcp --socket /path/to/socket --mcp-allow-write
//!
//! Listens on a Unix domain socket for JSON-RPC 2.0 / MCP requests.

use std::path::PathBuf;
use std::sync::Arc;

use torvox_mcp::{SessionInfo, SessionStore, serve_unix};

struct NoOpStore;

impl SessionStore for NoOpStore {
    fn read(&self, _: torvox_mcp::ReadRequest) -> Result<torvox_mcp::ReadResponse, String> {
        Ok(torvox_mcp::ReadResponse::Sessions(Vec::<SessionInfo>::new()))
    }
    fn write(&self, _: u32, _: Vec<u8>) -> Result<(), String> {
        Err("no GUI connected; cannot write to PTY".into())
    }
    fn signal(&self, _: u32, _: torvox_mcp::SignalKind) -> Result<(), String> {
        Err("no GUI connected; cannot send signal".into())
    }
}

fn main() -> std::process::ExitCode {
    let mut socket_path: Option<PathBuf> = None;
    let mut write_consent = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--socket" | "-s" => {
                socket_path = args.next().map(PathBuf::from);
            }
            "--mcp-allow-write" => {
                write_consent = true;
            }
            "--help" | "-h" => {
                eprintln!("torvox-mcp - Model Context Protocol server for Torvox");
                eprintln!();
                eprintln!("USAGE:");
                eprintln!("  torvox-mcp --socket <path> [--mcp-allow-write]");
                eprintln!();
                eprintln!("OPTIONS:");
                eprintln!("  --socket, -s <path>       Unix domain socket path to listen on");
                eprintln!("  --mcp-allow-write         Allow send_input tool (DANGEROUS)");
                eprintln!("  --help, -h                Show this help");
                return std::process::ExitCode::SUCCESS;
            }
            other => {
                eprintln!("unknown argument: {other}");
                return std::process::ExitCode::from(2);
            }
        }
    }

    let Some(socket_path) = socket_path else {
        eprintln!("ERROR: --socket <path> is required");
        return std::process::ExitCode::from(2);
    };

    let store: Arc<dyn SessionStore> = Arc::new(NoOpStore);

    if let Err(e) = serve_unix(socket_path, store, write_consent) {
        eprintln!("mcp: serve failed: {e}");
        return std::process::ExitCode::from(1);
    }
    std::process::ExitCode::SUCCESS
}
