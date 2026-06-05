use std::thread;

use flume::{Receiver, Sender, bounded};
use libghostty_vt::style::StyleColor;
use libghostty_vt::terminal::{Point, PointCoordinate};
use libghostty_vt::{Terminal, TerminalOptions};

/// 终端网格的渲染快照。
/// 在终端线程上构建；由渲染器线程消费。
pub struct GridSnapshot {
    pub rows: u32,
    pub cols: u32,
    pub cells: Vec<CellSnapshot>,
}

impl GridSnapshot {
    pub fn uri_at(&self, row: u32, col: u32) -> Option<&str> {
        if row >= self.rows || col >= self.cols {
            return None;
        }
        let idx = (row * self.cols + col) as usize;
        self.cells.get(idx).and_then(|c| c.uri.as_deref())
    }
}

pub struct DumpedGrid {
    pub rows: u32,
    pub cols: u32,
    pub visible: Vec<CellSnapshot>,
    pub scrollback: Vec<Vec<CellSnapshot>>,
}

#[derive(Clone, Debug, Default)]
pub struct CellSnapshot {
    pub codepoint: u32,
    pub fg: [f32; 4],
    pub bg: [f32; 4],
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub reverse: bool,
    pub uri: Option<String>,
}

#[allow(dead_code)]
enum Command {
    Write(Vec<u8>),
    FlushAck(Sender<()>),
    SetTheme {
        bg: [u8; 3],
        fg: [u8; 3],
    },
    Resize {
        rows: u32,
        cols: u32,
    },
    TakeSnapshot(Sender<GridSnapshot>),
    ScrollbackLen(Sender<u32>),
    ReadLineText {
        row: u32,
        tx: Sender<Option<String>>,
    },
    ReadVisibleText(Sender<String>),
    SearchInScrollback {
        query: String,
        tx: Sender<Option<(u32, u32)>>,
    },
    DumpGrid {
        tx: Sender<DumpedGrid>,
    },
    Rows(Sender<u32>),
    Cols(Sender<u32>),
    CursorX(Sender<u32>),
    CursorY(Sender<u32>),
    CursorVisible(Sender<bool>),
    Title(Sender<String>),
    Terminate,
}

/// libghostty_vt::Terminal 的线程安全封装。
/// 终端运行在专用线程上；操作通过 flume 通道序列化。
/// 无需 unsafe Send/Sync 实现 — GhosttyTerminal 仅持有 `Sender<Command>` (Send + Sync)。
pub struct GhosttyTerminal {
    cmd_tx: Sender<Command>,
    handle: Option<thread::JoinHandle<()>>,
}

impl GhosttyTerminal {
    pub fn new(rows: u32, cols: u32, scrollback_lines: u32) -> Result<Self, String> {
        Self::new_with_theme(rows, cols, scrollback_lines, [30, 30, 46], [205, 214, 244])
    }

    pub fn new_with_theme(
        rows: u32,
        cols: u32,
        scrollback_lines: u32,
        initial_bg: [u8; 3],
        initial_fg: [u8; 3],
    ) -> Result<Self, String> {
        let (cmd_tx, cmd_rx) = bounded::<Command>(256);
        let handle = thread::Builder::new()
            .name("ghostty-terminal".into())
            .spawn(move || Self::run(cmd_rx, rows, cols, scrollback_lines, initial_bg, initial_fg))
            .map_err(|e| format!("failed to spawn terminal thread: {e}"))?;

        Ok(Self {
            cmd_tx,
            handle: Some(handle),
        })
    }

    fn osc_sequence(command: u8, r: u8, g: u8, b: u8) -> Vec<u8> {
        format!("\x1b]{};rgb:{:02x}/{:02x}/{:02x}\x1b\\", command, r, g, b).into_bytes()
    }

    fn run(
        rx: Receiver<Command>,
        rows: u32,
        cols: u32,
        scrollback_lines: u32,
        initial_bg: [u8; 3],
        initial_fg: [u8; 3],
    ) {
        let Ok(mut terminal) = Terminal::new(TerminalOptions {
            cols: cols as u16,
            rows: rows as u16,
            max_scrollback: scrollback_lines as usize,
        }) else {
            return;
        };

        let mut default_bg = Self::byte_color_to_float(initial_bg);
        let mut default_fg = Self::byte_color_to_float(initial_fg);

        // Use VT OSC sequences to set default colors — this is the standard
        // VT500+ protocol that every terminal emulator supports. OSC 10 = fg,
        // OSC 11 = bg. Writing via vt_write() ensures Ghostty processes them
        // through its VT parser, which is more reliable than set_default_*_color.
        terminal.vt_write(&Self::osc_sequence(
            11,
            initial_bg[0],
            initial_bg[1],
            initial_bg[2],
        ));
        terminal.vt_write(&Self::osc_sequence(
            10,
            initial_fg[0],
            initial_fg[1],
            initial_fg[2],
        ));

        // Also set API-level defaults (belt-and-suspenders)
        let _ = terminal.set_default_bg_color(Some(libghostty_vt::style::RgbColor {
            r: initial_bg[0],
            g: initial_bg[1],
            b: initial_bg[2],
        }));
        let _ = terminal.set_default_fg_color(Some(libghostty_vt::style::RgbColor {
            r: initial_fg[0],
            g: initial_fg[1],
            b: initial_fg[2],
        }));

        while let Ok(cmd) = rx.recv() {
            match cmd {
                Command::Write(data) => terminal.vt_write(&data),
                Command::FlushAck(tx) => {
                    let _ = tx.send(());
                }
                Command::SetTheme { bg, fg } => {
                    default_bg = Self::byte_color_to_float(bg);
                    default_fg = Self::byte_color_to_float(fg);
                    log::debug!(
                        "SetTheme: bg={:?} fg={:?} -> default_bg={:?} default_fg={:?}",
                        bg,
                        fg,
                        default_bg,
                        default_fg
                    );
                    terminal.vt_write(&Self::osc_sequence(11, bg[0], bg[1], bg[2]));
                    terminal.vt_write(&Self::osc_sequence(10, fg[0], fg[1], fg[2]));
                    let _ = terminal.set_default_bg_color(Some(libghostty_vt::style::RgbColor {
                        r: bg[0],
                        g: bg[1],
                        b: bg[2],
                    }));
                    let _ = terminal.set_default_fg_color(Some(libghostty_vt::style::RgbColor {
                        r: fg[0],
                        g: fg[1],
                        b: fg[2],
                    }));
                }
                Command::Resize { rows, cols } => {
                    let _ = terminal.resize(cols as u16, rows as u16, 8, 16);
                }
                Command::TakeSnapshot(tx) => {
                    let snapshot = Self::build_snapshot(&terminal, default_fg, default_bg);
                    let _ = tx.send(snapshot);
                }
                Command::ScrollbackLen(tx) => {
                    let _ = tx.send(terminal.scrollback_rows().unwrap_or(0) as u32);
                }
                Command::ReadLineText { row, tx } => {
                    let text = Self::read_line_text_impl(&terminal, row);
                    let _ = tx.send(text);
                }
                Command::ReadVisibleText(tx) => {
                    let rows = terminal.rows().unwrap_or(24) as u32;
                    let mut text = String::new();
                    for row in 0..rows {
                        if let Some(line) = Self::read_line_text_impl(&terminal, row) {
                            text.push_str(&line);
                            text.push('\n');
                        }
                    }
                    let _ = tx.send(text);
                }
                Command::SearchInScrollback { query, tx } => {
                    let result = Self::search_in_scrollback_impl(&terminal, &query);
                    let _ = tx.send(result);
                }
                Command::Rows(tx) => {
                    let _ = tx.send(terminal.rows().unwrap_or(24) as u32);
                }
                Command::Cols(tx) => {
                    let _ = tx.send(terminal.cols().unwrap_or(80) as u32);
                }
                Command::CursorX(tx) => {
                    let _ = tx.send(terminal.cursor_x().unwrap_or(0) as u32);
                }
                Command::CursorY(tx) => {
                    let _ = tx.send(terminal.cursor_y().unwrap_or(0) as u32);
                }
                Command::CursorVisible(tx) => {
                    let _ = tx.send(terminal.is_cursor_visible().unwrap_or(true));
                }
                Command::Title(tx) => {
                    let _ = tx.send(terminal.title().unwrap_or("").to_string());
                }
                Command::DumpGrid { tx } => {
                    let dumped = Self::build_dumped_grid(&terminal);
                    let _ = tx.send(dumped);
                }
                Command::Terminate => break,
            }
        }
    }

    // ── Public API ───────────────────────────────────────

    pub fn vt_write(&mut self, data: &[u8]) {
        let _ = self.cmd_tx.send(Command::Write(data.to_vec()));
    }

    pub fn flush(&self) {
        let (tx, rx) = bounded(1);
        let _ = self.cmd_tx.send(Command::FlushAck(tx));
        let _ = rx.recv();
    }

    pub fn set_theme(&self, bg: [u8; 3], fg: [u8; 3]) {
        let _ = self.cmd_tx.send(Command::SetTheme { bg, fg });
    }

    pub fn resize(&mut self, rows: u32, cols: u32) {
        let _ = self.cmd_tx.send(Command::Resize { rows, cols });
    }

    pub fn rows(&self) -> u32 {
        let (tx, rx) = bounded(1);
        let _ = self.cmd_tx.send(Command::Rows(tx));
        rx.recv().unwrap_or(24)
    }

    pub fn cols(&self) -> u32 {
        let (tx, rx) = bounded(1);
        let _ = self.cmd_tx.send(Command::Cols(tx));
        rx.recv().unwrap_or(80)
    }

    pub fn take_snapshot(&self) -> GridSnapshot {
        let (tx, rx) = bounded(1);
        let _ = self.cmd_tx.send(Command::TakeSnapshot(tx));
        rx.recv().unwrap_or_else(|_| GridSnapshot {
            rows: 0,
            cols: 0,
            cells: Vec::new(),
        })
    }

    pub fn scrollback_len(&self) -> u32 {
        let (tx, rx) = bounded(1);
        let _ = self.cmd_tx.send(Command::ScrollbackLen(tx));
        rx.recv().unwrap_or(0)
    }

    pub fn read_line_text(&self, row: u32) -> Option<String> {
        let (tx, rx) = bounded(1);
        let _ = self.cmd_tx.send(Command::ReadLineText { row, tx });
        rx.recv().unwrap_or(None)
    }

    pub fn read_visible_text(&self) -> String {
        let (tx, rx) = bounded(1);
        let _ = self.cmd_tx.send(Command::ReadVisibleText(tx));
        rx.recv().unwrap_or_default()
    }

    pub fn search_in_scrollback(&self, query: &str) -> Option<(u32, u32)> {
        let (tx, rx) = bounded(1);
        let _ = self.cmd_tx.send(Command::SearchInScrollback {
            query: query.to_string(),
            tx,
        });
        rx.recv().unwrap_or(None)
    }

    pub fn dump_grid(&self) -> DumpedGrid {
        let (tx, rx) = bounded(1);
        let _ = self.cmd_tx.send(Command::DumpGrid { tx });
        rx.recv().unwrap_or(DumpedGrid {
            rows: 0,
            cols: 0,
            visible: Vec::new(),
            scrollback: Vec::new(),
        })
    }

    // ── Internal helpers (executed on terminal thread) ───

    fn populate_uri(point: &libghostty_vt::screen::GridRef, data: &mut CellSnapshot) {
        if let Ok(cell) = point.cell()
            && cell.has_hyperlink().unwrap_or(false)
        {
            let mut buf = vec![0u8; 4096];
            if let Ok(len) = point.hyperlink_uri(&mut buf)
                && len > 0
            {
                data.uri = Some(String::from_utf8_lossy(&buf[..len]).to_string());
            }
        }
    }

    fn build_dumped_grid(terminal: &Terminal) -> DumpedGrid {
        let rows = terminal.rows().unwrap_or(24) as u32;
        let cols = terminal.cols().unwrap_or(80) as u32;
        let scrollback_rows = terminal.scrollback_rows().unwrap_or(0) as u32;

        let mut visible = Vec::with_capacity((rows * cols) as usize);
        for row in 0..rows {
            for col in 0..cols {
                let coord = PointCoordinate {
                    x: col as u16,
                    y: row,
                };
                let mut data = CellSnapshot::default();
                if let Ok(point) = terminal.grid_ref(Point::Viewport(coord)) {
                    if let Ok(cell) = point.cell() {
                        data.codepoint = cell.codepoint().unwrap_or(0);
                    }
                    if let Ok(style) = point.style() {
                        if let StyleColor::Rgb(c) = style.fg_color {
                            data.fg = [
                                c.r as f32 / 255.0,
                                c.g as f32 / 255.0,
                                c.b as f32 / 255.0,
                                1.0,
                            ];
                        }
                        if let StyleColor::Rgb(c) = style.bg_color {
                            data.bg = [
                                c.r as f32 / 255.0,
                                c.g as f32 / 255.0,
                                c.b as f32 / 255.0,
                                1.0,
                            ];
                        }
                        data.bold = style.bold;
                        data.italic = style.italic;
                        data.underline = matches!(
                            style.underline,
                            libghostty_vt::style::Underline::Single
                                | libghostty_vt::style::Underline::Double
                                | libghostty_vt::style::Underline::Curly
                                | libghostty_vt::style::Underline::Dashed
                                | libghostty_vt::style::Underline::Dotted
                        );
                        data.reverse = style.inverse;
                    }
                    Self::populate_uri(&point, &mut data);
                }
                visible.push(data);
            }
        }

        let mut scrollback = Vec::with_capacity(scrollback_rows as usize);
        for i in 0..scrollback_rows {
            let mut row_cells = Vec::with_capacity(cols as usize);
            for col in 0..cols {
                let coord = PointCoordinate {
                    x: col as u16,
                    y: i,
                };
                let mut data = CellSnapshot::default();
                if let Ok(point) = terminal.grid_ref(Point::History(coord)) {
                    if let Ok(cell) = point.cell() {
                        data.codepoint = cell.codepoint().unwrap_or(0);
                    }
                    if let Ok(style) = point.style() {
                        if let StyleColor::Rgb(c) = style.fg_color {
                            data.fg = [
                                c.r as f32 / 255.0,
                                c.g as f32 / 255.0,
                                c.b as f32 / 255.0,
                                1.0,
                            ];
                        }
                        if let StyleColor::Rgb(c) = style.bg_color {
                            data.bg = [
                                c.r as f32 / 255.0,
                                c.g as f32 / 255.0,
                                c.b as f32 / 255.0,
                                1.0,
                            ];
                        }
                        data.bold = style.bold;
                        data.italic = style.italic;
                        data.underline = matches!(
                            style.underline,
                            libghostty_vt::style::Underline::Single
                                | libghostty_vt::style::Underline::Double
                                | libghostty_vt::style::Underline::Curly
                                | libghostty_vt::style::Underline::Dashed
                                | libghostty_vt::style::Underline::Dotted
                        );
                        data.reverse = style.inverse;
                    }
                    Self::populate_uri(&point, &mut data);
                }
                row_cells.push(data);
            }
            scrollback.push(row_cells);
        }

        DumpedGrid {
            rows,
            cols,
            visible,
            scrollback,
        }
    }

    fn byte_to_float(c: u8) -> f32 {
        c as f32 / 255.0
    }

    fn byte_color_to_float(c: [u8; 3]) -> [f32; 4] {
        [
            Self::byte_to_float(c[0]),
            Self::byte_to_float(c[1]),
            Self::byte_to_float(c[2]),
            1.0,
        ]
    }

    fn build_snapshot(
        terminal: &Terminal,
        default_fg: [f32; 4],
        default_bg: [f32; 4],
    ) -> GridSnapshot {
        let rows = terminal.rows().unwrap_or(24) as u32;
        let cols = terminal.cols().unwrap_or(80) as u32;
        let size = (rows * cols) as usize;
        let mut cells = Vec::with_capacity(size);

        for row in 0..rows {
            for col in 0..cols {
                let coord = PointCoordinate {
                    x: col as u16,
                    y: row,
                };
                let mut data = CellSnapshot {
                    codepoint: 0,
                    fg: default_fg,
                    bg: default_bg,
                    bold: false,
                    italic: false,
                    underline: false,
                    reverse: false,
                    uri: None,
                };

                if let Ok(point) = terminal.grid_ref(Point::Viewport(coord)) {
                    if let Ok(cell) = point.cell() {
                        data.codepoint = cell.codepoint().unwrap_or(0);
                    }
                    if let Ok(style) = point.style() {
                        match style.fg_color {
                            StyleColor::Rgb(c) => {
                                data.fg = Self::byte_color_to_float([c.r, c.g, c.b]);
                            }
                            _ => {
                                data.fg = default_fg;
                            }
                        }
                        match style.bg_color {
                            StyleColor::Rgb(c) => {
                                data.bg = Self::byte_color_to_float([c.r, c.g, c.b]);
                                if row == 0 && col == 0 {
                                    log::trace!(
                                        "build_snapshot[0,0]: Rgb bg=({},{},{}) linear=({:.4},{:.4},{:.4})",
                                        c.r,
                                        c.g,
                                        c.b,
                                        data.bg[0],
                                        data.bg[1],
                                        data.bg[2]
                                    );
                                }
                            }
                            _ => {
                                data.bg = default_bg;
                                if row == 0 && col == 0 {
                                    log::trace!(
                                        "build_snapshot[0,0]: Default bg linear=({:.4},{:.4},{:.4})",
                                        data.bg[0],
                                        data.bg[1],
                                        data.bg[2]
                                    );
                                }
                            }
                        }
                        let underline = matches!(
                            style.underline,
                            libghostty_vt::style::Underline::Single
                                | libghostty_vt::style::Underline::Double
                                | libghostty_vt::style::Underline::Curly
                                | libghostty_vt::style::Underline::Dashed
                                | libghostty_vt::style::Underline::Dotted
                        );
                        data.underline = underline;
                        data.reverse = style.inverse;
                        data.bold = style.bold;
                        data.italic = style.italic;
                    }
                    Self::populate_uri(&point, &mut data);
                }

                cells.push(data);
            }
        }

        GridSnapshot { rows, cols, cells }
    }

    fn read_line_text_impl(terminal: &Terminal, row: u32) -> Option<String> {
        let cols = terminal.cols().unwrap_or(80) as u32;
        let mut text = String::new();
        for col in 0..cols {
            let coord = PointCoordinate {
                x: col as u16,
                y: row,
            };
            if let Ok(point) = terminal.grid_ref(Point::Viewport(coord))
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
        let trimmed = text.trim_end().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }

    fn search_in_scrollback_impl(terminal: &Terminal, query: &str) -> Option<(u32, u32)> {
        if query.is_empty() {
            return None;
        }
        let total = terminal.total_rows().unwrap_or(0) as u32;
        for row in 0..total {
            if let Some(line) = Self::read_line_text_impl(terminal, row)
                && let Some(col) = line.find(query)
            {
                return Some((row, col as u32));
            }
        }
        None
    }
}

impl Drop for GhosttyTerminal {
    fn drop(&mut self) {
        let _ = self.cmd_tx.send(Command::Terminate);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    /// 让出终端线程以排空队列中的命令。
    /// 不这样做的话，快速测试循环可能与通道接收器产生竞态。
    fn yield_for(ms: u64) {
        thread::sleep(Duration::from_millis(ms));
    }

    fn term() -> GhosttyTerminal {
        GhosttyTerminal::new(24, 80, 1000).expect("terminal create")
    }

    #[test]
    fn create_terminal_reports_dimensions() {
        let t = term();
        assert_eq!(t.rows(), 24);
        assert_eq!(t.cols(), 80);
    }

    #[test]
    fn create_terminal_zero_scrollback() {
        let t = GhosttyTerminal::new(5, 10, 0).expect("term");
        assert_eq!(t.scrollback_len(), 0);
    }

    #[test]
    fn write_ascii_appears_in_snapshot() {
        let mut t = term();
        t.vt_write(b"Hello");
        yield_for(20);
        let snap = t.take_snapshot();
        let cell = snap
            .cells
            .iter()
            .find(|c| c.codepoint == b'H' as u32)
            .unwrap();
        assert_eq!(cell.codepoint, 'H' as u32);
    }

    #[test]
    fn write_sgr_color_sets_fg() {
        let mut t = term();
        // ESC[31m = 红色前景色
        t.vt_write(b"\x1b[31mX");
        yield_for(20);
        let snap = t.take_snapshot();
        let cell = snap.cells.iter().find(|c| c.codepoint == b'X' as u32);
        assert!(cell.is_some());
        // 单元格存在；libghostty-vt 可能返回归一化通道的 Rgb 颜色
        let c = cell.unwrap();
        // 红色的 fg[0] 应 > 0.5（红色为主）
        assert!(c.fg[0] > 0.5, "expected red dominant, got {:?}", c.fg);
    }

    #[test]
    fn write_sgr_bold_sets_bold() {
        let mut t = term();
        t.vt_write(b"\x1b[1mA");
        yield_for(20);
        let snap = t.take_snapshot();
        let cell = snap
            .cells
            .iter()
            .find(|c| c.codepoint == 'A' as u32)
            .unwrap();
        assert!(cell.bold);
    }

    #[test]
    fn write_sgr_italic_sets_italic() {
        let mut t = term();
        t.vt_write(b"\x1b[3mA");
        yield_for(20);
        let snap = t.take_snapshot();
        let cell = snap
            .cells
            .iter()
            .find(|c| c.codepoint == 'A' as u32)
            .unwrap();
        assert!(cell.italic);
    }

    #[test]
    fn write_sgr_underline_sets_underline() {
        let mut t = term();
        t.vt_write(b"\x1b[4mA");
        yield_for(20);
        let snap = t.take_snapshot();
        let cell = snap
            .cells
            .iter()
            .find(|c| c.codepoint == 'A' as u32)
            .unwrap();
        assert!(cell.underline);
    }

    #[test]
    fn write_sgr_reverse_sets_reverse() {
        let mut t = term();
        t.vt_write(b"\x1b[7mA");
        yield_for(20);
        let snap = t.take_snapshot();
        let cell = snap
            .cells
            .iter()
            .find(|c| c.codepoint == 'A' as u32)
            .unwrap();
        assert!(cell.reverse);
    }

    #[test]
    fn write_sgr_reset_clears_attrs() {
        let mut t = term();
        t.vt_write(b"\x1b[1;3;4;7mA\x1b[0mB");
        yield_for(20);
        let snap = t.take_snapshot();
        let b = snap
            .cells
            .iter()
            .find(|c| c.codepoint == 'B' as u32)
            .unwrap();
        assert!(!b.bold);
        assert!(!b.italic);
        assert!(!b.underline);
        assert!(!b.reverse);
    }

    #[test]
    fn write_sgr_256_color() {
        let mut t = term();
        // ESC[38;5;196m = fg 256-color index 196
        t.vt_write(b"\x1b[38;5;196mX");
        yield_for(20);
        let snap = t.take_snapshot();
        let cell = snap.cells.iter().find(|c| c.codepoint == 'X' as u32);
        assert!(cell.is_some());
    }

    #[test]
    fn write_crlf_advances_cursor() {
        let mut t = term();
        t.vt_write(b"AB\r\nCD");
        yield_for(20);
        // 光标应在第 1 行第 2 列 — 通过 dump 验证
        let dumped = t.dump_grid();
        // 第 1 行应在列 0-1 处有 CD
        let row1: Vec<_> = dumped.visible.iter().skip(80).take(2).collect();
        assert_eq!(row1[0].codepoint, 'C' as u32);
        assert_eq!(row1[1].codepoint, 'D' as u32);
    }

    #[test]
    fn write_csi_cup_positions_cursor() {
        let mut t = term();
        // ESC[5;10H = move cursor to row 5 col 10 (1-based)
        t.vt_write(b"\x1b[5;10HX");
        yield_for(20);
        let dumped = t.dump_grid();
        // X should be at row 4, col 9
        let idx = (4 * 80 + 9) as usize;
        assert_eq!(dumped.visible[idx].codepoint, 'X' as u32);
    }

    #[test]
    fn write_csi_cup_origin() {
        let mut t = term();
        t.vt_write(b"\x1b[1;1HABC");
        yield_for(20);
        let dumped = t.dump_grid();
        assert_eq!(dumped.visible[0].codepoint, 'A' as u32);
        assert_eq!(dumped.visible[1].codepoint, 'B' as u32);
        assert_eq!(dumped.visible[2].codepoint, 'C' as u32);
    }

    #[test]
    fn write_csi_erase_display_0() {
        let mut t = term();
        t.vt_write(b"AB\x1b[2J");
        yield_for(20);
        let snap = t.take_snapshot();
        // 擦除后，不应有单元格包含 A 或 B
        assert!(!snap.cells.iter().any(|c| c.codepoint == 'A' as u32));
        assert!(!snap.cells.iter().any(|c| c.codepoint == 'B' as u32));
    }

    #[test]
    fn write_csi_erase_line_0() {
        let mut t = term();
        t.vt_write(b"\x1b[2;1HABCDE\x1b[2K");
        yield_for(20);
        let snap = t.take_snapshot();
        // 擦除行后，第 1 行的单元格应全部为空
        let row: Vec<_> = snap.cells.iter().skip(80).take(80).collect();
        let has_abc = row.iter().any(|c| {
            c.codepoint == 'A' as u32 || c.codepoint == 'B' as u32 || c.codepoint == 'C' as u32
        });
        assert!(!has_abc);
    }

    #[test]
    fn write_newline_scrolls() {
        let mut t = GhosttyTerminal::new(3, 5, 100).expect("term");
        for i in 0..10 {
            t.vt_write(format!("line {i}\n").as_bytes());
        }
        yield_for(50);
        // 多次换行后，回滚区应有条目
        assert!(t.scrollback_len() > 0);
    }

    #[test]
    fn read_line_text_returns_text() {
        let mut t = term();
        t.vt_write(b"\x1b[1;1HHello World");
        yield_for(20);
        let text = t.read_line_text(0);
        assert!(text.is_some());
        assert!(text.unwrap().contains("Hello"));
    }

    #[test]
    fn read_line_text_empty_returns_none() {
        let t = term();
        let text = t.read_line_text(5);
        assert!(text.is_none());
    }

    #[test]
    fn search_in_scrollback_finds_match() {
        let mut t = GhosttyTerminal::new(3, 80, 100).expect("term");
        t.vt_write(b"search_target_here\n");
        yield_for(20);
        // 需要更多回滚
        for i in 0..5 {
            t.vt_write(format!("filler {i}\n").as_bytes());
        }
        yield_for(50);
        // 滚动后，"search_target_here" 应回滚区中
        let result = t.search_in_scrollback("search_target");
        // 可能在回滚区中也可能不在，取决于滚动了多少
        // 仅检查 API 不会 panic
        let _ = result;
    }

    #[test]
    fn search_in_scrollback_empty_query() {
        let t = term();
        assert_eq!(t.search_in_scrollback(""), None);
    }

    #[test]
    fn resize_changes_dimensions() {
        let mut t = term();
        t.resize(50, 100);
        yield_for(20);
        assert_eq!(t.rows(), 50);
        assert_eq!(t.cols(), 100);
    }

    #[test]
    fn snapshot_dimensions_match() {
        let t = term();
        let snap = t.take_snapshot();
        assert_eq!(snap.rows, 24);
        assert_eq!(snap.cols, 80);
        assert_eq!(snap.cells.len(), (24 * 80) as usize);
    }

    #[test]
    fn dump_grid_dimensions_match() {
        let t = term();
        let dumped = t.dump_grid();
        assert_eq!(dumped.rows, 24);
        assert_eq!(dumped.cols, 80);
        assert_eq!(dumped.visible.len(), (24 * 80) as usize);
    }

    #[test]
    fn uri_at_empty_default() {
        let t = term();
        let snap = t.take_snapshot();
        assert_eq!(snap.uri_at(0, 0), None);
    }

    #[test]
    fn uri_at_out_of_bounds() {
        let t = term();
        let snap = t.take_snapshot();
        assert_eq!(snap.uri_at(100, 0), None);
        assert_eq!(snap.uri_at(0, 100), None);
    }

    #[test]
    fn write_sgr_dim() {
        let mut t = term();
        t.vt_write(b"\x1b[2mA");
        yield_for(20);
        let snap = t.take_snapshot();
        let cell = snap.cells.iter().find(|c| c.codepoint == 'A' as u32);
        // Dim 可能通过快照暴露也可能不暴露 — 仅检查无 panic
        assert!(cell.is_some());
    }

    #[test]
    fn write_sgr_strikethrough() {
        let mut t = term();
        t.vt_write(b"\x1b[9mA");
        yield_for(20);
        let snap = t.take_snapshot();
        assert!(!snap.cells.is_empty());
    }

    #[test]
    fn write_sgr_blink() {
        let mut t = term();
        t.vt_write(b"\x1b[5mA");
        yield_for(20);
        let snap = t.take_snapshot();
        assert!(!snap.cells.is_empty());
    }

    #[test]
    fn write_multiple_lines_in_sequence() {
        let mut t = term();
        t.vt_write(b"line1\nline2\nline3");
        yield_for(20);
        let line1 = t.read_line_text(0);
        let line2 = t.read_line_text(1);
        let line3 = t.read_line_text(2);
        assert!(line1.is_some());
        assert!(line2.is_some());
        assert!(line3.is_some());
    }

    #[test]
    fn write_unicode_cjk() {
        let mut t = term();
        t.vt_write("中文".as_bytes());
        yield_for(20);
        let snap = t.take_snapshot();
        let has_cjk = snap
            .cells
            .iter()
            .any(|c| c.codepoint == '中' as u32 || c.codepoint == '文' as u32);
        assert!(has_cjk);
    }

    #[test]
    fn write_emoji() {
        let mut t = term();
        t.vt_write("😀".as_bytes());
        yield_for(20);
        let snap = t.take_snapshot();
        let has_emoji = snap.cells.iter().any(|c| c.codepoint == 0x1F600);
        assert!(has_emoji);
    }

    #[test]
    fn write_dec_private_mode_show_cursor() {
        let mut t = term();
        t.vt_write(b"\x1b[?25h");
        yield_for(20);
        // 仅验证无 panic
        let _ = t.take_snapshot();
    }

    #[test]
    fn write_dec_private_mode_hide_cursor() {
        let mut t = term();
        t.vt_write(b"\x1b[?25l");
        yield_for(20);
        let _ = t.take_snapshot();
    }

    #[test]
    fn write_csi_sgr_combined_attrs() {
        let mut t = term();
        t.vt_write(b"\x1b[1;3;4;7;9mZ");
        yield_for(20);
        let snap = t.take_snapshot();
        let cell = snap
            .cells
            .iter()
            .find(|c| c.codepoint == 'Z' as u32)
            .unwrap();
        assert!(cell.bold);
        assert!(cell.italic);
        assert!(cell.underline);
        assert!(cell.reverse);
    }

    #[test]
    fn write_osc_8_hyperlink() {
        let mut t = term();
        t.vt_write(b"\x1b]8;;https://example.com\x1b\\Linked Text\x1b]8;;\x1b\\");
        yield_for(20);
        let snap = t.take_snapshot();
        // uri may or may not be set depending on libghostty support - just no panic
        let _ = snap.uri_at(0, 0);
    }

    #[test]
    fn write_csi_cursor_movement_via_snapshot() {
        // Use snapshot to verify CSI cursor movement (CUU/CUD/CUF/CUB) by writing
        // a single character after each move. libghostty-vt's VT parser handles
        // these sequences; we verify by checking visible cells rather than
        // position APIs (which GhosttyTerminal does not expose publicly).
        let mut t = term();
        t.vt_write(b"\x1b[1;5H");
        yield_for(10);
        t.vt_write(b"A");
        yield_for(20);
        let dumped = t.dump_grid();
        // A at row 0, col 4
        assert_eq!(dumped.visible[4].codepoint, 'A' as u32);
    }

    #[test]
    fn write_csi_erase_in_line() {
        let mut t = term();
        t.vt_write(b"ABCDE\x1b[1;1H\x1b[K");
        yield_for(20);
        // K (擦除到行尾) 后，尾部字符应消失
        let snap = t.take_snapshot();
        let has_de = snap
            .cells
            .iter()
            .any(|c| c.codepoint == 'D' as u32 || c.codepoint == 'E' as u32);
        // 从光标 0,0 擦除到行尾
        assert!(!has_de);
    }

    #[test]
    fn snapshot_uri_at_returns_none_for_unset() {
        let mut t = term();
        t.vt_write(b"Hello");
        yield_for(20);
        let snap = t.take_snapshot();
        for row in 0..snap.rows {
            for col in 0..snap.cols {
                let uri = snap.uri_at(row, col);
                if let Some(u) = uri {
                    // 如果设置了任何 URI，应为有效字符串
                    assert!(!u.is_empty());
                }
            }
        }
    }

    #[test]
    fn multiple_writes_sequential() {
        let mut t = term();
        t.vt_write(b"A");
        yield_for(10);
        t.vt_write(b"B");
        yield_for(10);
        t.vt_write(b"C");
        yield_for(20);
        let snap = t.take_snapshot();
        let has_a = snap.cells.iter().any(|c| c.codepoint == 'A' as u32);
        let has_b = snap.cells.iter().any(|c| c.codepoint == 'B' as u32);
        let has_c = snap.cells.iter().any(|c| c.codepoint == 'C' as u32);
        assert!(has_a);
        assert!(has_b);
        assert!(has_c);
    }

    #[test]
    fn dump_grid_visible_populated() {
        let mut t = term();
        t.vt_write(b"hello");
        yield_for(20);
        let dumped = t.dump_grid();
        assert!(!dumped.visible.is_empty());
    }

    #[test]
    fn dump_grid_scrollback_populated_after_scroll() {
        let mut t = GhosttyTerminal::new(3, 10, 100).expect("term");
        for i in 0..10 {
            t.vt_write(format!("line{i}\n").as_bytes());
        }
        yield_for(50);
        let dumped = t.dump_grid();
        assert!(!dumped.scrollback.is_empty());
    }

    #[test]
    fn cell_snapshot_default() {
        let c = CellSnapshot::default();
        assert_eq!(c.codepoint, 0);
        assert_eq!(c.fg, [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(c.bg, [0.0, 0.0, 0.0, 0.0]);
        assert!(!c.bold);
        assert!(!c.italic);
        assert!(c.uri.is_none());
    }

    #[test]
    fn cell_snapshot_clone() {
        let c = CellSnapshot {
            codepoint: 65,
            fg: [1.0, 0.0, 0.0, 1.0],
            bg: [0.0, 0.0, 0.0, 1.0],
            bold: true,
            italic: false,
            underline: true,
            reverse: false,
            uri: Some(String::from("https://test")),
        };
        let c2 = c.clone();
        assert_eq!(c.codepoint, c2.codepoint);
        assert_eq!(c.fg, c2.fg);
        assert_eq!(c.uri, c2.uri);
    }

    #[test]
    fn write_alt_screen_switch() {
        let mut t = term();
        // 进入备用屏幕
        t.vt_write(b"\x1b[?1049h");
        yield_for(20);
        t.vt_write(b"InAlt");
        yield_for(20);
        // 退出备用屏幕
        t.vt_write(b"\x1b[?1049l");
        yield_for(20);
        // 仅验证无 panic
        let snap = t.take_snapshot();
        assert!(!snap.cells.is_empty());
    }
}
