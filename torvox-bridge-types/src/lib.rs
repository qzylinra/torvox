use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeCell {
    pub char_code: u32,
    pub fg: u32,
    pub bg: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    pub shell: String,
    pub rows: u32,
    pub cols: u32,
    pub scrollback_lines: u32,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            shell: "/system/bin/sh".to_string(),
            rows: 24,
            cols: 80,
            scrollback_lines: 5000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerminalEvent {
    Bell,
    ProcessExited { exit_code: i32 },
    CellUpdate { row: u32, col: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum TerminalError {
    #[error("PTY error: {message}")]
    PtyError { message: String },
    #[error("invalid config: {message}")]
    InvalidConfig { message: String },
}
