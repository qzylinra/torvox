use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use thiserror::Error;
use torvox_core::line::Line;
use torvox_renderer::font::FontPipeline;
use torvox_renderer::gpu::GpuContext;
use torvox_terminal::ghostty_terminal::{CellSnapshot, GhosttyTerminal};
use torvox_terminal::session::Session;

#[derive(Debug, Error)]
pub enum SurfaceError {
    #[error("GPU initialization failed: {0}")]
    GpuInit(String),
    #[error("Surface creation failed: {0}")]
    SurfaceCreation(String),
    #[error("No surface available")]
    NoSurface,
    #[error("Render error: {0}")]
    Render(String),
    #[error("session error: {0}")]
    Session(String),
}

pub struct AndroidSurface {
    gpu: GpuContext,
    font_pipeline: FontPipeline,
    session: Option<Session>,
    atlas_width: u32,
    atlas_height: u32,
    theme: torvox_core::config::Theme,
    rows: u32,
    cols: u32,
    exited: Arc<AtomicBool>,
    save_path: Option<PathBuf>,
    mouse_row: Option<u32>,
    mouse_col: Option<u32>,
    last_hovered_url: Option<String>,
}

fn cell_to_line(cells: &[CellSnapshot], cols: u32) -> Line {
    let mut line = Line::new(cols);
    for col in 0..cols as usize {
        if let Some(cs) = cells.get(col)
            && let Some(cell) = line.get_mut(col as u32)
        {
            cell.char = char::from_u32(cs.codepoint).unwrap_or(' ');
            cell.fg = torvox_core::cell::Color {
                r: (cs.fg[0] * 255.0) as u8,
                g: (cs.fg[1] * 255.0) as u8,
                b: (cs.fg[2] * 255.0) as u8,
                a: (cs.fg[3] * 255.0) as u8,
            };
            cell.bg = torvox_core::cell::Color {
                r: (cs.bg[0] * 255.0) as u8,
                g: (cs.bg[1] * 255.0) as u8,
                b: (cs.bg[2] * 255.0) as u8,
                a: (cs.bg[3] * 255.0) as u8,
            };
            cell.attrs.bold = cs.bold;
            cell.attrs.italic = cs.italic;
            cell.attrs.underline = cs.underline;
            cell.attrs.reverse = cs.reverse;
        }
    }
    line
}

fn line_to_text(line: &Line) -> String {
    (0..line.len())
        .filter_map(|c| line.get(c))
        .map(|cell| cell.char)
        .collect()
}

impl AndroidSurface {
    pub fn new(rows: u32, cols: u32, _scrollback_lines: u32) -> Self {
        let atlas_width = 2048;
        let atlas_height = 2048;
        let mut font_pipeline = FontPipeline::new(atlas_width as i32, atlas_height as i32, 14.0);
        font_pipeline.rasterize_ascii();

        Self {
            gpu: GpuContext::new_with_no_surface(),
            font_pipeline,
            session: None,
            atlas_width,
            atlas_height,
            theme: torvox_core::config::Theme::catppuccin_mocha(),
            rows,
            cols,
            exited: Arc::new(AtomicBool::new(false)),
            save_path: None,
            mouse_row: None,
            mouse_col: None,
            last_hovered_url: None,
        }
    }

    pub fn set_save_path(&mut self, path: String) {
        self.save_path = Some(PathBuf::from(path));
    }

    pub fn spawn_session(&mut self, shell: &str) -> Result<(), SurfaceError> {
        let session = Session::spawn(shell, self.rows, self.cols)
            .map_err(|e| SurfaceError::Session(e.to_string()))?;
        self.exited = session.exited_flag().clone();
        self.session = Some(session);
        Ok(())
    }

    pub fn set_native_window(
        &mut self,
        window_ptr: *mut std::ffi::c_void,
        width: u32,
        height: u32,
    ) -> Result<(), SurfaceError> {
        self.gpu
            .set_surface_from_native_window(window_ptr, width, height)
            .map_err(|e| SurfaceError::SurfaceCreation(e.to_string()))?;

        self.gpu
            .create_atlas_texture(self.atlas_width, self.atlas_height);

        let (aw, ah) = self.font_pipeline.atlas_dimensions();
        let (cw, ch) = self.font_pipeline.cell_metrics();
        self.gpu.update_bind_group(aw as f32, ah as f32, cw, ch);

        let atlas_data = self.font_pipeline.atlas_bitmap().to_vec();
        self.gpu.upload_atlas(&atlas_data, aw, ah);

        Ok(())
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        if let Some(session) = &mut self.session {
            session.process_output();
        }

        let snapshot = self
            .session
            .as_ref()
            .map(|s| s.terminal().take_snapshot())
            .ok_or(SurfaceError::NoSurface)?;

        if let (Some(row), Some(col)) = (self.mouse_row, self.mouse_col) {
            self.last_hovered_url = snapshot.uri_at(row, col).map(String::from);
        }

        let instances = torvox_renderer::gpu::build_cell_instances_from_snapshot(
            &snapshot,
            &mut self.font_pipeline,
            self.atlas_width as f32,
            self.atlas_height as f32,
            None,
        );

        self.gpu
            .render_frame(&instances)
            .map_err(|e| SurfaceError::Render(e.to_string()))
    }

    pub fn terminal(&self) -> &GhosttyTerminal {
        self.session
            .as_ref()
            .map(|s| s.terminal())
            .expect("no session")
    }

    pub fn terminal_mut(&mut self) -> &mut GhosttyTerminal {
        self.session
            .as_mut()
            .map(|s| s.terminal_mut())
            .expect("no session")
    }

    pub fn font_pipeline(&self) -> &FontPipeline {
        &self.font_pipeline
    }

    pub fn resize(&mut self, rows: u32, cols: u32) {
        self.rows = rows;
        self.cols = cols;
        if let Some(session) = &mut self.session {
            let _ = session.resize(rows, cols);
        }
    }

    pub fn write_to_pty(&mut self, data: &[u8]) {
        if let Some(session) = &mut self.session {
            let _ = session.write(data);
        }
    }

    pub fn is_exited(&self) -> bool {
        self.exited.load(Ordering::Acquire)
    }

    pub fn has_session(&self) -> bool {
        self.session.is_some()
    }

    pub fn set_font_size(&mut self, size: f32) {
        self.font_pipeline =
            FontPipeline::new(self.atlas_width as i32, self.atlas_height as i32, size);
        self.font_pipeline.rasterize_ascii();
        let (aw, ah) = self.font_pipeline.atlas_dimensions();
        let (cw, ch) = self.font_pipeline.cell_metrics();
        self.gpu.update_bind_group(aw as f32, ah as f32, cw, ch);
        let atlas_data = self.font_pipeline.atlas_bitmap().to_vec();
        self.gpu.upload_atlas(&atlas_data, aw, ah);
    }

    pub fn set_mouse_position(&mut self, row: u32, col: u32) {
        self.mouse_row = Some(row);
        self.mouse_col = Some(col);
    }

    pub fn get_hovered_url(&self) -> Option<String> {
        self.last_hovered_url.clone()
    }

    pub fn set_theme(&mut self, theme: torvox_core::config::Theme) {
        self.theme = theme;
    }

    pub fn theme(&self) -> &torvox_core::config::Theme {
        &self.theme
    }

    pub fn save_session(&self, path: &str) -> Result<(), SurfaceError> {
        use std::fs;
        use torvox_core::snapshot::SessionSnapshot;

        let dumped = self.terminal().dump_grid();
        let (rows, cols) = (dumped.rows, dumped.cols);

        let mut visible_lines = Vec::with_capacity(rows as usize);
        for row in 0..rows as usize {
            let start = row * cols as usize;
            let row_cells = &dumped.visible[start..start + cols as usize];
            visible_lines.push(cell_to_line(row_cells, cols));
        }

        let mut scrollback_lines = Vec::with_capacity(dumped.scrollback.len());
        for sb_row in &dumped.scrollback {
            scrollback_lines.push(cell_to_line(sb_row, cols));
        }

        let snapshot = SessionSnapshot {
            visible_lines,
            scrollback_lines,
            rows,
            cols,
            max_scrollback: 2000,
        };

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&snapshot)
            .map_err(|e| SurfaceError::Session(format!("rkyv serialize: {e}")))?;
        fs::write(path, &bytes).map_err(|e| SurfaceError::Session(format!("write failed: {e}")))?;
        Ok(())
    }

    pub fn restore_session(&mut self, path: &str) -> Result<(), SurfaceError> {
        use rkyv::rancor;
        use std::fs;
        use torvox_core::snapshot::SessionSnapshot;

        let data =
            fs::read(path).map_err(|e| SurfaceError::Session(format!("read failed: {e}")))?;
        let snapshot = rkyv::from_bytes::<SessionSnapshot, rancor::Error>(&data)
            .map_err(|e| SurfaceError::Session(format!("rkyv deserialize: {e}")))?;

        if let Some(session) = &mut self.session {
            let mut text = String::new();
            for line in snapshot
                .scrollback_lines
                .iter()
                .chain(&snapshot.visible_lines)
            {
                let trimmed = line_to_text(line).trim_end().to_string();
                if !trimmed.is_empty() {
                    text.push_str(&trimmed);
                    text.push('\n');
                }
            }
            if !text.is_empty() {
                session.terminal_mut().vt_write(text.as_bytes());
            }
        }

        let _ = fs::remove_file(path);
        Ok(())
    }

    pub fn has_saved_session(path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
}

impl Drop for AndroidSurface {
    fn drop(&mut self) {
        if let Some(path) = self.save_path.as_ref()
            && self.session.is_some()
        {
            let _ = self.save_session(&path.to_string_lossy());
        }
        self.session.take();
        if self.gpu.has_surface() {
            self.gpu.warmup();
        }
    }
}
