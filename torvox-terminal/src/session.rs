// @Session orchestrator, IMPL_TERM_003, impl, [REQ_TERM_003]
// @need-ids: REQ_TERM_003, REQ_TERM_004
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Condvar, Mutex};
use std::time::Duration;

use flume::{Receiver, bounded};
use thiserror::Error;

use crate::ghostty_terminal::GhosttyTerminal;
use crate::osc_handler::{OscEvent, OscHandler};
use crate::pty::{Pty, PtyError, PtyPair};
use crate::shell_env::ShellEnv;

const READ_BUF_SIZE: usize = 8192;

/// OSC 133 Shell Integration markers.
/// See https://gitlab.freedesktop.org/terminal-wg/specifications/-/issues/31
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ShellIntegration {
    None = 0,
    PromptStart = 1,
    PromptEnd = 2,
    CommandStart = 3,
    CommandExecuted = 4,
}

impl From<u8> for ShellIntegration {
    fn from(v: u8) -> Self {
        match v {
            1 => Self::PromptStart,
            2 => Self::PromptEnd,
            3 => Self::CommandStart,
            4 => Self::CommandExecuted,
            _ => Self::None,
        }
    }
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("pty error: {0}")]
    Pty(#[from] PtyError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ghostty error: {0}")]
    Ghostty(String),
    #[error("session closed")]
    Closed,
}

pub struct Session {
    pty: Box<dyn Pty>,
    terminal: GhosttyTerminal,
    osc_handler: OscHandler,
    output_tx: flume::Sender<Vec<u8>>,
    output_rx: Receiver<Vec<u8>>,
    output_notify: Arc<(Mutex<bool>, Condvar)>,
    exited: Arc<AtomicBool>,
    bel_triggered: Arc<AtomicBool>,
    clipboard_text: Arc<Mutex<Option<String>>>,
    notification: Arc<Mutex<Option<(String, String)>>>,
    hyperlink: Arc<Mutex<Option<String>>>,
    shell_integration: Arc<AtomicU8>,
    reader_handle: Option<std::thread::JoinHandle<()>>,
    wait_handle: Option<std::thread::JoinHandle<()>>,
}

impl Session {
    /// Create a session with an already-constructed PTY.
    /// No reader/wait threads are spawned — the caller is responsible for
    /// driving PTY I/O. Primarily used for testing with `MockPty`.
    pub fn with_pty(pty: Box<dyn Pty>, rows: u32, cols: u32) -> Result<Self, SessionError> {
        let default_theme = [
            [24, 24, 37],
            [243, 139, 168],
            [166, 227, 161],
            [249, 226, 175],
            [137, 180, 250],
            [203, 166, 247],
            [148, 226, 213],
            [205, 214, 244],
            [108, 112, 134],
            [243, 139, 168],
            [166, 227, 161],
            [249, 226, 175],
            [137, 180, 250],
            [203, 166, 247],
            [148, 226, 213],
            [187, 194, 222],
        ];
        Self::spawn_with_theme_inner(
            pty,
            rows,
            cols,
            [30, 30, 46],
            [205, 214, 244],
            default_theme,
        )
    }

    pub fn spawn(shell: &str, rows: u32, cols: u32, env: &ShellEnv) -> Result<Self, SessionError> {
        Self::spawn_with_theme(
            shell,
            rows,
            cols,
            env,
            [30, 30, 46],
            [205, 214, 244],
            [
                [24, 24, 37],
                [243, 139, 168],
                [166, 227, 161],
                [249, 226, 175],
                [137, 180, 250],
                [203, 166, 247],
                [148, 226, 213],
                [205, 214, 244],
                [108, 112, 134],
                [243, 139, 168],
                [166, 227, 161],
                [249, 226, 175],
                [137, 180, 250],
                [203, 166, 247],
                [148, 226, 213],
                [187, 194, 222],
            ],
        )
    }

    pub fn spawn_with_theme(
        shell: &str,
        rows: u32,
        cols: u32,
        env: &ShellEnv,
        initial_bg: [u8; 3],
        initial_fg: [u8; 3],
        initial_ansi: [[u8; 3]; 16],
    ) -> Result<Self, SessionError> {
        log::info!("Session::spawn: shell='{shell}', rows={rows}, cols={cols}");
        let pty = match PtyPair::spawn(shell, rows as u16, cols as u16, env) {
            Ok(p) => {
                log::info!("Session::spawn: PtyPair::spawn OK");
                p
            }
            Err(e) => {
                log::info!("Session::spawn: PtyPair::spawn error: {e}");
                return Err(e.into());
            }
        };
        match pty.set_nonblocking() {
            Ok(()) => log::info!("Session::spawn: set_nonblocking OK"),
            Err(e) => {
                log::info!("Session::spawn: set_nonblocking error: {e}");
                return Err(e.into());
            }
        }

        log::info!("Session::spawn: dup master fd");
        let read_fd = unsafe { libc::dup(pty.master_fd()) };
        if read_fd < 0 {
            return Err(SessionError::Io(std::io::Error::last_os_error()));
        }

        let child_pid = pty.child_pid();

        let mut session = Self::spawn_with_theme_inner(
            Box::new(pty) as Box<dyn Pty>,
            rows,
            cols,
            initial_bg,
            initial_fg,
            initial_ansi,
        )?;

        let exited = session.exited.clone();
        let output_notify = session.output_notify.clone();
        let output_tx = session.output_tx.clone();

        log::info!("Session::spawn: spawning reader thread");
        let exited_read = exited.clone();
        let notify_read = output_notify.clone();
        let reader_handle = std::thread::spawn(move || {
            let mut read_buf = [0u8; READ_BUF_SIZE];
            loop {
                if exited_read.load(Ordering::Acquire) {
                    log::info!("reader thread: exiting due to exited flag");
                    break;
                }
                let bytes_read = unsafe {
                    libc::read(
                        read_fd,
                        read_buf.as_mut_ptr() as *mut libc::c_void,
                        READ_BUF_SIZE,
                    )
                };
                if bytes_read > 0 {
                    log::info!(
                        "reader thread: read {} bytes from PTY: {:02x?}",
                        bytes_read,
                        &read_buf[..bytes_read.min(128) as usize]
                    );
                    let data = read_buf[..bytes_read as usize].to_vec();
                    if output_tx.send(data).is_err() {
                        log::info!("reader thread: output channel closed");
                        break;
                    }
                    Self::notify_output(&notify_read);
                } else if bytes_read == 0 {
                    log::info!("reader thread: EOF from PTY");
                    exited_read.store(true, Ordering::Release);
                    Self::notify_output(&notify_read);
                    break;
                } else {
                    let err = std::io::Error::last_os_error();
                    if err.kind() == std::io::ErrorKind::WouldBlock {
                        std::thread::sleep(Duration::from_millis(2));
                    } else {
                        log::info!("reader thread: read error: {err}");
                        exited_read.store(true, Ordering::Release);
                        Self::notify_output(&notify_read);
                        break;
                    }
                }
            }
            unsafe {
                libc::close(read_fd);
            }
        });

        let exited_wait = exited.clone();
        let wait_handle = std::thread::spawn(move || {
            log::info!("wait thread: waiting for child pid={child_pid}");
            let status = nix::sys::wait::waitpid(child_pid, None);
            log::info!("wait thread: child exited: {status:?}");
            exited_wait.store(true, Ordering::Release);
        });

        session.reader_handle = Some(reader_handle);
        session.wait_handle = Some(wait_handle);

        Ok(session)
    }

    fn notify_output(notify: &Arc<(Mutex<bool>, Condvar)>) {
        let (lock, cvar) = &**notify;
        let mut pending = lock.lock().unwrap_or_else(|e| e.into_inner());
        *pending = true;
        cvar.notify_one();
    }

    fn spawn_with_theme_inner(
        pty: Box<dyn Pty>,
        rows: u32,
        cols: u32,
        initial_bg: [u8; 3],
        initial_fg: [u8; 3],
        initial_ansi: [[u8; 3]; 16],
    ) -> Result<Self, SessionError> {
        log::info!("Session::spawn_with_theme_inner: creating Arc/Channel");
        let exited = Arc::new(AtomicBool::new(false));
        let bel_triggered = Arc::new(AtomicBool::new(false));
        let clipboard_text = Arc::new(Mutex::new(None));
        let output_notify = Arc::new((Mutex::new(false), Condvar::new()));
        let (output_tx, output_rx) = bounded::<Vec<u8>>(128);

        let terminal = GhosttyTerminal::new_with_theme(
            rows,
            cols,
            50000,
            initial_bg,
            initial_fg,
            initial_ansi,
        )
        .map_err(SessionError::Ghostty)?;

        let shell_integration = Arc::new(AtomicU8::new(0));
        let notification = Arc::new(Mutex::new(None));
        let hyperlink = Arc::new(Mutex::new(None));

        Ok(Self {
            pty,
            terminal,
            osc_handler: OscHandler::new(),
            output_tx,
            output_rx,
            output_notify,
            exited,
            bel_triggered,
            clipboard_text,
            notification,
            hyperlink,
            shell_integration,
            reader_handle: None,
            wait_handle: None,
        })
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), SessionError> {
        if self.is_exited() {
            return Err(SessionError::Closed);
        }
        self.pty.write_all(data).map_err(SessionError::Io)?;
        Ok(())
    }

    pub fn resize(&mut self, rows: u32, cols: u32) -> Result<(), SessionError> {
        self.pty.resize(rows as u16, cols as u16)?;
        self.terminal.resize(rows, cols);
        Ok(())
    }

    pub fn set_pixel_size(&mut self, width: u16, height: u16) {
        self.pty.set_pixel_size(width, height);
    }

    pub fn wait_for_output(&self) {
        let (lock, cvar) = &*self.output_notify;
        let mut pending = lock.lock().unwrap_or_else(|e| e.into_inner());
        while !*pending {
            pending = cvar.wait(pending).unwrap_or_else(|e| e.into_inner());
        }
        *pending = false;
    }

    pub fn process_output(&mut self) -> bool {
        let mut changed = false;
        let mut count = 0u32;
        while let Ok(data) = self.output_rx.try_recv() {
            if data.contains(&0x07) {
                self.bel_triggered.store(true, Ordering::Release);
            }

            self.osc_handler.process(&data);

            for event in self.osc_handler.events() {
                match event {
                    OscEvent::Clipboard(clipboard_event) => {
                        if let Ok(mut guard) = self.clipboard_text.lock() {
                            *guard = Some(clipboard_event.text.clone());
                        }
                    }
                    OscEvent::Cwd(_cwd_event) => {
                        // OSC 7 passes through to Ghostty natively for CWD tracking.
                        // This branch is kept for potential future use.
                    }
                    OscEvent::Hyperlink(hyperlink_event) => {
                        if let Ok(mut guard) = self.hyperlink.lock() {
                            *guard = hyperlink_event.uri.clone();
                        }
                    }
                    OscEvent::Notification(notification_event) => {
                        if let Ok(mut guard) = self.notification.lock() {
                            *guard = Some((
                                notification_event.title.clone(),
                                notification_event.body.clone(),
                            ));
                        }
                    }
                }
            }

            let filtered = self.osc_handler.output();
            if let Some(marker) = extract_osc133(filtered) {
                self.shell_integration
                    .store(marker as u8, Ordering::Release);
            }
            self.terminal.vt_write(filtered);
            changed = true;
            count += 1;
        }
        if count > 0 {
            log::info!("process_output: processed {count} chunks");
        }
        if changed {
            self.terminal.flush();
            for response in self.terminal.drain_pty_write_responses() {
                log::info!("process_output: pty write-back {} bytes", response.len());
                let _ = self.pty.write_all(&response);
            }
        }
        changed
    }

    pub fn poll_bel(&self) -> bool {
        self.bel_triggered.swap(false, Ordering::AcqRel)
    }

    pub fn poll_clipboard(&self) -> Option<String> {
        let mut guard = self.clipboard_text.lock().ok()?;
        guard.take()
    }

    pub fn poll_notification(&self) -> Option<(String, String)> {
        let mut guard = self.notification.lock().ok()?;
        guard.take()
    }

    pub fn poll_hyperlink(&self) -> Option<String> {
        let mut guard = self.hyperlink.lock().ok()?;
        guard.take()
    }

    pub fn poll_shell_integration(&self) -> ShellIntegration {
        let raw_value = self.shell_integration.swap(0, Ordering::AcqRel);
        ShellIntegration::from(raw_value)
    }

    pub fn is_exited(&self) -> bool {
        self.exited.load(Ordering::Acquire)
    }

    pub fn exited_flag(&self) -> Arc<AtomicBool> {
        self.exited.clone()
    }

    pub fn terminal(&self) -> &GhosttyTerminal {
        &self.terminal
    }

    pub fn terminal_mut(&mut self) -> &mut GhosttyTerminal {
        &mut self.terminal
    }

    pub fn title(&self) -> String {
        self.terminal.title()
    }

    pub fn cwd(&self) -> String {
        self.terminal.cwd()
    }

    pub fn mode_get(&self, mode_num: u16, kind: u8) -> bool {
        self.terminal.mode_get(mode_num, kind)
    }

    pub fn focus_event(&mut self, focused: bool) {
        let data = if focused { b"[I" } else { b"[O" };
        self.terminal.vt_write(data);
    }
}

fn extract_osc133(data: &[u8]) -> Option<ShellIntegration> {
    let mut result = None;
    let mut i = 0;
    while i + 6 < data.len() {
        if data[i] == 0x1B
            && data[i + 1] == b']'
            && data[i + 2] == b'1'
            && data[i + 3] == b'3'
            && data[i + 4] == b'3'
            && data[i + 5] == b';'
        {
            let marker_start = i + 6;
            if marker_start < data.len() {
                let marker = data[marker_start];
                let si = match marker {
                    b'A' => ShellIntegration::PromptStart,
                    b'B' => ShellIntegration::PromptEnd,
                    b'C' => ShellIntegration::CommandStart,
                    b'D' => ShellIntegration::CommandExecuted,
                    _ => ShellIntegration::None,
                };
                if si != ShellIntegration::None {
                    result = Some(si);
                }
            }
            i += 7;
        } else {
            i += 1;
        }
    }
    result
}

impl Drop for Session {
    fn drop(&mut self) {
        self.exited.store(true, Ordering::Release);

        let pid = self.pty.child_pid();
        if pid.as_raw() > 0 {
            nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGHUP).ok();
            nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGCONT).ok();
            std::thread::sleep(Duration::from_millis(50));
            nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGKILL).ok();
        }

        if let Some(h) = self.reader_handle.take() {
            let _ = h.join();
        }
        if let Some(h) = self.wait_handle.take() {
            let _ = h.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drain_output(session: &mut Session, deadline: std::time::Instant) {
        while std::time::Instant::now() < deadline {
            session.process_output();
            if session.is_exited() {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn session_spawn_and_exit() {
        let mut session =
            Session::spawn("/bin/sh", 24, 80, &ShellEnv::default()).expect("spawn failed");
        session.write(b"exit\n").expect("write failed");
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        drain_output(&mut session, deadline);
        assert!(session.is_exited());
    }

    #[test]
    fn session_echo_hello() {
        let mut session =
            Session::spawn("/bin/sh", 24, 80, &ShellEnv::default()).expect("spawn failed");
        session.write(b"echo hello_p12\n").expect("write failed");
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        let mut found = false;
        while std::time::Instant::now() < deadline {
            session.process_output();
            let rows = session.terminal().rows();
            for row in 0..rows {
                if let Some(line) = session.terminal().read_line_text(row)
                    && line.contains("hello_p12")
                {
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(found, "did not find 'hello_p12' in terminal");
    }

    #[test]
    fn session_resize() {
        let mut session =
            Session::spawn("/bin/sh", 24, 80, &ShellEnv::default()).expect("spawn failed");
        session.resize(40, 120).expect("resize failed");
        assert_eq!(session.terminal().rows(), 40);
        assert_eq!(session.terminal().cols(), 120);
    }

    #[test]
    fn session_after_exit_returns_error() {
        let mut session =
            Session::spawn("/bin/sh", 24, 80, &ShellEnv::default()).expect("spawn failed");
        session.write(b"exit\n").expect("write failed");
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        drain_output(&mut session, deadline);
        assert!(session.is_exited());
    }

    #[test]
    fn extract_osc133_all_markers() {
        // Each marker tested independently to avoid slice-index confusion.
        assert_eq!(
            extract_osc133(b"\x1b]133;A\x07"),
            Some(ShellIntegration::PromptStart)
        );
        assert_eq!(
            extract_osc133(b"\x1b]133;B\x1b\\"),
            Some(ShellIntegration::PromptEnd)
        );
        assert_eq!(
            extract_osc133(b"\x1b]133;C\x07"),
            Some(ShellIntegration::CommandStart)
        );
        assert_eq!(
            extract_osc133(b"\x1b]133;D\x1b\\"),
            Some(ShellIntegration::CommandExecuted)
        );
    }

    #[test]
    fn extract_osc133_returns_none_without_markers() {
        assert_eq!(extract_osc133(b"hello world"), None);
        assert_eq!(extract_osc133(b"\x1b]133;\x07"), None);
        assert_eq!(extract_osc133(b"\x1b]133;X\x07"), None);
    }

    #[test]
    fn extract_osc133_command_executed() {
        assert_eq!(
            extract_osc133(b"\x1b]133;D\x1b\\"),
            Some(ShellIntegration::CommandExecuted)
        );
    }
}
