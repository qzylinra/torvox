#[derive(Debug, Clone, uniffi::Record)]
pub struct BridgeCell {
    pub char_code: u32,
    pub fg: u32,
    pub bg: u32,
}

#[derive(Debug, Clone, uniffi::Record)]
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

#[derive(Debug, Clone, uniffi::Enum)]
pub enum TerminalEvent {
    Bell,
    ProcessExited { exit_code: i32 },
    CellUpdate { row: u32, col: u32 },
}

#[derive(Debug, Clone, thiserror::Error, uniffi::Error)]
pub enum TerminalError {
    #[error("PTY error: {detail}")]
    PtyError { detail: String },
    #[error("invalid config: {detail}")]
    InvalidConfig { detail: String },
}

#[derive(uniffi::Object)]
pub struct TorvoxBridge {
    config: TerminalConfig,
}

#[uniffi::export]
impl TorvoxBridge {
    #[uniffi::constructor]
    fn new(config: TerminalConfig) -> Self {
        Self { config }
    }

    fn ping(&self) -> String {
        "pong".to_string()
    }

    fn spawn_terminal(&self, rows: u32, cols: u32) -> Result<i32, TerminalError> {
        let shell = if self.config.shell.is_empty() {
            "/system/bin/sh"
        } else {
            &self.config.shell
        };
        let pty =
            torvox_terminal::PtyPair::spawn(shell, rows as u16, cols as u16).map_err(|e| {
                TerminalError::PtyError {
                    detail: e.to_string(),
                }
            })?;
        Ok(pty.child_pid().as_raw())
    }

    fn get_config(&self) -> TerminalConfig {
        self.config.clone()
    }

    fn echo_cells(&self, cells: Vec<BridgeCell>) -> Vec<BridgeCell> {
        cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_ping() {
        let config = TerminalConfig {
            shell: "/bin/sh".to_string(),
            rows: 24,
            cols: 80,
            scrollback_lines: 5000,
        };
        let bridge = TorvoxBridge::new(config);
        assert_eq!(bridge.ping(), "pong");
    }

    #[test]
    fn bridge_get_config() {
        let config = TerminalConfig {
            shell: "/bin/bash".to_string(),
            rows: 40,
            cols: 120,
            scrollback_lines: 10000,
        };
        let bridge = TorvoxBridge::new(config.clone());
        let got = bridge.get_config();
        assert_eq!(got.shell, config.shell);
        assert_eq!(got.rows, config.rows);
    }

    #[test]
    fn bridge_echo_cells() {
        let config = TerminalConfig::default();
        let bridge = TorvoxBridge::new(config);
        let cells = vec![BridgeCell {
            char_code: 'A' as u32,
            fg: 0xFFFFFF,
            bg: 0x000000,
        }];
        let result = bridge.echo_cells(cells.clone());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].char_code, 'A' as u32);
    }
}
