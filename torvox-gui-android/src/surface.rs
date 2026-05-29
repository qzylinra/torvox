use thiserror::Error;
use torvox_renderer::font::FontPipeline;
use torvox_renderer::gpu::{self, FlatGrid, GpuContext};
use torvox_terminal::session::Session;
use torvox_terminal::terminal::TerminalState;

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
    #[error("Terminal init failed: {0}")]
    TerminalInit(String),
}

pub struct AndroidSurface {
    gpu: GpuContext,
    font_pipeline: FontPipeline,
    session: Option<Session>,
    terminal: TerminalState,
    flat_grid: FlatGrid,
    atlas_width: u32,
    atlas_height: u32,
}

// SAFETY: AndroidSurface is only accessed from the Android UI thread.
// Ghostty VT's Terminal is !Send + !Sync by design (single-threaded use),
// but our architecture guarantees all access happens on the UI thread.
unsafe impl Send for AndroidSurface {}
unsafe impl Sync for AndroidSurface {}

impl AndroidSurface {
    pub fn new(rows: u32, cols: u32) -> Result<Self, SurfaceError> {
        let atlas_width = 2048;
        let atlas_height = 2048;
        let mut font_pipeline = FontPipeline::new(atlas_width as i32, atlas_height as i32, 14.0);
        font_pipeline.rasterize_ascii();
        let terminal = TerminalState::new(rows, cols)
            .map_err(|e| SurfaceError::TerminalInit(e.to_string()))?;
        let flat_grid = FlatGrid::new(rows, cols);

        Ok(Self {
            gpu: GpuContext::new_with_no_surface(),
            font_pipeline,
            session: None,
            terminal,
            flat_grid,
            atlas_width,
            atlas_height,
        })
    }

    pub fn spawn_session(&mut self, shell: &str) -> Result<(), SurfaceError> {
        let session = Session::spawn(shell, self.terminal.rows(), self.terminal.cols())
            .map_err(|e| SurfaceError::GpuInit(e.to_string()))?;
        self.session = Some(session);
        Ok(())
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

    fn update_flat_grid_from_ghostty(&mut self) {
        use libghostty_vt::render::{CellIterator, RowIterator};

        let terminal = &mut self.terminal;
        let render_state_ptr: *mut libghostty_vt::RenderState<'static> =
            terminal.render_state_mut();
        let terminal_ptr: *const libghostty_vt::Terminal<'static, 'static> = terminal.terminal();

        // SAFETY: render_state and terminal are separate fields of TerminalState.
        // render_state.update() reads from terminal and writes to render_state.
        // No overlapping mutable borrows exist — this mirrors libghostty-vt's internal pattern.
        unsafe {
            let Ok(snapshot) = (*render_state_ptr).update(&*terminal_ptr) else {
                return;
            };
            let Ok(mut rows_iter) = RowIterator::new() else {
                return;
            };
            let Ok(mut cells_iter) = CellIterator::new() else {
                return;
            };
            let Ok(mut row_iter) = rows_iter.update(&snapshot) else {
                return;
            };

            let mut row_index = 0u32;
            while let Some(row) = row_iter.next() {
                let Ok(mut cell_iter) = cells_iter.update(row) else {
                    continue;
                };
                let mut col_index = 0u32;
                while let Some(cell) = cell_iter.next() {
                    let ch = cell
                        .graphemes()
                        .ok()
                        .and_then(|g| g.first().copied())
                        .unwrap_or(' ');

                    let fg = cell
                        .fg_color()
                        .ok()
                        .flatten()
                        .map(|c| {
                            [
                                c.r as f32 / 255.0,
                                c.g as f32 / 255.0,
                                c.b as f32 / 255.0,
                                1.0,
                            ]
                        })
                        .unwrap_or([1.0, 1.0, 1.0, 1.0]);

                    let bg = cell
                        .bg_color()
                        .ok()
                        .flatten()
                        .map(|c| {
                            [
                                c.r as f32 / 255.0,
                                c.g as f32 / 255.0,
                                c.b as f32 / 255.0,
                                1.0,
                            ]
                        })
                        .unwrap_or([0.0, 0.0, 0.0, 1.0]);

                    self.flat_grid.set_cell(row_index, col_index, ch, fg, bg);
                    col_index += 1;
                }
                row_index += 1;
            }
        }
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        if let Some(session) = &mut self.session {
            session.process_output();
        }

        self.update_flat_grid_from_ghostty();

        let instances = gpu::build_cell_instances_from_flat(
            &self.flat_grid,
            &mut self.font_pipeline,
            self.atlas_width as f32,
            self.atlas_height as f32,
        );

        self.gpu
            .render_frame(&instances)
            .map_err(|e| SurfaceError::Render(e.to_string()))
    }

    pub fn terminal(&self) -> &TerminalState {
        &self.terminal
    }

    pub fn terminal_mut(&mut self) -> &mut TerminalState {
        &mut self.terminal
    }

    pub fn resize(&mut self, rows: u32, cols: u32) {
        self.terminal.resize(rows, cols);
        self.flat_grid = FlatGrid::new(rows, cols);
    }

    pub fn write_to_pty(&mut self, data: &[u8]) {
        if let Some(session) = &mut self.session {
            let _ = session.write(data);
        }
    }
}
