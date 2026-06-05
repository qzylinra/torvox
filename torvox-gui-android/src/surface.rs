use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use thiserror::Error;
use torvox_core::line::Line;
use torvox_renderer::font::FontPipeline;
use torvox_renderer::gpu::GpuContext;
use torvox_terminal::ghostty_terminal::{CellSnapshot, GhosttyTerminal};
use torvox_terminal::session::Session;

// ANativeWindow NDK API for software rendering
#[cfg(target_os = "android")]
#[repr(C)]
struct NativeBuffer {
    width: i32,
    height: i32,
    stride: i32,
    format: i32,
    bits: *mut std::ffi::c_void,
    reserved: [u32; 6],
}

#[cfg(target_os = "android")]
#[link(name = "android")]
unsafe extern "C" {
    fn ANativeWindow_lock(
        window: *mut std::ffi::c_void,
        buffer: *mut NativeBuffer,
        dirty: *const std::ffi::c_void,
    ) -> i32;
    fn ANativeWindow_unlockAndPost(window: *mut std::ffi::c_void) -> i32;
    fn ANativeWindow_getWidth(window: *mut std::ffi::c_void) -> i32;
    fn ANativeWindow_getHeight(window: *mut std::ffi::c_void) -> i32;
    fn ANativeWindow_setBuffersGeometry(
        window: *mut std::ffi::c_void,
        width: i32,
        height: i32,
        format: i32,
    ) -> i32;
}

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

/// Wrap `NonNull<c_void>` for `Send + Sync` (ANativeWindow pointer is thread-safe on Android).
#[allow(dead_code)]
struct NativeWindow(std::ptr::NonNull<std::ffi::c_void>);
unsafe impl Send for NativeWindow {}
unsafe impl Sync for NativeWindow {}

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
    surface_width: u32,
    surface_height: u32,
    native_window: Option<NativeWindow>,
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

#[cfg(target_os = "android")]
fn to_u8(v: f32) -> u8 {
    (v * 255.0).clamp(0.0, 255.0) as u8
}

impl AndroidSurface {
    pub fn new(rows: u32, cols: u32, _scrollback_lines: u32) -> Self {
        let atlas_width = 2048;
        let atlas_height = 2048;
        let font_pipeline = FontPipeline::new(atlas_width as i32, atlas_height as i32, 14.0);

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
            surface_width: 0,
            surface_height: 0,
            native_window: None,
        }
    }

    pub fn set_save_path(&mut self, path: String) {
        self.save_path = Some(PathBuf::from(path));
    }

    pub fn spawn_session(&mut self, shell: &str) -> Result<(), SurfaceError> {
        let (bg, fg) = (self.theme.bg, self.theme.fg);
        let session = Session::spawn_with_theme(shell, self.rows, self.cols, bg, fg)
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
        self.native_window = std::ptr::NonNull::new(window_ptr).map(NativeWindow);
        self.surface_width = width;
        self.surface_height = height;

        // Compute font size to fill surface with the terminal grid
        let font_size = ((width as f32 / self.cols as f32) * 1.6).max(24.0);

        self.font_pipeline =
            FontPipeline::new(self.atlas_width as i32, self.atlas_height as i32, font_size);
        self.font_pipeline.rasterize_ascii();
        let (_aw, _ah) = self.font_pipeline.atlas_dimensions();
        let (cw, ch) = self.font_pipeline.cell_metrics();

        // Compute grid dimensions to fill the surface
        let cols = (width as f32 / cw).floor().clamp(20.0, 300.0) as u32;
        let rows = (height as f32 / ch).floor().clamp(5.0, 200.0) as u32;
        self.rows = rows;
        self.cols = cols;

        log::info!(
            "SURFACE_SET_NATIVE_WINDOW: grid={}x{} surface={}x{}",
            rows,
            cols,
            width,
            height,
        );

        #[cfg(not(target_os = "android"))]
        {
            // Desktop: use wgpu GPU rendering
            self.gpu
                .set_surface_from_native_window(window_ptr, width, height)
                .map_err(|e| SurfaceError::GpuInit(e.to_string()))?;
        }

        let (aw, ah) = self.font_pipeline.atlas_dimensions();
        let (cw, ch) = self.font_pipeline.cell_metrics();
        log::info!(
            "SURFACE_CELL_METRICS: font_size={} cell={:.1}x{:.1} grid={}x{} surface={}x{} atlas={}x{}",
            font_size,
            cw,
            ch,
            rows,
            cols,
            width,
            height,
            aw,
            ah,
        );

        #[cfg(not(target_os = "android"))]
        {
            self.gpu.create_atlas_texture(aw, ah);
            let atlas_data = self.font_pipeline.atlas_bitmap().to_vec();
            self.gpu.upload_atlas(&atlas_data, aw, ah);
            self.gpu.update_bind_group(aw as f32, ah as f32, cw, ch);
        }

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

        // Always rasterize glyphs so the atlas is populated for software rendering
        let gen_before = self.font_pipeline.atlas_generation();
        let instances = torvox_renderer::gpu::build_cell_instances_from_snapshot(
            &snapshot,
            &mut self.font_pipeline,
            self.atlas_width as f32,
            self.atlas_height as f32,
            None,
        );
        let gen_after = self.font_pipeline.atlas_generation();
        if gen_after > gen_before {
            let atlas_data = self.font_pipeline.atlas_bitmap().to_vec();
            let non_zero = atlas_data.iter().filter(|&&b| b != 0).count();
            log::trace!(
                "atlas re-upload gen={}->{} non_zero_pixels={}",
                gen_before,
                gen_after,
                non_zero,
            );
            #[cfg(not(target_os = "android"))]
            self.gpu
                .upload_atlas(&atlas_data, self.atlas_width, self.atlas_height);
        }

        #[cfg(target_os = "android")]
        {
            // Android: software rendering via ANativeWindow_lock
            // wgpu Fifo present mode blocks ~37s on SwiftShader emulator.
            drop(instances);
            self.render_software_with_snapshot(&snapshot)
        }

        #[cfg(not(target_os = "android"))]
        {
            log::trace!(
                "SURFACE_RENDER: snapshot={}x{} instances={}",
                snapshot.cols,
                snapshot.rows,
                instances.len(),
            );
            self.gpu
                .render_frame(&instances)
                .map_err(|e| SurfaceError::Render(e.to_string()))
        }
    }

    #[cfg(target_os = "android")]
    fn render_software_with_snapshot(
        &mut self,
        snapshot: &torvox_terminal::ghostty_terminal::GridSnapshot,
    ) -> Result<(), SurfaceError> {
        let window_ptr = self
            .native_window
            .as_ref()
            .map(|p| p.0.as_ptr())
            .ok_or(SurfaceError::NoSurface)?;

        let w = self.surface_width as i32;
        let h = self.surface_height as i32;

        unsafe {
            ANativeWindow_setBuffersGeometry(window_ptr, w, h, 1);
        }

        let mut buffer = NativeBuffer {
            width: 0,
            height: 0,
            stride: 0,
            format: 0,
            bits: std::ptr::null_mut(),
            reserved: [0; 6],
        };

        let result = unsafe { ANativeWindow_lock(window_ptr, &mut buffer, std::ptr::null()) };
        if result != 0 {
            log::error!("ANativeWindow_lock failed: {}", result);
            return Err(SurfaceError::Render(format!("lock failed: {}", result)));
        }

        let bw = buffer.width as usize;
        let bh = buffer.height as usize;
        let stride = buffer.stride as usize;
        let bits = buffer.bits as *mut u8;

        if bits.is_null() || bw == 0 || bh == 0 {
            unsafe { ANativeWindow_unlockAndPost(window_ptr) };
            return Err(SurfaceError::Render("invalid buffer".to_string()));
        }

        let cells_w = self.cols as usize;
        let cells_h = self.rows as usize;
        let cell_w = bw / cells_w.max(1);
        let cell_h = bh / cells_h.max(1);

        let atlas_bitmap = self.font_pipeline.atlas_bitmap().to_vec();
        let (atlas_w, atlas_h) = self.font_pipeline.atlas_dimensions();
        let atlas_w = atlas_w as usize;
        let atlas_h = atlas_h as usize;

        for row in 0..cells_h {
            for col in 0..cells_w {
                let idx = (row * cells_w + col).min(snapshot.cells.len() - 1);
                let cell = &snapshot.cells[idx];

                let px = col * cell_w;
                let py = row * cell_h;

                let bg_r = to_u8(cell.bg[0]);
                let bg_g = to_u8(cell.bg[1]);
                let bg_b = to_u8(cell.bg[2]);

                for cy in py..(py + cell_h).min(bh) {
                    let row_start = cy * stride;
                    for cx in px..(px + cell_w).min(bw) {
                        let p = row_start + cx * 4;
                        unsafe {
                            *bits.add(p) = bg_r;
                            *bits.add(p + 1) = bg_g;
                            *bits.add(p + 2) = bg_b;
                            *bits.add(p + 3) = 255;
                        }
                    }
                }

                if cell.codepoint != 0 && cell.codepoint != 0x20 {
                    let ch = char::from_u32(cell.codepoint).unwrap_or('\u{FFFD}');
                    if let Some(info) = self.font_pipeline.glyph_info(ch) {
                        if info.width > 0 && info.height > 0 {
                            let fg_r = to_u8(cell.fg[0]);
                            let fg_g = to_u8(cell.fg[1]);
                            let fg_b = to_u8(cell.fg[2]);

                            let gly_x = info.atlas_x as usize;
                            let gly_y = info.atlas_y as usize;
                            let gly_w = info.width as usize;
                            let gly_h = info.height as usize;

                            let off_x = (cell_w.saturating_sub(gly_w)) / 2;
                            let off_y = (cell_h.saturating_sub(gly_h)) / 2;

                            for gy in 0..gly_h {
                                let dst_y = py + off_y + gy;
                                if dst_y >= bh {
                                    break;
                                }
                                for gx in 0..gly_w {
                                    let dst_x = px + off_x + gx;
                                    if dst_x >= bw {
                                        break;
                                    }
                                    let atlas_idx = (gly_y + gy) * atlas_w * 4 + (gly_x + gx) * 4;
                                    let alpha = if atlas_idx + 3 < atlas_bitmap.len() {
                                        atlas_bitmap[atlas_idx + 3]
                                    } else {
                                        0
                                    };
                                    if alpha == 0 {
                                        continue;
                                    }
                                    let p = dst_y * stride + dst_x * 4;
                                    let blend = |bg: u8, fg: u8, a: u8| -> u8 {
                                        ((bg as u16 * (255 - a as u16) + fg as u16 * a as u16)
                                            / 255) as u8
                                    };
                                    unsafe {
                                        let eb = *bits.add(p);
                                        let eg = *bits.add(p + 1);
                                        let er = *bits.add(p + 2);
                                        *bits.add(p) = blend(eb, fg_r, alpha);
                                        *bits.add(p + 1) = blend(eg, fg_g, alpha);
                                        *bits.add(p + 2) = blend(er, fg_b, alpha);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        unsafe { ANativeWindow_unlockAndPost(window_ptr) };
        Ok(())
    }

    #[cfg(target_os = "android")]
    pub fn render_software(&mut self) -> Result<(), SurfaceError> {
        let snapshot = self
            .session
            .as_ref()
            .map(|s| s.terminal().take_snapshot())
            .ok_or(SurfaceError::NoSurface)?;
        self.render_software_with_snapshot(&snapshot)
    }

    #[cfg(not(target_os = "android"))]
    pub fn render_software(&mut self) -> Result<(), SurfaceError> {
        Err(SurfaceError::Render(
            "software rendering not available".to_string(),
        ))
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
        log::trace!(
            "SURFACE_RESIZE: rows={} cols={} has_session={}",
            rows,
            cols,
            self.session.is_some(),
        );
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

        // Recalculate grid dimensions to match new cell size
        if self.surface_width > 0 && self.surface_height > 0 {
            let new_cols = (self.surface_width as f32 / cw).floor().clamp(20.0, 300.0) as u32;
            let new_rows = (self.surface_height as f32 / ch).floor().clamp(5.0, 200.0) as u32;
            self.cols = new_cols;
            self.rows = new_rows;
            if let Some(session) = &mut self.session {
                let _ = session.resize(new_rows, new_cols);
            }
            log::info!(
                "set_font_size: font_size={} cells={:.1}x{:.1} grid={}x{}",
                size,
                cw,
                ch,
                new_rows,
                new_cols,
            );
        }
    }

    pub fn set_font_family(&mut self, family_name: &str) -> bool {
        if !self.font_pipeline.set_font_family(family_name) {
            return false;
        }
        self.font_pipeline.rasterize_ascii();
        let (aw, ah) = self.font_pipeline.atlas_dimensions();
        let (cw, ch) = self.font_pipeline.cell_metrics();
        self.gpu.update_bind_group(aw as f32, ah as f32, cw, ch);
        let atlas_data = self.font_pipeline.atlas_bitmap().to_vec();
        self.gpu.upload_atlas(&atlas_data, aw, ah);
        true
    }

    pub fn set_mouse_position(&mut self, row: u32, col: u32) {
        self.mouse_row = Some(row);
        self.mouse_col = Some(col);
    }

    pub fn get_hovered_url(&self) -> Option<String> {
        self.last_hovered_url.clone()
    }

    pub fn set_theme(&mut self, theme: torvox_core::config::Theme) {
        let (bg, fg) = (theme.bg, theme.fg);
        self.theme = theme;
        if let Some(session) = &self.session {
            session.terminal().set_theme(bg, fg);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_to_line_empty() {
        let cells = [];
        let line = cell_to_line(&cells, 5);
        assert_eq!(line.len(), 5);
    }

    #[test]
    fn cell_to_line_populated() {
        let cells = vec![
            CellSnapshot {
                codepoint: 0x41, // 'A'
                fg: [1.0, 0.0, 0.0, 1.0],
                bg: [0.0, 0.0, 1.0, 1.0],
                bold: true,
                italic: false,
                underline: false,
                reverse: false,
                uri: None,
            },
            CellSnapshot {
                codepoint: 0x42, // 'B'
                ..Default::default()
            },
        ];
        let line = cell_to_line(&cells, 2);
        assert_eq!(line.len(), 2);
        let c0 = line.get(0).unwrap();
        assert_eq!(c0.char, 'A');
        assert!(c0.attrs.bold);
        assert_eq!(c0.fg.r, 255);
        assert_eq!(c0.bg.b, 255);
    }

    #[test]
    fn cell_to_line_truncates_excess() {
        let cells = vec![
            CellSnapshot {
                codepoint: 0x41,
                ..Default::default()
            },
            CellSnapshot {
                codepoint: 0x42,
                ..Default::default()
            },
            CellSnapshot {
                codepoint: 0x43,
                ..Default::default()
            },
        ];
        let line = cell_to_line(&cells, 2);
        assert_eq!(line.len(), 2);
        assert_eq!(line.get(0).unwrap().char, 'A');
        assert_eq!(line.get(1).unwrap().char, 'B');
    }

    #[test]
    fn cell_to_line_pads_short_input() {
        let cells = [CellSnapshot {
            codepoint: 0x41,
            ..Default::default()
        }];
        let line = cell_to_line(&cells, 5);
        assert_eq!(line.len(), 5);
        assert_eq!(line.get(4).unwrap().char, ' ');
    }

    #[test]
    fn line_to_text_basic() {
        let cells = vec![
            CellSnapshot {
                codepoint: 0x48,
                ..Default::default()
            }, // H
            CellSnapshot {
                codepoint: 0x69,
                ..Default::default()
            }, // i
        ];
        let line = cell_to_line(&cells, 2);
        assert_eq!(line_to_text(&line), "Hi");
    }

    #[test]
    fn line_to_text_includes_null_chars() {
        let cells = vec![
            CellSnapshot {
                codepoint: 0x48,
                ..Default::default()
            },
            CellSnapshot {
                codepoint: 0x69,
                ..Default::default()
            },
            CellSnapshot {
                codepoint: 0x00,
                ..Default::default()
            },
        ];
        let line = cell_to_line(&cells, 3);
        assert_eq!(line_to_text(&line), "Hi\0");
    }

    #[test]
    fn line_to_text_all_spaces() {
        let line = cell_to_line(&[], 5);
        assert_eq!(line_to_text(&line), "     ");
    }

    #[test]
    fn line_to_text_preserves_colors() {
        let cell = CellSnapshot {
            codepoint: 0x41,
            fg: [0.5, 0.0, 0.0, 1.0],
            ..Default::default()
        };
        let line = cell_to_line(&[cell], 1);
        assert_eq!(line_to_text(&line), "A");
    }

    #[test]
    fn line_to_text_unicode() {
        let cells = vec![
            CellSnapshot {
                codepoint: 0x4E2D,
                ..Default::default()
            }, // 中
            CellSnapshot {
                codepoint: 0x6587,
                ..Default::default()
            }, // 文
        ];
        let line = cell_to_line(&cells, 2);
        assert_eq!(line_to_text(&line), "中文");
    }

    #[test]
    fn has_saved_session_nonexistent() {
        assert!(!AndroidSurface::has_saved_session("/nonexistent/path"));
    }

    #[test]
    fn cell_to_line_with_bold_italic_underline_reverse() {
        let cell = CellSnapshot {
            codepoint: 0x41,
            bold: true,
            italic: true,
            underline: true,
            reverse: true,
            ..Default::default()
        };
        let line = cell_to_line(&[cell], 1);
        let c = line.get(0).unwrap();
        assert!(c.attrs.bold);
        assert!(c.attrs.italic);
        assert!(c.attrs.underline);
        assert!(c.attrs.reverse);
    }
}
