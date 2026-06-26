// @BoltFFI data bridge, IMPL_ANDR_001, impl, [REQ_ANDR_001]
// @need-ids: REQ_ANDR_001, REQ_ANDR_002, REQ_ANDR_003
#[derive(Debug, Clone, PartialEq, Eq)]
#[boltffi::data]
pub struct BridgeCell {
    pub char_code: u32,
    pub fg: u32,
    pub bg: u32,
    pub attrs: BridgeAttrs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[boltffi::data]
pub struct BridgeAttrs {
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub double_underline: bool,
    pub reverse: bool,
    pub strikethrough: bool,
    pub blink: bool,
    pub hidden: bool,
    pub overline: bool,
    pub protected: bool,
    pub double_width: bool,
    pub double_height_top: bool,
    pub double_height_bottom: bool,
}

impl From<torvox_core::cell::Attrs> for BridgeAttrs {
    fn from(a: torvox_core::cell::Attrs) -> Self {
        Self {
            bold: a.bold,
            dim: a.dim,
            italic: a.italic,
            underline: a.underline,
            double_underline: a.double_underline,
            reverse: a.reverse,
            strikethrough: a.strikethrough,
            blink: a.blink,
            hidden: a.hidden,
            overline: a.overline,
            protected: a.protected,
            double_width: a.double_width,
            double_height_top: a.double_height_top,
            double_height_bottom: a.double_height_bottom,
        }
    }
}

impl From<BridgeAttrs> for torvox_core::cell::Attrs {
    fn from(a: BridgeAttrs) -> Self {
        Self {
            bold: a.bold,
            dim: a.dim,
            italic: a.italic,
            underline: a.underline,
            double_underline: a.double_underline,
            reverse: a.reverse,
            strikethrough: a.strikethrough,
            blink: a.blink,
            hidden: a.hidden,
            overline: a.overline,
            protected: a.protected,
            double_width: a.double_width,
            double_height_top: a.double_height_top,
            double_height_bottom: a.double_height_bottom,
        }
    }
}

impl From<torvox_core::cell::Cell> for BridgeCell {
    fn from(cell: torvox_core::cell::Cell) -> Self {
        Self {
            char_code: cell.char as u32,
            fg: ((cell.fg.r as u32) << 24)
                | ((cell.fg.g as u32) << 16)
                | ((cell.fg.b as u32) << 8)
                | (cell.fg.a as u32),
            bg: ((cell.bg.r as u32) << 24)
                | ((cell.bg.g as u32) << 16)
                | ((cell.bg.b as u32) << 8)
                | (cell.bg.a as u32),
            attrs: cell.attrs.into(),
        }
    }
}

impl From<BridgeCell> for torvox_core::cell::Cell {
    fn from(bridge_cell: BridgeCell) -> Self {
        Self {
            char: char::from_u32(bridge_cell.char_code).unwrap_or(' '),
            fg: torvox_core::cell::Color {
                r: ((bridge_cell.fg >> 24) & 0xFF) as u8,
                g: ((bridge_cell.fg >> 16) & 0xFF) as u8,
                b: ((bridge_cell.fg >> 8) & 0xFF) as u8,
                a: (bridge_cell.fg & 0xFF) as u8,
            },
            bg: torvox_core::cell::Color {
                r: ((bridge_cell.bg >> 24) & 0xFF) as u8,
                g: ((bridge_cell.bg >> 16) & 0xFF) as u8,
                b: ((bridge_cell.bg >> 8) & 0xFF) as u8,
                a: (bridge_cell.bg & 0xFF) as u8,
            },
            attrs: bridge_cell.attrs.into(),
            width: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[boltffi::data]
pub enum Shell {
    #[default]
    SystemDefault,
    Custom {
        path: String,
    },
}

impl From<Shell> for torvox_core::config::Shell {
    fn from(shell: Shell) -> Self {
        match shell {
            Shell::SystemDefault => torvox_core::config::Shell::SystemDefault,
            Shell::Custom { path } => torvox_core::config::Shell::Custom(path),
        }
    }
}

impl From<torvox_core::config::Shell> for Shell {
    fn from(core_shell: torvox_core::config::Shell) -> Self {
        match core_shell {
            torvox_core::config::Shell::SystemDefault => Shell::SystemDefault,
            torvox_core::config::Shell::Custom(path) => Shell::Custom { path },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[boltffi::data]
pub struct BridgeTheme {
    pub name: String,
    pub bg: u32,
    pub fg: u32,
    pub cursor: u32,
    pub selection_bg: u32,
    pub ansi0: u32,
    pub ansi1: u32,
    pub ansi2: u32,
    pub ansi3: u32,
    pub ansi4: u32,
    pub ansi5: u32,
    pub ansi6: u32,
    pub ansi7: u32,
    pub ansi8: u32,
    pub ansi9: u32,
    pub ansi10: u32,
    pub ansi11: u32,
    pub ansi12: u32,
    pub ansi13: u32,
    pub ansi14: u32,
    pub ansi15: u32,
}

fn rgb_to_u32(c: [u8; 3]) -> u32 {
    ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | (c[2] as u32)
}

fn u32_to_rgb(v: u32) -> [u8; 3] {
    [
        ((v >> 16) & 0xFF) as u8,
        ((v >> 8) & 0xFF) as u8,
        (v & 0xFF) as u8,
    ]
}

impl From<torvox_core::config::Theme> for BridgeTheme {
    fn from(theme: torvox_core::config::Theme) -> Self {
        Self {
            name: theme.name,
            bg: rgb_to_u32(theme.bg),
            fg: rgb_to_u32(theme.fg),
            cursor: rgb_to_u32(theme.cursor),
            selection_bg: rgb_to_u32(theme.bg),
            ansi0: rgb_to_u32(theme.ansi[0]),
            ansi1: rgb_to_u32(theme.ansi[1]),
            ansi2: rgb_to_u32(theme.ansi[2]),
            ansi3: rgb_to_u32(theme.ansi[3]),
            ansi4: rgb_to_u32(theme.ansi[4]),
            ansi5: rgb_to_u32(theme.ansi[5]),
            ansi6: rgb_to_u32(theme.ansi[6]),
            ansi7: rgb_to_u32(theme.ansi[7]),
            ansi8: rgb_to_u32(theme.ansi[8]),
            ansi9: rgb_to_u32(theme.ansi[9]),
            ansi10: rgb_to_u32(theme.ansi[10]),
            ansi11: rgb_to_u32(theme.ansi[11]),
            ansi12: rgb_to_u32(theme.ansi[12]),
            ansi13: rgb_to_u32(theme.ansi[13]),
            ansi14: rgb_to_u32(theme.ansi[14]),
            ansi15: rgb_to_u32(theme.ansi[15]),
        }
    }
}

impl From<BridgeTheme> for torvox_core::config::Theme {
    fn from(bridge_theme: BridgeTheme) -> Self {
        Self {
            name: bridge_theme.name,
            bg: u32_to_rgb(bridge_theme.bg),
            fg: u32_to_rgb(bridge_theme.fg),
            cursor: u32_to_rgb(bridge_theme.cursor),
            ansi: [
                u32_to_rgb(bridge_theme.ansi0),
                u32_to_rgb(bridge_theme.ansi1),
                u32_to_rgb(bridge_theme.ansi2),
                u32_to_rgb(bridge_theme.ansi3),
                u32_to_rgb(bridge_theme.ansi4),
                u32_to_rgb(bridge_theme.ansi5),
                u32_to_rgb(bridge_theme.ansi6),
                u32_to_rgb(bridge_theme.ansi7),
                u32_to_rgb(bridge_theme.ansi8),
                u32_to_rgb(bridge_theme.ansi9),
                u32_to_rgb(bridge_theme.ansi10),
                u32_to_rgb(bridge_theme.ansi11),
                u32_to_rgb(bridge_theme.ansi12),
                u32_to_rgb(bridge_theme.ansi13),
                u32_to_rgb(bridge_theme.ansi14),
                u32_to_rgb(bridge_theme.ansi15),
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[boltffi::data]
pub struct TerminalConfig {
    pub shell: Shell,
    pub rows: u32,
    pub cols: u32,
    pub scrollback_lines: u32,
    pub font_size_tenths: u32,
    pub theme: BridgeTheme,
    pub home: String,
    pub user: String,
    pub path: String,
    pub working_directory: String,
    pub prefix: String,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            shell: Shell::SystemDefault,
            rows: 24,
            cols: 80,
            scrollback_lines: 50_000,
            font_size_tenths: 140,
            theme: torvox_core::config::Theme::catppuccin_mocha().into(),
            home: String::new(),
            user: String::new(),
            path: String::new(),
            working_directory: String::new(),
            prefix: String::new(),
        }
    }
}

impl From<TerminalConfig> for torvox_core::config::TerminalConfig {
    fn from(bridge_config: TerminalConfig) -> Self {
        Self {
            shell: bridge_config.shell.into(),
            rows: bridge_config.rows,
            cols: bridge_config.cols,
            scrollback_lines: bridge_config.scrollback_lines,
            font_size_tenths: bridge_config.font_size_tenths,
            backspace_mode: torvox_core::config::BackspaceMode::default(),
            right_alt_mode: torvox_core::config::RightAltMode::default(),
        }
    }
}

impl From<torvox_core::config::TerminalConfig> for TerminalConfig {
    fn from(core_config: torvox_core::config::TerminalConfig) -> Self {
        Self {
            shell: core_config.shell.into(),
            rows: core_config.rows,
            cols: core_config.cols,
            scrollback_lines: core_config.scrollback_lines,
            font_size_tenths: core_config.font_size_tenths,
            theme: torvox_core::config::Theme::catppuccin_mocha().into(),
            home: String::new(),
            user: String::new(),
            path: String::new(),
            working_directory: String::new(),
            prefix: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[boltffi::data]
pub enum TerminalEvent {
    Bell,
    TitleChanged {
        title: String,
    },
    ClipboardRequest {
        text: String,
    },
    HyperlinkHover {
        url: Option<String>,
    },
    ProcessExited {
        exit_code: i32,
    },
    DirtyRegion {
        start_row: u32,
        end_row: u32,
    },
    CursorChanged {
        row: u32,
        col: u32,
    },
    SelectionChanged {
        start_row: u32,
        start_col: u32,
        end_row: u32,
        end_col: u32,
    },
}

impl From<torvox_core::event::TerminalEvent> for TerminalEvent {
    fn from(event: torvox_core::event::TerminalEvent) -> Self {
        match event {
            torvox_core::event::TerminalEvent::OutputReady => TerminalEvent::DirtyRegion {
                start_row: 0,
                end_row: 0,
            },
            torvox_core::event::TerminalEvent::Bell => TerminalEvent::Bell,
            torvox_core::event::TerminalEvent::TitleChanged(title) => {
                TerminalEvent::TitleChanged { title }
            }
            torvox_core::event::TerminalEvent::ClipboardRequest(text) => {
                TerminalEvent::ClipboardRequest { text }
            }
            torvox_core::event::TerminalEvent::HyperlinkHover(url) => {
                TerminalEvent::HyperlinkHover { url }
            }
            torvox_core::event::TerminalEvent::ProcessExited(exit_code) => {
                TerminalEvent::ProcessExited { exit_code }
            }
            torvox_core::event::TerminalEvent::CursorChanged(cursor) => {
                TerminalEvent::CursorChanged {
                    row: cursor.row,
                    col: cursor.col,
                }
            }
            torvox_core::event::TerminalEvent::SelectionChanged(sel) => match sel {
                Some(selection) => {
                    let (start, end) = selection.ordered();
                    TerminalEvent::SelectionChanged {
                        start_row: start.row,
                        start_col: start.col,
                        end_row: end.row,
                        end_col: end.col,
                    }
                }
                None => TerminalEvent::SelectionChanged {
                    start_row: 0,
                    start_col: 0,
                    end_row: 0,
                    end_col: 0,
                },
            },
            torvox_core::event::TerminalEvent::DirtyRegion(dirty) => TerminalEvent::DirtyRegion {
                start_row: dirty.start_row,
                end_row: dirty.end_row,
            },
        }
    }
}

#[boltffi::error]
#[derive(Debug, Clone, thiserror::Error)]
pub enum TerminalError {
    #[error("PTY error: {detail}")]
    PtyError { detail: String },
    #[error("invalid config: {detail}")]
    InvalidConfig { detail: String },
}

pub struct TorvoxBridge {
    config: TerminalConfig,
    surface: std::sync::Mutex<Option<crate::surface::AndroidSurface>>,
}

impl TorvoxBridge {
    fn shell_env(&self) -> torvox_terminal::shell_env::ShellEnv {
        if self.config.home.is_empty() {
            torvox_terminal::shell_env::ShellEnv::default()
        } else {
            let working_directory = if self.config.working_directory.is_empty() {
                self.config.home.clone()
            } else {
                self.config.working_directory.clone()
            };
            let android_vars = [
                "ANDROID_ASSETS",
                "ANDROID_DATA",
                "ANDROID_ROOT",
                "ANDROID_STORAGE",
                "ANDROID_RUNTIME_ROOT",
                "ANDROID_ART_ROOT",
                "ANDROID_I18N_ROOT",
                "ANDROID_TZDATA_ROOT",
                "BOOTCLASSPATH",
                "EXTERNAL_STORAGE",
            ];
            let extra: Vec<(String, String)> = android_vars
                .into_iter()
                .filter_map(|key| std::env::var(key).ok().map(|val| (key.to_string(), val)))
                .collect();
            let prefix = if self.config.prefix.is_empty() {
                None
            } else {
                Some(self.config.prefix.clone())
            };
            torvox_terminal::shell_env::ShellEnv {
                home: self.config.home.clone(),
                user: self.config.user.clone(),
                path: self.config.path.clone(),
                working_directory,
                prefix,
                extra,
            }
        }
    }
}

fn with_bridge<F, T>(handle: i64, f: F) -> Result<T, TerminalError>
where
    F: FnOnce(&TorvoxBridge) -> Result<T, TerminalError>,
{
    let ptr = handle as *const TorvoxBridge;
    if ptr.is_null() {
        return Err(TerminalError::InvalidConfig {
            detail: "null bridge handle".to_string(),
        });
    }
    let bridge = unsafe { &*ptr };
    f(bridge)
}

#[boltffi::export]
impl TorvoxBridge {
    pub fn new(config: TerminalConfig) -> Self {
        #[cfg(target_os = "android")]
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("TorvoxRust"),
        );
        std::panic::set_hook(Box::new(|info| {
            log::error!("PANIC: {info}");
            if let Some(location) = info.location() {
                log::error!("  at {}:{}", location.file(), location.line());
            }
        }));
        Self {
            config,
            surface: std::sync::Mutex::new(None),
        }
    }

    pub fn ping(&self) -> Result<String, TerminalError> {
        let ptr = self as *const TorvoxBridge;
        log::info!(
            "ping: self={:p}, aligned={}",
            ptr,
            (ptr as usize).is_multiple_of(8)
        );
        Ok("pong".to_string())
    }

    pub fn spawn_terminal(&self, _rows: u32, _cols: u32) -> Result<i32, TerminalError> {
        let shell: torvox_core::config::Shell = self.config.shell.clone().into();
        let shell_path = match &shell {
            torvox_core::config::Shell::SystemDefault => "/system/bin/sh",
            torvox_core::config::Shell::Custom(path) => path.as_str(),
        };
        let mut surface_guard = self.surface.lock().map_err(|e| TerminalError::PtyError {
            detail: format!("lock failed: {}", e),
        })?;
        let surface = surface_guard.as_mut().ok_or(TerminalError::InvalidConfig {
            detail: "no surface — call set_native_window first".to_string(),
        })?;
        let env = self.shell_env();
        surface
            .spawn_session(shell_path, &env)
            .map_err(|e| TerminalError::PtyError {
                detail: e.to_string(),
            })?;
        Ok(0)
    }

    pub fn set_native_window(
        &self,
        window_ptr: i64,
        width: u32,
        height: u32,
    ) -> Result<(), TerminalError> {
        log::debug!(
            "set_native_window: window_ptr={:#x}, width={}, height={}",
            window_ptr,
            width,
            height
        );
        let mut surface_guard = self.surface.lock().map_err(|e| TerminalError::PtyError {
            detail: format!("lock failed: {}", e),
        })?;
        if surface_guard.is_none() {
            // GPU init happens here
            let mut surface = crate::surface::AndroidSurface::new(
                self.config.rows,
                self.config.cols,
                self.config.scrollback_lines,
                self.config.font_size_tenths as f32 / 10.0,
            );
            surface
                .set_native_window(
                    window_ptr as *mut std::ffi::c_void,
                    width,
                    height,
                    self.config.font_size_tenths,
                )
                .map_err(|e| TerminalError::PtyError {
                    detail: e.to_string(),
                })?;

            // NOTE: Do NOT spawn session here. spawn_terminal() is called separately
            // after set_native_window. Spawning here causes double spawn — session #1
            // is immediately killed when spawn_terminal creates session #2.
            *surface_guard = Some(surface);
        }
        Ok(())
    }

    pub fn render(&self) -> Result<(), TerminalError> {
        let mut surface_guard = self.surface.lock().map_err(|e| TerminalError::PtyError {
            detail: format!("lock failed: {}", e),
        })?;
        if let Some(surface) = surface_guard.as_mut() {
            surface.render().map_err(|e| TerminalError::PtyError {
                detail: e.to_string(),
            })?;
        }
        Ok(())
    }

    pub fn save_test_frame(&self, data_dir: &str) -> Result<String, TerminalError> {
        let mut surface_guard = self.surface.lock().map_err(|e| TerminalError::PtyError {
            detail: format!("lock failed: {}", e),
        })?;
        if let Some(surface) = surface_guard.as_mut() {
            surface
                .save_test_frame(data_dir)
                .map_err(|e| TerminalError::PtyError {
                    detail: e.to_string(),
                })
        } else {
            Err(TerminalError::PtyError {
                detail: "No surface".to_string(),
            })
        }
    }

    pub fn poll_bel(&self) -> bool {
        let mut surface_guard = match self.surface.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };
        surface_guard
            .as_mut()
            .map(|s| s.poll_bel())
            .unwrap_or(false)
    }

    pub fn poll_clipboard(&self) -> Option<String> {
        let mut surface_guard = self.surface.lock().ok()?;
        surface_guard.as_mut()?.poll_clipboard()
    }

    pub(crate) fn poll_notification_raw(&self) -> Option<(String, String)> {
        let mut surface_guard = self.surface.lock().ok()?;
        surface_guard.as_mut()?.poll_notification()
    }

    pub fn poll_shell_integration(&self) -> u8 {
        let mut surface_guard = match self.surface.lock() {
            Ok(g) => g,
            Err(_) => return 0,
        };
        surface_guard
            .as_mut()
            .map(|s| s.poll_shell_integration())
            .unwrap_or(0)
    }

    pub fn poll_sync_active(&self) -> bool {
        let mut surface_guard = match self.surface.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };
        surface_guard
            .as_mut()
            .map(|s| s.poll_sync_active())
            .unwrap_or(false)
    }

    pub fn cwd(&self) -> String {
        if let Ok(guard) = self.surface.lock() {
            guard.as_ref().map(|s| s.cwd()).unwrap_or_default()
        } else {
            String::new()
        }
    }

    pub fn focus_event(&self, focused: bool) {
        let Ok(mut guard) = self.surface.lock() else {
            return;
        };
        let Some(s) = guard.as_mut() else { return };
        s.focus_event(focused);
    }

    pub fn resize(&self, rows: u32, cols: u32) -> Result<(), TerminalError> {
        let mut surface_guard = self.surface.lock().map_err(|e| TerminalError::PtyError {
            detail: format!("lock failed: {}", e),
        })?;
        if let Some(surface) = surface_guard.as_mut() {
            surface.resize(rows, cols);
        }
        Ok(())
    }

    pub fn recompute_grid(&self, width: u32, height: u32) -> Result<(), TerminalError> {
        let mut surface_guard = self.surface.lock().map_err(|e| TerminalError::PtyError {
            detail: format!("lock failed: {}", e),
        })?;
        if let Some(surface) = surface_guard.as_mut() {
            surface.recompute_grid(width, height);
        }
        Ok(())
    }

    pub fn update_native_window(
        &self,
        window_ptr: i64,
        width: u32,
        height: u32,
    ) -> Result<(), TerminalError> {
        let mut surface_guard = self.surface.lock().map_err(|e| TerminalError::PtyError {
            detail: format!("lock failed: {}", e),
        })?;
        if let Some(surface) = surface_guard.as_mut() {
            surface
                .update_native_window(window_ptr as *mut std::ffi::c_void, width, height)
                .map_err(|e| TerminalError::PtyError {
                    detail: e.to_string(),
                })?;
        }
        Ok(())
    }

    pub fn release_surface(&self) {
        if let Ok(mut guard) = self.surface.lock() {
            *guard = None;
        }
    }

    pub fn release_gpu_surface(&self) {
        if let Ok(mut guard) = self.surface.lock()
            && let Some(surface) = guard.as_mut()
        {
            surface.release_gpu_surface();
        }
    }

    pub fn scrollback_len(&self) -> u32 {
        self.surface
            .lock()
            .ok()
            .and_then(|g| {
                g.as_ref()
                    .and_then(|s| s.terminal().ok())
                    .map(|t| t.scrollback_len())
            })
            .unwrap_or(0)
    }

    pub fn scrollback_line(&self, index: u32) -> Option<String> {
        self.surface.lock().ok().and_then(|g| {
            g.as_ref()
                .and_then(|s| s.terminal().ok())
                .and_then(|t| t.read_line_text(index))
        })
    }

    pub fn get_config(&self) -> TerminalConfig {
        self.config.clone()
    }

    pub fn get_theme_names(&self) -> Vec<String> {
        torvox_core::config::Theme::all_built_in()
            .into_iter()
            .map(|t| t.name)
            .collect()
    }

    pub fn get_theme(&self, name: String) -> Option<BridgeTheme> {
        torvox_core::config::Theme::all_built_in()
            .into_iter()
            .find(|t| t.name == name)
            .map(|t| t.into())
    }

    pub fn set_font_size(&self, size_tenths: u32) -> Result<(), TerminalError> {
        let size = size_tenths as f32 / 10.0;
        let mut surface_guard = self.surface.lock().map_err(|e| TerminalError::PtyError {
            detail: format!("lock failed: {}", e),
        })?;
        if let Some(surface) = surface_guard.as_mut() {
            surface.set_font_size(size);
        }
        Ok(())
    }

    pub fn set_selection(
        &self,
        start_row: i32,
        start_col: i32,
        end_row: i32,
        end_col: i32,
        active: bool,
    ) -> Result<(), TerminalError> {
        let mut surface_guard = self.surface.lock().map_err(|e| TerminalError::PtyError {
            detail: format!("lock failed: {}", e),
        })?;
        if let Some(surface) = surface_guard.as_mut() {
            if active && start_row >= 0 && end_row >= 0 {
                surface.set_selection(Some(torvox_renderer::gpu::SelectionRange {
                    start_row,
                    start_col,
                    end_row,
                    end_col,
                    active: true,
                }));
            } else {
                surface.set_selection(None);
            }
        }
        Ok(())
    }

    pub fn set_font_family(&self, family_name: String) -> Result<(), TerminalError> {
        let mut surface_guard = self.surface.lock().map_err(|e| TerminalError::PtyError {
            detail: format!("lock failed: {}", e),
        })?;
        for surface in surface_guard.iter_mut() {
            if !surface.set_font_family(&family_name) {
                log::warn!(
                    "FONT_FAMILY: '{}' not found, using bundled font",
                    family_name
                );
            }
        }
        Ok(())
    }

    pub fn set_theme(&self, theme: BridgeTheme) -> Result<(), TerminalError> {
        let mut surface_guard = self.surface.lock().map_err(|e| TerminalError::PtyError {
            detail: format!("lock failed: {}", e),
        })?;
        if let Some(surface) = surface_guard.as_mut() {
            surface.set_theme(theme.into());
        }
        Ok(())
    }

    pub fn search_in_scrollback(&self, query: String) -> Option<String> {
        self.surface.lock().ok().and_then(|g| {
            g.as_ref()
                .and_then(|s| s.terminal().ok())
                .and_then(|t| t.search_in_scrollback(&query))
                .map(|(r, c)| format!("{r},{c}"))
        })
    }

    pub fn list_fonts(&self) -> Vec<String> {
        self.surface
            .lock()
            .ok()
            .and_then(|g| g.as_ref().map(|s| s.font_pipeline().list_monospace_fonts()))
            .unwrap_or_default()
    }

    pub fn set_save_path(&self, path: String) -> Result<(), TerminalError> {
        let mut surface_guard = self.surface.lock().map_err(|e| TerminalError::PtyError {
            detail: format!("lock failed: {}", e),
        })?;
        if let Some(surface) = surface_guard.as_mut() {
            surface.set_save_path(path);
        }
        Ok(())
    }

    pub fn save_session(&self, path: String) -> Result<(), TerminalError> {
        let mut surface_guard = self.surface.lock().map_err(|e| TerminalError::PtyError {
            detail: format!("lock failed: {}", e),
        })?;
        if let Some(surface) = surface_guard.as_mut() {
            surface
                .save_session(&path)
                .map_err(|e| TerminalError::PtyError {
                    detail: e.to_string(),
                })?;
        }
        Ok(())
    }

    pub fn restore_session(&self, path: String) -> Result<(), TerminalError> {
        let mut surface_guard = self.surface.lock().map_err(|e| TerminalError::PtyError {
            detail: format!("lock failed: {}", e),
        })?;
        if let Some(surface) = surface_guard.as_mut() {
            surface
                .restore_session(&path)
                .map_err(|e| TerminalError::PtyError {
                    detail: e.to_string(),
                })?;
        }
        Ok(())
    }

    pub fn has_saved_session(&self, path: String) -> bool {
        crate::surface::AndroidSurface::has_saved_session(&path)
    }

    pub fn set_mouse_position(&self, row: u32, col: u32) -> Result<(), TerminalError> {
        let mut surface_guard = self.surface.lock().map_err(|e| TerminalError::PtyError {
            detail: format!("lock failed: {}", e),
        })?;
        if let Some(surface) = surface_guard.as_mut() {
            surface.set_mouse_position(row, col);
        }
        Ok(())
    }

    pub fn get_hovered_url(&self) -> Option<String> {
        self.surface
            .lock()
            .ok()
            .and_then(|g| g.as_ref().and_then(|s| s.get_hovered_url()))
    }

    pub fn get_terminal_text(&self) -> String {
        let text = self
            .surface
            .lock()
            .ok()
            .and_then(|g| {
                g.as_ref()
                    .and_then(|s| s.terminal().ok())
                    .map(|t| t.read_visible_text())
            })
            .unwrap_or_default();
        let preview_end = text
            .char_indices()
            .nth(80)
            .map(|(i, _)| i)
            .unwrap_or(text.len());
        log::debug!(
            "get_terminal_text: len={}, text={:?}",
            text.len(),
            &text[..preview_end]
        );
        text
    }

    pub fn get_active_session_title(&self) -> String {
        self.surface
            .lock()
            .ok()
            .and_then(|g| g.as_ref().map(|s| s.get_title()))
            .unwrap_or_default()
    }

    pub fn get_grid_rows(&self) -> u32 {
        self.surface
            .lock()
            .ok()
            .and_then(|g| g.as_ref().and_then(|s| s.terminal().ok().map(|t| t.rows())))
            .unwrap_or(24)
    }

    pub fn get_grid_cols(&self) -> u32 {
        self.surface
            .lock()
            .ok()
            .and_then(|g| g.as_ref().and_then(|s| s.terminal().ok().map(|t| t.cols())))
            .unwrap_or(80)
    }

    pub fn write_to_pty(&self, data: Vec<u8>) -> Result<(), TerminalError> {
        let mut surface_guard = self.surface.lock().map_err(|e| TerminalError::PtyError {
            detail: format!("lock failed: {}", e),
        })?;
        if let Some(surface) = surface_guard.as_mut() {
            surface.write_to_pty(&data);
        }
        Ok(())
    }
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`.
/// The ANativeWindow pointer reconstructed from `window_ptr_low` and `window_ptr_high` must be a valid, non-null pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_set_native_window(
    handle: i64,
    window_ptr_low: u32,
    window_ptr_high: u32,
    width: u32,
    height: u32,
) -> i32 {
    log::debug!(
        "set_native_window_ffi: handle={handle}, low={window_ptr_low:#x}, high={window_ptr_high:#x}, width={width}, height={height}"
    );
    let window_ptr = ((window_ptr_high as i64) << 32) | (window_ptr_low as i64);
    log::debug!(
        "set_native_window_ffi: reconstructed window_ptr={:#x}",
        window_ptr
    );
    with_bridge(handle, |bridge| {
        bridge.set_native_window(window_ptr, width, height)
    })
    .map(|_| 0)
    .unwrap_or(-1)
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_resize(handle: i64, rows: u32, cols: u32) -> i32 {
    with_bridge(handle, |bridge| bridge.resize(rows, cols))
        .map(|_| 0)
        .unwrap_or(-1)
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_update_native_window(
    handle: i64,
    window_ptr_low: u32,
    window_ptr_high: u32,
    width: u32,
    height: u32,
) -> i32 {
    let window_ptr = ((window_ptr_high as i64) << 32) | (window_ptr_low as i64);
    with_bridge(handle, |bridge| {
        bridge.update_native_window(window_ptr, width, height)
    })
    .map(|_| 0)
    .unwrap_or(-1)
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_recompute_grid(handle: i64, width: u32, height: u32) -> i32 {
    with_bridge(handle, |bridge| bridge.recompute_grid(width, height))
        .map(|_| 0)
        .unwrap_or(-1)
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_spawn_terminal(handle: i64, rows: u32, cols: u32) -> i32 {
    with_bridge(handle, |bridge| bridge.spawn_terminal(rows, cols))
        .map(|_| 0)
        .unwrap_or(-1)
}

// SAFETY: callers must ensure `ptr` is valid for reads of `len` bytes.
fn read_string(ptr: *const u8, len: i32) -> String {
    if ptr.is_null() || len <= 0 {
        String::new()
    } else {
        let slice = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
        String::from_utf8_lossy(slice).to_string()
    }
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`, or zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_release_gpu_surface(handle: i64) {
    let ptr = handle as *const TorvoxBridge;
    if !ptr.is_null() {
        unsafe {
            (*ptr).release_gpu_surface();
        }
    }
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`, or zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_release_surface(handle: i64) {
    let ptr = handle as *const TorvoxBridge;
    if !ptr.is_null() {
        unsafe {
            (*ptr).release_surface();
        }
    }
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`.
/// `path_ptr` must be valid for reads of `path_len` bytes, and must not be aliased.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_set_save_path(
    handle: i64,
    path_ptr: *const u8,
    path_len: i32,
) -> i32 {
    let path = read_string(path_ptr, path_len);
    with_bridge(handle, |bridge| bridge.set_save_path(path))
        .map(|_| 0)
        .unwrap_or(-1)
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`, or zero.
/// `path_ptr` must be valid for reads of `path_len` bytes, and must not be aliased.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_has_saved_session(
    handle: i64,
    path_ptr: *const u8,
    path_len: i32,
) -> bool {
    let path = read_string(path_ptr, path_len);
    let ptr = handle as *const TorvoxBridge;
    if ptr.is_null() {
        return false;
    }
    unsafe { &*ptr }.has_saved_session(path)
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`.
/// `path_ptr` must be valid for reads of `path_len` bytes, and must not be aliased.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_save_session(
    handle: i64,
    path_ptr: *const u8,
    path_len: i32,
) -> i32 {
    let path = read_string(path_ptr, path_len);
    with_bridge(handle, |bridge| bridge.save_session(path))
        .map(|_| 0)
        .unwrap_or(-1)
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`.
/// `path_ptr` must be valid for reads of `path_len` bytes, and must not be aliased.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_restore_session(
    handle: i64,
    path_ptr: *const u8,
    path_len: i32,
) -> i32 {
    let path = read_string(path_ptr, path_len);
    with_bridge(handle, |bridge| bridge.restore_session(path))
        .map(|_| 0)
        .unwrap_or(-1)
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`.
/// `data_ptr` must be valid for reads of `data_len` bytes, and must not be aliased.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_write_to_pty(
    handle: i64,
    data_ptr: *const u8,
    data_len: i32,
) -> i32 {
    let data = if data_ptr.is_null() || data_len <= 0 {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(data_ptr, data_len as usize) }.to_vec()
    };
    with_bridge(handle, |bridge| bridge.write_to_pty(data))
        .map(|_| 0)
        .unwrap_or(-1)
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`, or zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_get_terminal_text(handle: i64) -> i64 {
    let ptr = handle as *const TorvoxBridge;
    if ptr.is_null() {
        return 0;
    }
    let text = unsafe { &*ptr }.get_terminal_text();
    if text.is_empty() {
        return 0;
    }
    let c_str = std::ffi::CString::new(text).unwrap_or_default();
    c_str.into_raw() as i64
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`, or zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_get_active_session_title(handle: i64) -> i64 {
    let ptr = handle as *const TorvoxBridge;
    if ptr.is_null() {
        return 0;
    }
    let title = unsafe { &*ptr }.get_active_session_title();
    if title.is_empty() {
        return 0;
    }
    let c_str = std::ffi::CString::new(title).unwrap_or_default();
    c_str.into_raw() as i64
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`, or zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_get_grid_rows(handle: i64) -> u32 {
    let ptr = handle as *const TorvoxBridge;
    if ptr.is_null() {
        return 24;
    }
    unsafe { &*ptr }.get_grid_rows()
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`, or zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_get_grid_cols(handle: i64) -> u32 {
    let ptr = handle as *const TorvoxBridge;
    if ptr.is_null() {
        return 80;
    }
    unsafe { &*ptr }.get_grid_cols()
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_set_font_size(handle: i64, size_tenths: u32) -> i32 {
    with_bridge(handle, |bridge| bridge.set_font_size(size_tenths))
        .map(|_| 0)
        .unwrap_or(-1)
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_set_selection(
    handle: i64,
    start_row: i32,
    start_col: i32,
    end_row: i32,
    end_col: i32,
    active: i32,
) -> i32 {
    with_bridge(handle, |bridge| {
        bridge.set_selection(start_row, start_col, end_row, end_col, active != 0)
    })
    .map(|_| 0)
    .unwrap_or(-1)
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`.
/// `family_ptr` must be valid for reads of `family_len` bytes, and must not be aliased.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_set_font_family(
    handle: i64,
    family_ptr: *const u8,
    family_len: i32,
) -> i32 {
    let family = read_string(family_ptr, family_len);
    with_bridge(handle, |bridge| bridge.set_font_family(family))
        .map(|_| 0)
        .unwrap_or(-1)
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`.
/// `theme_ptr` must be valid for reads of `theme_len` bytes, and must not be aliased.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_set_theme(
    handle: i64,
    theme_ptr: *const u8,
    theme_len: i32,
) -> i32 {
    if theme_ptr.is_null() || theme_len <= 0 {
        return -1;
    }
    let bytes = unsafe { std::slice::from_raw_parts(theme_ptr, theme_len as usize) };
    // Deserialize BridgeTheme from boltffi wire format
    let mut pos = 0usize;
    let read_string = |bytes: &[u8], pos: &mut usize| -> String {
        let len = u32::from_le_bytes(bytes[*pos..*pos + 4].try_into().unwrap()) as usize;
        *pos += 4;
        let string_value = String::from_utf8_lossy(&bytes[*pos..*pos + len]).to_string();
        *pos += len;
        string_value
    };
    let read_u32 = |bytes: &[u8], pos: &mut usize| -> u32 {
        let value = u32::from_le_bytes(bytes[*pos..*pos + 4].try_into().unwrap());
        *pos += 4;
        value
    };
    let theme = BridgeTheme {
        name: read_string(bytes, &mut pos),
        bg: read_u32(bytes, &mut pos),
        fg: read_u32(bytes, &mut pos),
        cursor: read_u32(bytes, &mut pos),
        selection_bg: read_u32(bytes, &mut pos),
        ansi0: read_u32(bytes, &mut pos),
        ansi1: read_u32(bytes, &mut pos),
        ansi2: read_u32(bytes, &mut pos),
        ansi3: read_u32(bytes, &mut pos),
        ansi4: read_u32(bytes, &mut pos),
        ansi5: read_u32(bytes, &mut pos),
        ansi6: read_u32(bytes, &mut pos),
        ansi7: read_u32(bytes, &mut pos),
        ansi8: read_u32(bytes, &mut pos),
        ansi9: read_u32(bytes, &mut pos),
        ansi10: read_u32(bytes, &mut pos),
        ansi11: read_u32(bytes, &mut pos),
        ansi12: read_u32(bytes, &mut pos),
        ansi13: read_u32(bytes, &mut pos),
        ansi14: read_u32(bytes, &mut pos),
        ansi15: read_u32(bytes, &mut pos),
    };
    with_bridge(handle, |bridge| bridge.set_theme(theme))
        .map(|_| 0)
        .unwrap_or(-1)
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`, or zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_scrollback_line(handle: i64, index: u32) -> i64 {
    let ptr = handle as *const TorvoxBridge;
    if ptr.is_null() {
        return 0;
    }
    match unsafe { &*ptr }.scrollback_line(index) {
        Some(s) => {
            // Leak string and return pointer; caller must call torvox_bridge_free_string
            let c_str = std::ffi::CString::new(s).unwrap_or_default();
            c_str.into_raw() as i64
        }
        None => 0,
    }
}

/// # Safety
/// `s` must be a valid C string pointer previously returned by `torvox_bridge_scrollback_line` or `torvox_bridge_search_in_scrollback`, or zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_free_string(s: i64) {
    if s != 0 {
        unsafe {
            let _ = std::ffi::CString::from_raw(s as *mut std::ffi::c_char);
        }
    }
}

/// # Safety
/// `handle` must be a valid bridge handle previously returned by `torvox_bridge_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_ping(handle: i64) -> i32 {
    log::debug!("torvox_bridge_ping: handle={handle:#x}");
    match with_bridge(handle, |bridge| bridge.ping()) {
        Ok(_) => 0,
        Err(e) => {
            log::error!("torvox_bridge_ping: error: {e}");
            -1
        }
    }
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_render(handle: i64) -> i32 {
    with_bridge(handle, |bridge| bridge.render())
        .map(|_| 0)
        .unwrap_or(-1)
}

/// # Safety
/// `handle` must be a valid surface handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_poll_bel(handle: i64) -> i32 {
    let ptr = handle as *mut TorvoxBridge;
    if ptr.is_null() {
        return 0;
    }
    let bridge = unsafe { &*ptr };
    if bridge.poll_bel() { 1 } else { 0 }
}

/// # Safety
/// `handle` must be a valid surface handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_poll_shell_integration(handle: i64) -> i32 {
    let ptr = handle as *const TorvoxBridge;
    if ptr.is_null() {
        return 0;
    }
    let bridge = unsafe { &*ptr };
    bridge.poll_shell_integration() as i32
}

/// # Safety
/// `handle` must be a valid surface handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_poll_sync_active(handle: i64) -> i32 {
    let ptr = handle as *const TorvoxBridge;
    if ptr.is_null() {
        return 0;
    }
    let bridge = unsafe { &*ptr };
    if bridge.poll_sync_active() { 1 } else { 0 }
}

/// Save a GPU test frame to disk.
/// `data_dir` must be a valid C string pointing to a writable directory.
/// Returns 0 on success, -1 on error.
/// # Safety
/// `handle` must be a valid surface handle. `data_dir` must be a valid C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_save_test_frame(
    handle: i64,
    data_dir: *const std::ffi::c_char,
) -> i32 {
    if data_dir.is_null() {
        return -1;
    }
    let dir = unsafe { std::ffi::CStr::from_ptr(data_dir) }
        .to_string_lossy()
        .into_owned();
    let ptr = handle as *const TorvoxBridge;
    if ptr.is_null() {
        return -1;
    }
    let bridge = unsafe { &*ptr };
    match bridge.save_test_frame(&dir) {
        Ok(_) => 0,
        Err(e) => {
            log::error!("save_test_frame failed: {e}");
            -1
        }
    }
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`, or zero.
/// Returns a C string pointer that must be freed with `torvox_bridge_free_string`, or 0 if no clipboard text.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_poll_clipboard(handle: i64) -> i64 {
    let ptr = handle as *const TorvoxBridge;
    if ptr.is_null() {
        return 0;
    }
    let bridge = unsafe { &*ptr };
    match bridge.poll_clipboard() {
        Some(text) => {
            let c_str = std::ffi::CString::new(text).unwrap_or_default();
            c_str.into_raw() as i64
        }
        None => 0,
    }
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`, or zero.
/// Returns two consecutive C string pointers (title, body) that must be freed with
/// `torvox_bridge_free_string`, or 0 if no notification.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_poll_notification(handle: i64) -> i64 {
    let ptr = handle as *const TorvoxBridge;
    if ptr.is_null() {
        return 0;
    }
    let bridge = unsafe { &*ptr };
    match bridge.poll_notification_raw() {
        Some((title, body)) => {
            let combined = format!("{title}\0{body}");
            let c_str = std::ffi::CString::new(combined).unwrap_or_default();
            c_str.into_raw() as i64
        }
        None => 0,
    }
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_cwd(handle: i64) -> *mut std::ffi::c_char {
    let ptr = handle as *const TorvoxBridge;
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    let bridge = unsafe { &*ptr };
    let cwd = bridge.cwd();
    let cwd = if cwd.is_empty() { "unknown" } else { &cwd };
    let c_cwd = std::ffi::CString::new(cwd.as_bytes()).unwrap_or_default();
    c_cwd.into_raw()
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_free_cstring(s: *mut std::ffi::c_char) {
    if !s.is_null() {
        unsafe { drop(std::ffi::CString::from_raw(s)) };
    }
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_focus_event(handle: i64, focused: i32) {
    let ptr = handle as *mut TorvoxBridge;
    if ptr.is_null() {
        return;
    }
    let bridge = unsafe { &*ptr };
    bridge.focus_event(focused != 0);
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`, or zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_scrollback_len(handle: i64) -> u32 {
    let ptr = handle as *const TorvoxBridge;
    if ptr.is_null() {
        return 0;
    }
    unsafe { &*ptr }.scrollback_len()
}

/// # Safety
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`, or zero.
/// `query_ptr` must be valid for reads of `query_len` bytes, and must not be aliased.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_search_in_scrollback(
    handle: i64,
    query_ptr: *const u8,
    query_len: i32,
) -> i64 {
    let query = read_string(query_ptr, query_len);
    let ptr = handle as *const TorvoxBridge;
    if ptr.is_null() {
        return 0;
    }
    match unsafe { &*ptr }.search_in_scrollback(query) {
        Some(s) => {
            let c_str = std::ffi::CString::new(s).unwrap_or_default();
            c_str.into_raw() as i64
        }
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_ping() {
        let config = TerminalConfig {
            shell: Shell::Custom {
                path: "/bin/sh".to_string(),
            },
            rows: 24,
            cols: 80,
            scrollback_lines: 50_000,
            font_size_tenths: 140,
            theme: torvox_core::config::Theme::catppuccin_mocha().into(),
            home: String::new(),
            user: String::new(),
            path: String::new(),
            working_directory: String::new(),
            prefix: String::new(),
        };
        let bridge = TorvoxBridge::new(config);
        assert_eq!(bridge.ping().unwrap(), "pong");
    }

    #[test]
    fn bridge_get_config() {
        let config = TerminalConfig {
            shell: Shell::Custom {
                path: "/bin/bash".to_string(),
            },
            rows: 40,
            cols: 120,
            scrollback_lines: 10000,
            font_size_tenths: 160,
            theme: torvox_core::config::Theme::dracula_plus().into(),
            home: String::new(),
            user: String::new(),
            path: String::new(),
            working_directory: String::new(),
            prefix: String::new(),
        };
        let bridge = TorvoxBridge::new(config.clone());
        let got = bridge.get_config();
        assert_eq!(got.shell, config.shell);
        assert_eq!(got.rows, config.rows);
    }

    #[test]
    fn shell_enum_default_is_system() {
        let s = Shell::default();
        assert!(matches!(s, Shell::SystemDefault));
    }

    #[test]
    fn shell_roundtrip_with_core() {
        let core_shell = torvox_core::config::Shell::Custom("/bin/zsh".to_string());
        let bridge_shell: Shell = core_shell.clone().into();
        assert!(matches!(bridge_shell, Shell::Custom { .. }));
        let back: torvox_core::config::Shell = bridge_shell.into();
        assert_eq!(core_shell, back);
    }

    #[test]
    fn terminal_config_roundtrip_with_core() {
        let core_config = torvox_core::config::TerminalConfig::default();
        let bridge_config: TerminalConfig = core_config.clone().into();
        assert!(matches!(bridge_config.shell, Shell::SystemDefault));
        let back: torvox_core::config::TerminalConfig = bridge_config.into();
        assert_eq!(core_config, back);
    }

    #[test]
    fn bridge_attrs_roundtrip() {
        let core_attrs = torvox_core::cell::Attrs {
            bold: true,
            dim: true,
            italic: false,
            underline: true,
            double_underline: false,
            reverse: false,
            strikethrough: true,
            blink: false,
            hidden: false,
            overline: false,
            protected: true,
            double_width: false,
            double_height_top: false,
            double_height_bottom: false,
        };
        let bridge_attrs: BridgeAttrs = core_attrs.into();
        let back: torvox_core::cell::Attrs = bridge_attrs.into();
        assert_eq!(core_attrs, back);
    }

    #[test]
    fn grid_getters_return_defaults_without_surface() {
        let bridge = TorvoxBridge::new(TerminalConfig::default());
        assert!(bridge.get_grid_rows() > 0);
        assert!(bridge.get_grid_cols() > 0);
        assert_eq!(bridge.get_grid_rows(), 24);
        assert_eq!(bridge.get_grid_cols(), 80);
    }

    #[test]
    fn poll_bel_returns_false_without_surface() {
        let bridge = TorvoxBridge::new(TerminalConfig::default());
        assert!(!bridge.poll_bel(), "bell should be false without surface");
    }

    #[test]
    fn poll_bel_idempotent() {
        let bridge = TorvoxBridge::new(TerminalConfig::default());
        assert!(!bridge.poll_bel());
        assert!(
            !bridge.poll_bel(),
            "repeated poll_bel should still be false"
        );
    }

    #[test]
    fn poll_clipboard_returns_none_without_surface() {
        let bridge = TorvoxBridge::new(TerminalConfig::default());
        assert_eq!(
            bridge.poll_clipboard(),
            None,
            "clipboard should be None without surface"
        );
    }

    #[test]
    fn poll_clipboard_idempotent() {
        let bridge = TorvoxBridge::new(TerminalConfig::default());
        assert_eq!(bridge.poll_clipboard(), None);
        assert_eq!(
            bridge.poll_clipboard(),
            None,
            "repeated poll_clipboard should still be None"
        );
    }

    #[test]
    fn poll_shell_integration_returns_zero_without_surface() {
        let bridge = TorvoxBridge::new(TerminalConfig::default());
        assert_eq!(
            bridge.poll_shell_integration(),
            0,
            "shell integration should be 0 without surface"
        );
    }

    #[test]
    fn poll_sync_active_returns_false_without_surface() {
        let bridge = TorvoxBridge::new(TerminalConfig::default());
        assert!(
            !bridge.poll_sync_active(),
            "sync_active should be false without surface"
        );
    }

    #[test]
    fn cwd_returns_empty_without_surface() {
        let bridge = TorvoxBridge::new(TerminalConfig::default());
        assert_eq!(bridge.cwd(), "", "cwd should be empty without surface");
    }

    #[test]
    fn focus_event_does_not_panic_without_surface() {
        let bridge = TorvoxBridge::new(TerminalConfig::default());
        bridge.focus_event(true);
        bridge.focus_event(false);
    }

    #[test]
    fn scrollback_len_zero_without_surface() {
        let bridge = TorvoxBridge::new(TerminalConfig::default());
        assert_eq!(
            bridge.scrollback_len(),
            0,
            "scrollback should be 0 without surface"
        );
    }

    #[test]
    fn set_save_path_succeeds() {
        let bridge = TorvoxBridge::new(TerminalConfig::default());
        let temp_dir = std::env::temp_dir().join("torvox_test_save");
        let result = bridge.set_save_path(temp_dir.to_string_lossy().to_string());
        assert!(
            result.is_ok(),
            "set_save_path should succeed: {:?}",
            result.err()
        );
    }

    #[test]
    fn has_saved_session_false_for_nonexistent() {
        let bridge = TorvoxBridge::new(TerminalConfig::default());
        assert!(!bridge.has_saved_session("/nonexistent/path/session.bin".to_string()));
    }

    #[test]
    fn null_handle_poll_bel_returns_zero() {
        unsafe {
            let result = torvox_bridge_poll_bel(0);
            assert_eq!(result, 0, "null handle poll_bel should return 0");
        }
    }

    #[test]
    fn null_handle_poll_clipboard_returns_zero() {
        unsafe {
            let result = torvox_bridge_poll_clipboard(0);
            assert_eq!(result, 0, "null handle poll_clipboard should return 0");
        }
    }

    #[test]
    fn null_handle_poll_shell_integration_returns_zero() {
        unsafe {
            let result = torvox_bridge_poll_shell_integration(0);
            assert_eq!(
                result, 0,
                "null handle poll_shell_integration should return 0"
            );
        }
    }

    #[test]
    fn null_handle_poll_sync_active_returns_zero() {
        unsafe {
            let result = torvox_bridge_poll_sync_active(0);
            assert_eq!(result, 0, "null handle poll_sync_active should return 0");
        }
    }

    #[test]
    fn null_handle_cwd_returns_null() {
        unsafe {
            let result = torvox_bridge_cwd(0);
            assert!(result.is_null(), "null handle cwd should return null");
        }
    }

    #[test]
    fn null_handle_scrollback_len_returns_zero() {
        unsafe {
            let result = torvox_bridge_scrollback_len(0);
            assert_eq!(result, 0, "null handle scrollback_len should return 0");
        }
    }

    #[test]
    fn null_handle_focus_event_does_not_panic() {
        unsafe {
            torvox_bridge_focus_event(0, 1);
            torvox_bridge_focus_event(0, 0);
        }
    }

    #[test]
    fn free_cstring_null_does_not_panic() {
        unsafe {
            torvox_bridge_free_cstring(std::ptr::null_mut());
        }
    }

    #[test]
    fn free_string_null_does_not_panic() {
        unsafe {
            torvox_bridge_free_string(0);
        }
    }

    #[test]
    fn get_theme_names_returns_all_built_in() {
        let bridge = TorvoxBridge::new(TerminalConfig::default());
        let names = bridge.get_theme_names();
        assert!(
            names.len() >= 10,
            "should have at least 10 built-in themes, got {}",
            names.len()
        );
        assert!(
            names.contains(&"Catppuccin Mocha".to_string()),
            "must include Catppuccin Mocha"
        );
    }

    #[test]
    fn get_theme_returns_known_theme() {
        let bridge = TorvoxBridge::new(TerminalConfig::default());
        let theme = bridge.get_theme("Catppuccin Mocha".to_string());
        assert!(theme.is_some(), "Catppuccin Mocha should exist");
    }

    #[test]
    fn get_theme_returns_none_for_unknown() {
        let bridge = TorvoxBridge::new(TerminalConfig::default());
        let theme = bridge.get_theme("Nonexistent Theme".to_string());
        assert!(theme.is_none(), "unknown theme should return None");
    }

    #[test]
    fn all_poll_apis_are_idempotent() {
        let bridge = TorvoxBridge::new(TerminalConfig::default());
        for _ in 0..10 {
            assert!(!bridge.poll_bel());
            assert_eq!(bridge.poll_clipboard(), None);
            assert_eq!(bridge.poll_shell_integration(), 0);
            assert!(!bridge.poll_sync_active());
        }
    }

    #[test]
    fn concurrent_poll_apis_no_deadlock() {
        let bridge = TorvoxBridge::new(TerminalConfig::default());
        let bridge = &bridge;
        std::thread::scope(|scope| {
            let h1 = scope.spawn(|| {
                for _ in 0..100 {
                    let _ = bridge.poll_bel();
                }
            });
            let h2 = scope.spawn(|| {
                for _ in 0..100 {
                    let _ = bridge.poll_clipboard();
                }
            });
            let h3 = scope.spawn(|| {
                for _ in 0..100 {
                    let _ = bridge.poll_shell_integration();
                }
            });
            let h4 = scope.spawn(|| {
                for _ in 0..100 {
                    let _ = bridge.poll_sync_active();
                }
            });
            let h5 = scope.spawn(|| {
                for _ in 0..100 {
                    let _ = bridge.cwd();
                }
            });
            let h6 = scope.spawn(|| {
                for _ in 0..100 {
                    let _ = bridge.scrollback_len();
                }
            });
            h1.join().unwrap();
            h2.join().unwrap();
            h3.join().unwrap();
            h4.join().unwrap();
            h5.join().unwrap();
            h6.join().unwrap();
        });
    }

    #[test]
    fn bridge_new_and_free_roundtrip() {
        let config = TerminalConfig::default();
        let bridge = Box::new(TorvoxBridge::new(config));
        let ptr = Box::into_raw(bridge);
        unsafe {
            let _ = Box::from_raw(ptr);
        }
    }

    #[test]
    fn get_config_preserves_all_fields() {
        let config = TerminalConfig {
            shell: Shell::Custom {
                path: "/bin/fish".to_string(),
            },
            rows: 50,
            cols: 160,
            scrollback_lines: 200000,
            font_size_tenths: 200,
            theme: torvox_core::config::Theme::dracula_plus().into(),
            home: "/home/test".to_string(),
            user: "testuser".to_string(),
            path: "/usr/bin:/usr/local/bin".to_string(),
            working_directory: "/tmp".to_string(),
            prefix: "myterm".to_string(),
        };
        let bridge = TorvoxBridge::new(config.clone());
        let got = bridge.get_config();
        assert_eq!(got.shell, config.shell);
        assert_eq!(got.rows, 50);
        assert_eq!(got.cols, 160);
        assert_eq!(got.scrollback_lines, 200000);
        assert_eq!(got.font_size_tenths, 200);
        assert_eq!(got.home, "/home/test");
        assert_eq!(got.user, "testuser");
        assert_eq!(got.path, "/usr/bin:/usr/local/bin");
        assert_eq!(got.working_directory, "/tmp");
        assert_eq!(got.prefix, "myterm");
    }
}
