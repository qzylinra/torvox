use libghostty_vt::{Terminal, TerminalOptions};

pub struct VtParser {
    terminal: Terminal<'static, 'static>,
}

impl VtParser {
    pub fn new(rows: u16, cols: u16) -> Result<Self, libghostty_vt::Error> {
        let terminal = Terminal::new(TerminalOptions {
            cols,
            rows,
            max_scrollback: 10000,
        })?;
        Ok(Self { terminal })
    }

    pub fn advance(&mut self, bytes: &[u8]) {
        self.terminal.vt_write(bytes);
    }

    pub fn terminal(&self) -> &Terminal<'static, 'static> {
        &self.terminal
    }

    pub fn terminal_mut(&mut self) -> &mut Terminal<'static, 'static> {
        &mut self.terminal
    }
}
