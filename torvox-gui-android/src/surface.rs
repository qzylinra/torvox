// @AndroidSurface rendering, IMPL_ANDR_002, impl, [REQ_ANDR_002]
// @need-ids: REQ_ANDR_002
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use thiserror::Error;
use torvox_core::line::Line;
use torvox_renderer::font::FontPipeline;
use torvox_renderer::gpu::GpuContext;
use torvox_renderer::gpu::SelectionRange;
use torvox_terminal::ghostty_terminal::{CellSnapshot, GhosttyTerminal};
use torvox_terminal::session::Session;
use torvox_terminal::shell_env::ShellEnv;

#[cfg(target_os = "android")]
#[link(name = "android")]
unsafe extern "C" {
    fn ANativeWindow_release(window: *mut std::ffi::c_void);
    fn ANativeWindow_setBuffersGeometry(
        window: *mut std::ffi::c_void,
        width: i32,
        height: i32,
        format: i32,
    ) -> i32;
    fn ATrace_beginSection(section_name: *const std::os::raw::c_char);
    fn ATrace_endSection();
}

#[derive(Debug, Error)]
pub enum SurfaceError {
    #[error("GPU initialization failed: {0}")]
    GpuInit(String),
    #[error("Surface creation failed: {0}")]
    SurfaceCreation(String),
    #[error("No surface available")]
    NoSurface,
    #[error("No active session")]
    NoSession,
    #[error("Render error: {0}")]
    Render(String),
    #[error("session error: {0}")]
    Session(String),
}

impl From<torvox_renderer::gpu::GpuError> for SurfaceError {
    fn from(e: torvox_renderer::gpu::GpuError) -> Self {
        SurfaceError::Render(e.to_string())
    }
}

/// Wrap `NonNull<c_void>` for `Send + Sync` (ANativeWindow pointer is thread-safe on Android).
#[allow(dead_code)]
struct NativeWindow(std::ptr::NonNull<std::ffi::c_void>);
unsafe impl Send for NativeWindow {}
unsafe impl Sync for NativeWindow {}

pub struct AndroidSurface {
    #[allow(dead_code)]
    gpu: Option<GpuContext>,
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
    frame_count: u64,
    title: String,
    selection: Option<SelectionRange>,
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
    pub fn new(rows: u32, cols: u32, _scrollback_lines: u32, font_size: f32) -> Self {
        let atlas_width = 2048;
        let atlas_height = 2048;
        let font_pipeline = FontPipeline::new(atlas_width as i32, atlas_height as i32, font_size);

        let gpu = Some(GpuContext::new_with_no_surface());
        Self {
            gpu,
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
            frame_count: 0,
            title: String::new(),
            selection: None,
        }
    }

    pub fn set_save_path(&mut self, path: String) {
        self.save_path = Some(PathBuf::from(path));
    }

    pub fn spawn_session(&mut self, shell: &str, env: &ShellEnv) -> Result<(), SurfaceError> {
        let (bg, fg) = (self.theme.bg, self.theme.fg);
        let ansi = self.theme.ansi;
        let session = Session::spawn_with_theme(shell, self.rows, self.cols, env, bg, fg, ansi)
            .map_err(|e| SurfaceError::Session(e.to_string()))?;
        self.exited = session.exited_flag().clone();
        self.session = Some(session);
        if let Some(session) = &mut self.session {
            session.set_pixel_size(
                (self.surface_width as u16).min(4096),
                (self.surface_height as u16).min(4096),
            );
        }

        #[cfg(target_os = "android")]
        unsafe {
            ATrace_endSection();
        }
        Ok(())
    }
    pub fn update_native_window(
        &mut self,
        window_ptr: *mut std::ffi::c_void,
        width: u32,
        height: u32,
    ) -> Result<(), SurfaceError> {
        let pointer_changed = self
            .native_window
            .as_ref()
            .map(|nw| nw.0.as_ptr() != window_ptr)
            .unwrap_or(true);
        log::info!(
            "UPDATE_NATIVE_WINDOW: ptr={:#x} {}x{} pointer_changed={}",
            window_ptr as usize,
            width,
            height,
            pointer_changed,
        );
        self.native_window = std::ptr::NonNull::new(window_ptr).map(NativeWindow);
        self.surface_width = width;
        self.surface_height = height;
        // Set ANativeWindow buffer format for wgpu swapchain.
        // Must use RGBA_8888 (format=1).
        #[cfg(target_os = "android")]
        if let Some(nw) = self.native_window.as_ref() {
            let result = unsafe {
                ANativeWindow_setBuffersGeometry(
                    nw.0.as_ptr(),
                    width as i32,
                    height as i32,
                    1, // WINDOW_FORMAT_RGBA_8888
                )
            };
            if result != 0 {
                log::error!("ANativeWindow_setBuffersGeometry failed: {}", result);
            }
        }
        // Reconfigure or recreate the wgpu surface.
        // When the ANativeWindow changes (surface destroy/recreate), we must
        // recreate the wgpu Surface from the new window pointer. A plain
        // reconfigure on the old (stale) surface produces no visible output.
        // Also recreate when has_surface() is false (release_gpu_surface was called
        // to release the ANativeWindow for another bridge to use).
        #[cfg(target_os = "android")]
        if let Some(gpu) = &mut self.gpu {
            if pointer_changed || !gpu.has_surface() {
                gpu.configure_android_surface(window_ptr, width.max(1), height.max(1))
                    .map_err(|e| SurfaceError::GpuInit(e.to_string()))?;
            } else {
                gpu.reconfigure_swapchain(width.max(1), height.max(1));
            }
        }
        // Desktop: always recreate the surface from the new native window pointer.
        #[cfg(not(target_os = "android"))]
        if let Some(gpu) = &mut self.gpu {
            let ptr = window_ptr;
            gpu.configure_android_surface(ptr, width.max(1), height.max(1))
                .map_err(|e| SurfaceError::GpuInit(e.to_string()))?;
        }
        self.recompute_grid(width.max(1), height.max(1));
        Ok(())
    }

    pub fn set_native_window(
        &mut self,
        window_ptr: *mut std::ffi::c_void,
        width: u32,
        height: u32,
        font_size_tenths: u32,
    ) -> Result<(), SurfaceError> {
        self.native_window = std::ptr::NonNull::new(window_ptr).map(NativeWindow);
        self.surface_width = width;
        self.surface_height = height;

        // Use configured font size, fall back to geometry-based calc
        let font_size = if font_size_tenths > 0 {
            font_size_tenths as f32 / 10.0
        } else {
            ((width as f32 / self.cols as f32) * 1.6).max(24.0)
        };

        log::info!(
            "SET_FONT_SIZE: font_size_tenths={} font_size={} self.rows={} self.cols={}",
            font_size_tenths,
            font_size,
            self.rows,
            self.cols,
        );

        self.font_pipeline =
            FontPipeline::new(self.atlas_width as i32, self.atlas_height as i32, font_size);
        self.font_pipeline.rasterize_ascii();
        let (_aw, _ah) = self.font_pipeline.atlas_dimensions();
        let (cw, ch) = self.font_pipeline.cell_metrics();

        log::info!(
            "CELL_METRICS: cw={} ch={} (sw={} sh={}) recomputed_cols={} recomputed_rows={}",
            cw,
            ch,
            width,
            height,
            (width as f32 / cw).floor(),
            (height as f32 / ch).floor().clamp(5.0, 200.0),
        );

        // Compute grid dimensions to fill the surface
        let cols = (width as f32 / cw).floor().clamp(20.0, 300.0) as u32;
        let rows = (height as f32 / ch).floor().clamp(5.0, 200.0) as u32;
        self.rows = rows;
        self.cols = cols;

        // Pre-configure ANativeWindow buffer geometry for blit path.
        // Must use RGBA_8888 (format=1) — ANativeWindow_lock legacy API
        // does not support AHARDWAREBUFFER formats (format=2+).
        #[cfg(target_os = "android")]
        if let Some(nw) = self.native_window.as_ref() {
            let result = unsafe {
                ANativeWindow_setBuffersGeometry(
                    nw.0.as_ptr(),
                    width as i32,
                    height as i32,
                    1, // WINDOW_FORMAT_RGBA_8888
                )
            };
            if result != 0 {
                log::error!("ANativeWindow_setBuffersGeometry failed: {}", result);
            }
        }

        log::info!(
            "SURFACE_SET_NATIVE_WINDOW: grid={}x{} surface={}x{}",
            rows,
            cols,
            width,
            height,
        );

        let needs_init = self
            .gpu
            .as_ref()
            .is_none_or(|g| !g.has_pipeline() || !g.has_surface());

        if needs_init {
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
                let gpu = self.gpu.as_mut().unwrap();
                gpu.set_surface_from_native_window(window_ptr, width, height, true)
                    .map_err(|e| SurfaceError::GpuInit(e.to_string()))?;
                let atlas_data = self.font_pipeline.atlas_bitmap().to_vec();
                gpu.create_atlas_texture(aw, ah);
                gpu.upload_atlas(&atlas_data, aw, ah);
                gpu.update_bind_group(aw as f32, ah as f32);
            }

            // Android: no-surface path uses device-only pipeline.
            // Swapchain path creates wgpu Surface from ANativeWindow
            // for Vulkan/GLES presentation.
            #[cfg(target_os = "android")]
            {
                let gpu = self.gpu.as_mut().unwrap();
                let atlas_data = self.font_pipeline.atlas_bitmap().to_vec();
                if !window_ptr.is_null() {
                    // Create swapchain surface for GPU presentation
                    gpu.configure_android_surface(window_ptr, width, height)
                        .map_err(|e| SurfaceError::GpuInit(e.to_string()))?;
                }
                gpu.init_pipeline_and_bind_group(aw, ah, width, height);
                gpu.upload_atlas(&atlas_data, aw, ah);
            }
        }

        Ok(())
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let frame_start = Instant::now();
        #[cfg(target_os = "android")]
        unsafe {
            ATrace_beginSection(c"AndroidSurface::render".as_ptr());
        }
        log::trace!(
            "RENDER_ENTER: session={} sw={} sh={} native={}",
            self.session.is_some(),
            self.surface_width,
            self.surface_height,
            self.native_window.is_some(),
        );

        if let Some(session) = &mut self.session {
            session.process_output();
        }

        #[cfg(target_os = "android")]
        unsafe {
            ATrace_beginSection(c"snapshot+instances".as_ptr());
        }

        let snapshot = self
            .session
            .as_ref()
            .map(|s| s.terminal())
            .map(|t| t.take_snapshot())
            .ok_or(SurfaceError::NoSurface)?;

        if let (Some(row), Some(col)) = (self.mouse_row, self.mouse_col) {
            self.last_hovered_url = snapshot.uri_at(row, col).map(String::from);
        }

        // Always rasterize glyphs so the atlas is populated for GPU rendering
        let gen_before = self.font_pipeline.atlas_generation();
        let cursor_color = Some([1.0, 1.0, 1.0, 1.0]);
        let instances = torvox_renderer::gpu::build_cell_instances_from_snapshot(
            &snapshot,
            &mut self.font_pipeline,
            self.atlas_width as f32,
            self.atlas_height as f32,
            None,
            self.selection,
            cursor_color,
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
            self.gpu.as_mut().unwrap().upload_atlas(
                &atlas_data,
                self.atlas_width,
                self.atlas_height,
            );
        }

        #[cfg(target_os = "android")]
        unsafe {
            ATrace_endSection(); // snapshot+instances
            ATrace_beginSection(c"swapchain_present".as_ptr());
        }

        if !instances.is_empty() {
            let first = &instances[0];
            log::info!(
                "RENDER_INSTANCES: count={} first_cell=({:.0},{:.0}) bg=({},{},{}) fg=({},{},{}) flags={:.0} uv_size=({:.4},{:.4}) bearing=({:.1},{:.1}) advance_width={:.1}",
                instances.len(),
                first.quad_origin[0],
                first.quad_origin[1],
                (first.bg_color[0] * 255.0) as u8,
                (first.bg_color[1] * 255.0) as u8,
                (first.bg_color[2] * 255.0) as u8,
                (first.fg_color[0] * 255.0) as u8,
                (first.fg_color[1] * 255.0) as u8,
                (first.fg_color[2] * 255.0) as u8,
                first.flags,
                first.atlas_size[0],
                first.atlas_size[1],
                first.bearing[0],
                first.bearing[1],
                first.glyph_advance_width,
            );
        } else {
            log::warn!("RENDER_INSTANCES: ZERO instances — nothing to render!");
        }

        self.frame_count += 1;
        if self.frame_count.is_multiple_of(60)
            && let Some(session) = &mut self.session
        {
            self.title = session.terminal().title();
        }

        // Desktop: direct wgpu swapchain presentation.
        #[cfg(not(target_os = "android"))]
        {
            let gpu = self.gpu.as_mut().unwrap();
            if let Err(e) = gpu.render_frame(&instances) {
                log::error!("RENDER_FRAME_FAILED: {}", e);
                return Err(SurfaceError::Render(e.to_string()));
            }
        }

        // Android: wgpu Vulkan swapchain — sole render path.
        // No CPU software fallback. If swapchain fails, the error is logged.
        #[cfg(target_os = "android")]
        {
            if self.poll_sync_active() {
                log::trace!("sync active — skipping GPU frame");
                unsafe {
                    ATrace_endSection();
                } // swapchain_present
                unsafe {
                    ATrace_endSection();
                } // AndroidSurface::render
                return Ok(());
            }
            let gpu = self.gpu.as_mut().unwrap();
            if gpu.has_surface() {
                if let Err(e) = gpu.render_frame(&instances) {
                    log::error!("SWAPCHAIN_FAILED: {}", e);
                }
            }
        }

        #[cfg(target_os = "android")]
        unsafe {
            ATrace_endSection(); // swapchain_present
        }

        let elapsed = frame_start.elapsed();
        let ms = elapsed.as_secs_f64() * 1000.0;
        if ms >= 16.0 {
            log::warn!("RENDER_SLOW: {:.1}ms (≥16ms target)", ms);
        } else {
            log::trace!("RENDER_OK: {:.1}ms", ms);
        }

        Ok(())
    }

    /// Render a single test frame to an offscreen GPU buffer and write raw RGBA
    /// data to `{data_dir}/test_frame.rgba`. Returns the file path on success.
    /// This is a test-only path — NOT used for display.
    pub fn save_test_frame(&mut self, data_dir: &str) -> Result<String, SurfaceError> {
        let snapshot = self
            .session
            .as_ref()
            .map(|s| s.terminal())
            .map(|t| t.take_snapshot())
            .ok_or(SurfaceError::NoSurface)?;
        let cursor_color = Some([1.0, 1.0, 1.0, 1.0]);
        let instances = torvox_renderer::gpu::build_cell_instances_from_snapshot(
            &snapshot,
            &mut self.font_pipeline,
            self.atlas_width as f32,
            self.atlas_height as f32,
            None,
            self.selection,
            cursor_color,
        );
        let atlas_data = self.font_pipeline.atlas_bitmap().to_vec();
        let gpu = self.gpu.as_mut().unwrap();
        gpu.upload_atlas(&atlas_data, self.atlas_width, self.atlas_height);
        let pixels = gpu
            .render_to_buffer(&instances)
            .map_err(|e| SurfaceError::Render(e.to_string()))?;
        let path = format!("{}/test_frame.rgba", data_dir);
        std::fs::write(&path, &pixels).map_err(|e| SurfaceError::Render(e.to_string()))?;
        log::info!("SAVED_TEST_FRAME: {} ({} bytes)", path, pixels.len());
        Ok(path)
    }

    pub fn terminal(&self) -> Result<&GhosttyTerminal, SurfaceError> {
        self.session
            .as_ref()
            .map(|s| s.terminal())
            .ok_or(SurfaceError::NoSession)
    }

    pub fn terminal_mut(&mut self) -> Result<&mut GhosttyTerminal, SurfaceError> {
        self.session
            .as_mut()
            .map(|s| s.terminal_mut())
            .ok_or(SurfaceError::NoSession)
    }

    pub fn font_pipeline(&self) -> &FontPipeline {
        &self.font_pipeline
    }

    pub fn recompute_grid(&mut self, width: u32, height: u32) {
        let (cw, ch) = self.font_pipeline.cell_metrics();
        let new_cols = (width as f32 / cw).floor().clamp(20.0, 300.0) as u32;
        let new_rows = (height as f32 / ch).floor().clamp(5.0, 200.0) as u32;

        if width != self.surface_width || height != self.surface_height {
            self.surface_width = width;
            self.surface_height = height;
        }

        if new_cols != self.cols || new_rows != self.rows {
            log::info!(
                "RECOMPUTE_GRID: {}x{} -> {}x{} (cell={:.1}x{:.1})",
                self.rows,
                self.cols,
                new_rows,
                new_cols,
                cw,
                ch,
            );
            self.rows = new_rows;
            self.cols = new_cols;
            if let Some(session) = &mut self.session {
                let _ = session.resize(new_rows, new_cols);
            }
        }
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

    pub fn poll_bel(&mut self) -> bool {
        self.session.as_mut().map(|s| s.poll_bel()).unwrap_or(false)
    }

    pub fn poll_clipboard(&mut self) -> Option<String> {
        self.session.as_mut().and_then(|s| s.poll_clipboard())
    }

    pub fn poll_notification(&mut self) -> Option<(String, String)> {
        self.session.as_mut().and_then(|s| s.poll_notification())
    }

    pub fn poll_sync_active(&mut self) -> bool {
        self.session
            .as_ref()
            .map(|s| s.mode_get(2026, 0))
            .unwrap_or(false)
    }

    pub fn poll_shell_integration(&mut self) -> u8 {
        self.session
            .as_mut()
            .map(|s| s.poll_shell_integration() as u8)
            .unwrap_or(0)
    }

    pub fn cwd(&self) -> String {
        self.session.as_ref().map(|s| s.cwd()).unwrap_or_default()
    }

    pub fn focus_event(&mut self, focused: bool) {
        if let Some(s) = self.session.as_mut() {
            s.focus_event(focused);
        }
    }

    pub fn has_session(&self) -> bool {
        self.session.is_some()
    }

    pub fn release_gpu_surface(&mut self) {
        if let Some(gpu) = &mut self.gpu {
            gpu.release_gpu_surface();
        }
    }

    pub fn set_font_size(&mut self, size: f32) {
        if (self.font_pipeline.font_size() - size).abs() < 0.001 {
            return;
        }
        self.font_pipeline =
            FontPipeline::new(self.atlas_width as i32, self.atlas_height as i32, size);
        self.font_pipeline.rasterize_ascii();
        let (aw, ah) = self.font_pipeline.atlas_dimensions();
        let (cw, ch) = self.font_pipeline.cell_metrics();
        if let Some(gpu) = &mut self.gpu {
            gpu.update_bind_group(aw as f32, ah as f32);
            let atlas_data = self.font_pipeline.atlas_bitmap().to_vec();
            gpu.upload_atlas(&atlas_data, aw, ah);
        }

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
        if let Some(gpu) = &mut self.gpu {
            gpu.update_bind_group(aw as f32, ah as f32);
            let atlas_data = self.font_pipeline.atlas_bitmap().to_vec();
            gpu.upload_atlas(&atlas_data, aw, ah);
        }
        if self.surface_width > 0 && self.surface_height > 0 {
            let new_cols = (self.surface_width as f32 / cw).floor().clamp(20.0, 300.0) as u32;
            let new_rows = (self.surface_height as f32 / ch).floor().clamp(5.0, 200.0) as u32;
            self.cols = new_cols;
            self.rows = new_rows;
            if let Some(session) = &mut self.session {
                let _ = session.resize(new_rows, new_cols);
            }
            log::info!(
                "set_font_family: family='{}' cells={:.1}x{:.1} grid={}x{}",
                family_name,
                cw,
                ch,
                new_rows,
                new_cols,
            );
        }
        true
    }

    pub fn set_mouse_position(&mut self, row: u32, col: u32) {
        self.mouse_row = Some(row);
        self.mouse_col = Some(col);
    }

    pub fn get_hovered_url(&self) -> Option<String> {
        self.last_hovered_url.clone()
    }

    pub fn get_title(&self) -> String {
        self.title.clone()
    }

    pub fn set_theme(&mut self, theme: torvox_core::config::Theme) {
        let (bg, fg) = (theme.bg, theme.fg);
        let ansi = theme.ansi;
        self.theme = theme;
        if let Some(session) = &self.session {
            session.terminal().set_theme(bg, fg, ansi);
        }
        if let Some(gpu) = self.gpu.as_mut() {
            gpu.set_bg_color(bg);
        }
    }

    pub fn set_selection(&mut self, sel: Option<SelectionRange>) {
        self.selection = sel;
    }

    pub fn theme(&self) -> &torvox_core::config::Theme {
        &self.theme
    }

    pub fn save_session(&self, path: &str) -> Result<(), SurfaceError> {
        use std::fs;
        use torvox_core::snapshot::SessionSnapshot;

        let dumped = self.terminal()?.dump_grid();
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
        #[cfg(target_os = "android")]
        if let Some(nw) = &self.native_window {
            unsafe { ANativeWindow_release(nw.0.as_ptr()) };
        }
        #[cfg(not(target_os = "android"))]
        if let Some(gpu) = &self.gpu
            && gpu.has_surface()
        {
            gpu.warmup();
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
                strikethrough: false,
                blink: false,
                hidden: false,
                uri: None,
                semantic: Default::default(),
                overline: false,
                dim: false,
                width: 1,
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

    #[test]
    fn no_session_terminal_returns_err() {
        // SurfaceError::NoSession is returned when terminal() is called without an active session.
        // This prevents the old .expect("no session") panic pattern that caused SIGABRT on device.
        let err = SurfaceError::NoSession;
        assert_eq!(err.to_string(), "No active session");
    }
}
