use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use thiserror::Error;
use torvox_renderer::font::FontPipeline;
use torvox_renderer::gpu::GpuContext;
use torvox_terminal::ghostty_terminal::GhosttyTerminal;
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
        }
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

    pub fn set_theme(&mut self, theme: torvox_core::config::Theme) {
        self.theme = theme;
    }

    pub fn theme(&self) -> &torvox_core::config::Theme {
        &self.theme
    }
}

impl Drop for AndroidSurface {
    /// Ensures GPU resources (which hold a reference to the ANativeWindow) are
    /// dropped before the window itself is destroyed. Drop order in Rust is
    /// declaration order, so the explicit pre-drop of the session and gpu
    /// fields is not needed — they are dropped after `exited` and the other
    /// primitives — but this documents the intent and adds a defensive
    /// `take()` to make the ordering robust against future field reordering.
    fn drop(&mut self) {
        // Drop the session first: it owns the PTY + terminal thread, which
        // may submit commands to the GPU. We must let the terminal thread
        // exit before we tear down the GPU.
        self.session.take();
        // The GpuContext holds the wgpu Surface which references the
        // ANativeWindow. After the rest of the struct is dropped, the
        // GpuContext will be dropped last, which is the correct order.
        // We call a no-op render to flush any pending GPU work before
        // the surface itself is released.
        if self.gpu.has_surface() {
            self.gpu.warmup();
        }
    }
}
