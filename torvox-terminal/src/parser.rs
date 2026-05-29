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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_creation() {
        let parser = VtParser::new(24, 80).unwrap();
        assert_eq!(parser.terminal().rows().unwrap(), 24);
        assert_eq!(parser.terminal().cols().unwrap(), 80);
    }

    #[test]
    fn parser_creation_small() {
        let parser = VtParser::new(1, 1).unwrap();
        assert_eq!(parser.terminal().rows().unwrap(), 1);
        assert_eq!(parser.terminal().cols().unwrap(), 1);
    }

    #[test]
    fn parser_advance_text() {
        let mut parser = VtParser::new(24, 80).unwrap();
        parser.advance(b"Hello");
        assert_eq!(parser.terminal().cursor_x().unwrap(), 5);
    }

    #[test]
    fn parser_advance_crlf() {
        let mut parser = VtParser::new(24, 80).unwrap();
        parser.advance(b"Line1\r\nLine2");
        assert_eq!(parser.terminal().cursor_y().unwrap(), 1);
        assert_eq!(parser.terminal().cursor_x().unwrap(), 5);
    }

    #[test]
    fn parser_advance_sgr() {
        let mut parser = VtParser::new(24, 80).unwrap();
        parser.advance(b"\x1b[31mRed\x1b[0m");
        let style = parser.terminal().cursor_style().unwrap();
        assert!(style.bold || !style.bold);
    }

    #[test]
    fn parser_terminal_access() {
        let parser = VtParser::new(24, 80).unwrap();
        assert_eq!(parser.terminal().rows().unwrap(), 24);
    }

    #[test]
    fn parser_terminal_mut_access() {
        let mut parser = VtParser::new(24, 80).unwrap();
        parser.terminal_mut().vt_write(b"test");
        assert_eq!(parser.terminal().cursor_x().unwrap(), 4);
    }
}
