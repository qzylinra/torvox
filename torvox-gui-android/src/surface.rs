use thiserror::Error;
use torvox_renderer::font::FontPipeline;
use torvox_renderer::gpu::GpuContext;
use torvox_terminal::ghostty_terminal::GhosttyTerminal;

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
}

pub struct AndroidSurface {
    gpu: GpuContext,
    font_pipeline: FontPipeline,
    terminal: GhosttyTerminal,
    atlas_width: u32,
    atlas_height: u32,
    theme: torvox_core::config::Theme,
}

impl AndroidSurface {
    pub fn new(rows: u32, cols: u32) -> Self {
        let atlas_width = 2048;
        let atlas_height = 2048;
        let mut font_pipeline = FontPipeline::new(atlas_width as i32, atlas_height as i32, 14.0);
        font_pipeline.rasterize_ascii();
        let terminal =
            GhosttyTerminal::new(rows, cols, 50000).expect("failed to create GhosttyTerminal");

        Self {
            gpu: GpuContext::new_with_no_surface(),
            font_pipeline,
            terminal,
            atlas_width,
            atlas_height,
            theme: torvox_core::config::Theme::catppuccin_mocha(),
        }
    }

    pub fn set_native_window(
        &mut self,
        window_ptr: *mut std::ffi::c_void,
    ) -> Result<(), SurfaceError> {
        self.gpu
            .set_surface_from_native_window(window_ptr)
            .map_err(|e| SurfaceError::SurfaceCreation(e.to_string()))?;

        self.gpu
            .create_atlas_texture(self.atlas_width, self.atlas_height);

        let (aw, ah) = self.font_pipeline.atlas_dimensions();
        self.gpu.update_bind_group(aw as f32, ah as f32);

        let atlas_data = self.font_pipeline.atlas_bitmap().to_vec();
        self.gpu.upload_atlas(&atlas_data, aw, ah);

        Ok(())
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let instances = torvox_renderer::gpu::build_cell_instances_from_ghostty(
            &self.terminal,
            &mut self.font_pipeline,
            8.0,
            16.0,
            self.atlas_width as f32,
            self.atlas_height as f32,
        );

        self.gpu
            .render_frame(&instances)
            .map_err(|e| SurfaceError::Render(e.to_string()))
    }

    pub fn terminal(&self) -> &GhosttyTerminal {
        &self.terminal
    }

    pub fn terminal_mut(&mut self) -> &mut GhosttyTerminal {
        &mut self.terminal
    }

    pub fn font_pipeline(&self) -> &FontPipeline {
        &self.font_pipeline
    }

    pub fn resize(&mut self, rows: u32, cols: u32) {
        self.terminal.resize(rows, cols, 8, 16);
    }

    pub fn write_to_pty(&mut self, data: &[u8]) {
        self.terminal.vt_write(data);
    }

    pub fn set_font_size(&mut self, size: f32) {
        self.font_pipeline =
            FontPipeline::new(self.atlas_width as i32, self.atlas_height as i32, size);
        self.font_pipeline.rasterize_ascii();
        let (aw, ah) = self.font_pipeline.atlas_dimensions();
        self.gpu.update_bind_group(aw as f32, ah as f32);
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
