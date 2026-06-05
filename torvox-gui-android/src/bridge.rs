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
    fn from(s: Shell) -> Self {
        match s {
            Shell::SystemDefault => torvox_core::config::Shell::SystemDefault,
            Shell::Custom { path } => torvox_core::config::Shell::Custom(path),
        }
    }
}

impl From<torvox_core::config::Shell> for Shell {
    fn from(s: torvox_core::config::Shell) -> Self {
        match s {
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
    fn from(t: torvox_core::config::Theme) -> Self {
        Self {
            name: t.name,
            bg: rgb_to_u32(t.bg),
            fg: rgb_to_u32(t.fg),
            cursor: rgb_to_u32(t.cursor),
            selection_bg: rgb_to_u32(t.bg), // default selection bg = background
            ansi0: rgb_to_u32(t.ansi[0]),
            ansi1: rgb_to_u32(t.ansi[1]),
            ansi2: rgb_to_u32(t.ansi[2]),
            ansi3: rgb_to_u32(t.ansi[3]),
            ansi4: rgb_to_u32(t.ansi[4]),
            ansi5: rgb_to_u32(t.ansi[5]),
            ansi6: rgb_to_u32(t.ansi[6]),
            ansi7: rgb_to_u32(t.ansi[7]),
            ansi8: rgb_to_u32(t.ansi[8]),
            ansi9: rgb_to_u32(t.ansi[9]),
            ansi10: rgb_to_u32(t.ansi[10]),
            ansi11: rgb_to_u32(t.ansi[11]),
            ansi12: rgb_to_u32(t.ansi[12]),
            ansi13: rgb_to_u32(t.ansi[13]),
            ansi14: rgb_to_u32(t.ansi[14]),
            ansi15: rgb_to_u32(t.ansi[15]),
        }
    }
}

impl From<BridgeTheme> for torvox_core::config::Theme {
    fn from(t: BridgeTheme) -> Self {
        Self {
            name: t.name,
            bg: u32_to_rgb(t.bg),
            fg: u32_to_rgb(t.fg),
            cursor: u32_to_rgb(t.cursor),
            ansi: [
                u32_to_rgb(t.ansi0),
                u32_to_rgb(t.ansi1),
                u32_to_rgb(t.ansi2),
                u32_to_rgb(t.ansi3),
                u32_to_rgb(t.ansi4),
                u32_to_rgb(t.ansi5),
                u32_to_rgb(t.ansi6),
                u32_to_rgb(t.ansi7),
                u32_to_rgb(t.ansi8),
                u32_to_rgb(t.ansi9),
                u32_to_rgb(t.ansi10),
                u32_to_rgb(t.ansi11),
                u32_to_rgb(t.ansi12),
                u32_to_rgb(t.ansi13),
                u32_to_rgb(t.ansi14),
                u32_to_rgb(t.ansi15),
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
        }
    }
}

impl From<TerminalConfig> for torvox_core::config::TerminalConfig {
    fn from(c: TerminalConfig) -> Self {
        Self {
            shell: c.shell.into(),
            rows: c.rows,
            cols: c.cols,
            scrollback_lines: c.scrollback_lines,
            font_size_tenths: c.font_size_tenths,
        }
    }
}

impl From<torvox_core::config::TerminalConfig> for TerminalConfig {
    fn from(c: torvox_core::config::TerminalConfig) -> Self {
        Self {
            shell: c.shell.into(),
            rows: c.rows,
            cols: c.cols,
            scrollback_lines: c.scrollback_lines,
            font_size_tenths: c.font_size_tenths,
            theme: torvox_core::config::Theme::catppuccin_mocha().into(),
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
    fn from(e: torvox_core::event::TerminalEvent) -> Self {
        match e {
            torvox_core::event::TerminalEvent::OutputReady => TerminalEvent::DirtyRegion {
                start_row: 0,
                end_row: 0,
            },
            torvox_core::event::TerminalEvent::Bell => TerminalEvent::Bell,
            torvox_core::event::TerminalEvent::TitleChanged(t) => {
                TerminalEvent::TitleChanged { title: t }
            }
            torvox_core::event::TerminalEvent::ClipboardRequest(t) => {
                TerminalEvent::ClipboardRequest { text: t }
            }
            torvox_core::event::TerminalEvent::HyperlinkHover(u) => {
                TerminalEvent::HyperlinkHover { url: u }
            }
            torvox_core::event::TerminalEvent::ProcessExited(c) => {
                TerminalEvent::ProcessExited { exit_code: c }
            }
            torvox_core::event::TerminalEvent::CursorChanged(cursor) => {
                TerminalEvent::CursorChanged {
                    row: cursor.row,
                    col: cursor.col,
                }
            }
            torvox_core::event::TerminalEvent::SelectionChanged(sel) => match sel {
                Some(s) => {
                    let (lo, hi) = s.ordered();
                    TerminalEvent::SelectionChanged {
                        start_row: lo.row,
                        start_col: lo.col,
                        end_row: hi.row,
                        end_col: hi.col,
                    }
                }
                None => TerminalEvent::SelectionChanged {
                    start_row: 0,
                    start_col: 0,
                    end_row: 0,
                    end_col: 0,
                },
            },
            torvox_core::event::TerminalEvent::DirtyRegion(dr) => TerminalEvent::DirtyRegion {
                start_row: dr.start_row,
                end_row: dr.end_row,
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
    shell_path: std::ffi::CString,
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
        let shell_path = match &config.shell {
            Shell::SystemDefault => std::ffi::CString::new("/system/bin/sh").unwrap(),
            Shell::Custom { path } => std::ffi::CString::new(path.as_str()).unwrap(),
        };
        Self {
            config,
            surface: std::sync::Mutex::new(None),
            shell_path,
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
        surface
            .spawn_session(shell_path)
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
            // Copy shell path to stack buffer BEFORE any GPU work (heap may be corrupted by wgpu)
            let mut shell_buf = [0u8; 4096];
            let shell_len = {
                let bytes = self.shell_path.to_bytes();
                let n = bytes.len().min(4095);
                shell_buf[..n].copy_from_slice(&bytes[..n]);
                n
            };

            // GPU init happens here — heap may be corrupted, but stack buffer is safe
            let mut surface = crate::surface::AndroidSurface::new(
                self.config.rows,
                self.config.cols,
                self.config.scrollback_lines,
            );
            surface
                .set_native_window(window_ptr as *mut std::ffi::c_void, width, height)
                .map_err(|e| TerminalError::PtyError {
                    detail: e.to_string(),
                })?;

            // Use stack buffer after GPU init (immune to heap corruption)
            let shell_path = std::str::from_utf8(&shell_buf[..shell_len]).map_err(|_| {
                TerminalError::PtyError {
                    detail: "invalid shell path".to_string(),
                }
            })?;
            surface
                .spawn_session(shell_path)
                .map_err(|e| TerminalError::PtyError {
                    detail: e.to_string(),
                })?;
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

    pub fn render_software(&self) -> Result<(), TerminalError> {
        let mut surface_guard = self.surface.lock().map_err(|e| TerminalError::PtyError {
            detail: format!("lock failed: {}", e),
        })?;
        if let Some(surface) = surface_guard.as_mut() {
            surface
                .render_software()
                .map_err(|e| TerminalError::PtyError {
                    detail: e.to_string(),
                })?;
        }
        Ok(())
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

    pub fn release_surface(&self) {
        if let Ok(mut guard) = self.surface.lock() {
            *guard = None;
        }
    }

    pub fn scrollback_len(&self) -> u32 {
        self.surface
            .lock()
            .ok()
            .and_then(|g| g.as_ref().map(|s| s.terminal().scrollback_len()))
            .unwrap_or(0)
    }

    pub fn scrollback_line(&self, index: u32) -> Option<String> {
        self.surface
            .lock()
            .ok()
            .and_then(|g| g.as_ref().and_then(|s| s.terminal().read_line_text(index)))
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
                .and_then(|s| s.terminal().search_in_scrollback(&query))
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
            .and_then(|g| g.as_ref().map(|s| s.terminal().read_visible_text()))
            .unwrap_or_default();
        log::debug!(
            "get_terminal_text: len={}, text={:?}",
            text.len(),
            &text[..text.len().min(80)]
        );
        text
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
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_set_font_size(handle: i64, size_tenths: u32) -> i32 {
    with_bridge(handle, |bridge| bridge.set_font_size(size_tenths))
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
    // 从 boltffi 线格式反序列化 BridgeTheme
    let mut pos = 0usize;
    let read_string = |bytes: &[u8], pos: &mut usize| -> String {
        let len = u32::from_le_bytes(bytes[*pos..*pos + 4].try_into().unwrap()) as usize;
        *pos += 4;
        let s = String::from_utf8_lossy(&bytes[*pos..*pos + len]).to_string();
        *pos += len;
        s
    };
    let read_u32 = |bytes: &[u8], pos: &mut usize| -> u32 {
        let v = u32::from_le_bytes(bytes[*pos..*pos + 4].try_into().unwrap());
        *pos += 4;
        v
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
            // 泄露字符串并返回指针；调用者必须调用 torvox_bridge_free_string
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
/// `handle` must be a valid surface handle previously returned by `torvox_bridge_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn torvox_bridge_render_software(handle: i64) -> i32 {
    with_bridge(handle, |bridge| bridge.render_software())
        .map(|_| 0)
        .unwrap_or(-1)
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
            theme: torvox_core::config::Theme::dracula().into(),
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
        };
        let bridge_attrs: BridgeAttrs = core_attrs.into();
        let back: torvox_core::cell::Attrs = bridge_attrs.into();
        assert_eq!(core_attrs, back);
    }
}
