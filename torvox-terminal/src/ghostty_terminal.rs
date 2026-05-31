use libghostty_vt::{Terminal, TerminalOptions};

pub struct GhosttyTerminal {
    terminal: Terminal<'static, 'static>,
}

// SAFETY: GhosttyTerminal is only accessed from JNI single thread.
// The Terminal is !Send by design, but in Android context it's
// always accessed from the same thread.
unsafe impl Send for GhosttyTerminal {}
unsafe impl Sync for GhosttyTerminal {}

#[allow(clippy::collapsible_if)]
impl GhosttyTerminal {
    pub fn new(rows: u32, cols: u32, scrollback_lines: u32) -> Result<Self, String> {
        let terminal = Terminal::new(TerminalOptions {
            cols: cols as u16,
            rows: rows as u16,
            max_scrollback: scrollback_lines as usize,
        })
        .map_err(|e| format!("Ghostty Terminal::new failed: {e}"))?;

        Ok(Self { terminal })
    }

    pub fn vt_write(&mut self, data: &[u8]) {
        self.terminal.vt_write(data);
    }

    pub fn resize(&mut self, rows: u32, cols: u32, cell_width_px: u32, cell_height_px: u32) {
        let _ = self
            .terminal
            .resize(cols as u16, rows as u16, cell_width_px, cell_height_px);
    }

    pub fn rows(&self) -> u32 {
        self.terminal.rows().unwrap_or(24) as u32
    }

    pub fn cols(&self) -> u32 {
        self.terminal.cols().unwrap_or(80) as u32
    }

    pub fn cursor_x(&self) -> u32 {
        self.terminal.cursor_x().unwrap_or(0) as u32
    }

    pub fn cursor_y(&self) -> u32 {
        self.terminal.cursor_y().unwrap_or(0) as u32
    }

    pub fn cursor_visible(&self) -> bool {
        self.terminal.is_cursor_visible().unwrap_or(true)
    }

    pub fn title(&self) -> &str {
        self.terminal.title().unwrap_or("")
    }

    pub fn scrollback_len(&self) -> u32 {
        self.terminal.scrollback_rows().unwrap_or(0) as u32
    }

    pub fn total_rows(&self) -> u32 {
        self.terminal.total_rows().unwrap_or(0) as u32
    }

    pub fn read_line_text(&self, row: u32) -> Option<String> {
        use libghostty_vt::terminal::Point;
        use libghostty_vt::terminal::PointCoordinate;
        let mut text = String::new();
        let cols = self.cols();
        for col in 0..cols {
            let coord = PointCoordinate {
                x: col as u16,
                y: row,
            };
            if let Ok(point) = self.terminal.grid_ref(Point::Viewport(coord))
                && let Ok(cell) = point.cell()
            {
                let cp = cell.codepoint().unwrap_or(0);
                if cp != 0 {
                    if let Some(ch) = char::from_u32(cp) {
                        text.push(ch);
                    }
                } else {
                    text.push(' ');
                }
            }
        }
        Some(text.trim_end().to_string())
    }

    pub fn terminal_mut(&mut self) -> &mut Terminal<'static, 'static> {
        &mut self.terminal
    }

    pub fn terminal(&self) -> &Terminal<'static, 'static> {
        &self.terminal
    }
}
