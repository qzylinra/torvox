use std::thread;

use flume::{Receiver, Sender, bounded};
use libghostty_vt::style::StyleColor;
use libghostty_vt::terminal::{Point, PointCoordinate};
use libghostty_vt::{Terminal, TerminalOptions};

/// Snapshot of the terminal grid for rendering.
/// Built on the terminal thread; consumed by the renderer thread.
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

/// Thread-safe wrapper around libghostty_vt::Terminal.
/// The terminal lives on a dedicated thread; operations are serialized through a flume channel.
/// No unsafe Send/Sync impls needed — GhosttyTerminal holds only `Sender<Command>` (Send + Sync).
pub struct GhosttyTerminal {
    cmd_tx: Sender<Command>,
    handle: Option<thread::JoinHandle<()>>,
}

impl GhosttyTerminal {
    pub fn new(rows: u32, cols: u32, scrollback_lines: u32) -> Result<Self, String> {
        let (cmd_tx, cmd_rx) = bounded::<Command>(256);
        let handle = thread::Builder::new()
            .name("ghostty-terminal".into())
            .spawn(move || Self::run(cmd_rx, rows, cols, scrollback_lines))
            .map_err(|e| format!("failed to spawn terminal thread: {e}"))?;

        Ok(Self {
            cmd_tx,
            handle: Some(handle),
        })
    }

    fn run(rx: Receiver<Command>, rows: u32, cols: u32, scrollback_lines: u32) {
        let Ok(mut terminal) = Terminal::new(TerminalOptions {
            cols: cols as u16,
            rows: rows as u16,
            max_scrollback: scrollback_lines as usize,
        }) else {
            return;
        };

        while let Ok(cmd) = rx.recv() {
            match cmd {
                Command::Write(data) => terminal.vt_write(&data),
                Command::Resize { rows, cols } => {
                    let _ = terminal.resize(cols as u16, rows as u16, 8, 16);
                }
                Command::TakeSnapshot(tx) => {
                    let snapshot = Self::build_snapshot(&terminal);
                    let _ = tx.send(snapshot);
                }
                Command::ScrollbackLen(tx) => {
                    let _ = tx.send(terminal.scrollback_rows().unwrap_or(0) as u32);
                }
                Command::ReadLineText { row, tx } => {
                    let text = Self::read_line_text_impl(&terminal, row);
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
            let mut buf = [0u8; 4096];
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

    fn build_snapshot(terminal: &Terminal) -> GridSnapshot {
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
                    fg: [1.0, 1.0, 1.0, 1.0],
                    bg: [0.0, 0.0, 0.0, 1.0],
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
