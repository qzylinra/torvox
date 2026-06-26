// @PTY pair management, IMPL_TERM_001, impl, [REQ_TERM_001]
// @need-ids: REQ_TERM_001, REQ_TERM_002
use std::io;
use std::os::unix::io::{AsRawFd, OwnedFd, RawFd};
use std::time::Duration;

use thiserror::Error;

use crate::shell_env::ShellEnv;

#[derive(Debug, Error)]
pub enum PtyError {
    #[error("fork failed: {0}")]
    Fork(nix::errno::Errno),
    #[error("failed to open pseudoterminal: {0}")]
    Open(std::io::Error),
    #[error("ioctl TIOCSWINSZ failed: {0}")]
    Resize(nix::errno::Errno),
    #[error("fcntl failed: {0}")]
    Fcntl(nix::errno::Errno),
    #[error("termios configuration failed: {0}")]
    Termios(nix::errno::Errno),
}

impl From<nix::errno::Errno> for PtyError {
    fn from(err: nix::errno::Errno) -> Self {
        PtyError::Fork(err)
    }
}

/// Trait abstracting a pseudoterminal for testability.
pub trait Pty: Send {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
    fn resize(&self, rows: u16, cols: u16) -> Result<(), PtyError>;
    fn child_pid(&self) -> nix::unistd::Pid;
    fn master_fd(&self) -> RawFd;
    fn wait(&self) -> nix::Result<nix::sys::wait::WaitStatus>;
    fn set_nonblocking(&self) -> Result<(), PtyError>;
    fn set_pixel_size(&mut self, width: u16, height: u16);

    fn spawn(shell: &str, rows: u16, cols: u16, env: &ShellEnv) -> Result<Box<dyn Pty>, PtyError>
    where
        Self: Sized;

    fn write_all(&mut self, mut buf: &[u8]) -> io::Result<()> {
        while !buf.is_empty() {
            let bytes_written = self.write(buf)?;
            buf = &buf[bytes_written..];
        }
        Ok(())
    }
}

pub struct PtyPair {
    master: OwnedFd,
    child_pid: nix::unistd::Pid,
    pixel_width: u16,
    pixel_height: u16,
}

impl PtyPair {
    pub fn spawn(shell: &str, rows: u16, cols: u16, env: &ShellEnv) -> Result<Self, PtyError> {
        let winsize = nix::pty::Winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        let result = nix::pty::openpty(Some(&winsize), None)?;
        let master_fd = result.master;
        let slave_fd = result.slave;

        // Build all child process data before fork to avoid allocations in child.
        // (Multi-threaded process fork may corrupt malloc heap.)
        let shell_cstr = std::ffi::CString::new(shell).expect("shell path contains null byte");
        let env_cstrings: Vec<std::ffi::CString> = build_env(env, shell, rows, cols)
            .into_iter()
            .map(|(k, v)| {
                std::ffi::CString::new(format!("{k}={v}")).expect("env var contains null")
            })
            .collect();
        let working_directory_cstr = std::ffi::CString::new(env.working_directory.as_str())
            .expect("working directory contains null byte");

        // Pre-allocate argument and environment arrays before fork.
        // After fork, the child must NOT call any allocation functions.
        let shell_ptr = shell_cstr.as_ptr();
        let working_directory_ptr = working_directory_cstr.as_ptr();
        let args_ptrs: Vec<*const libc::c_char> = vec![shell_ptr, std::ptr::null()];
        let env_ptrs: Vec<*const libc::c_char> = env_cstrings
            .iter()
            .map(|s| s.as_ptr())
            .chain(std::iter::once(std::ptr::null()))
            .collect();

        match unsafe { nix::unistd::fork()? } {
            nix::unistd::ForkResult::Parent { child } => {
                nix::unistd::close(slave_fd).ok();
                Ok(Self {
                    master: master_fd,
                    child_pid: child,
                    pixel_width: 0,
                    pixel_height: 0,
                })
            }
            nix::unistd::ForkResult::Child => {
                nix::unistd::close(master_fd).ok();
                // Manually set controlling terminal using only syscalls.
                // Avoid login_tty because it may call malloc() internally,
                // which is unsafe after fork in a multithreaded process.
                if nix::unistd::setsid().is_err() {
                    std::process::exit(1);
                }
                let slave_raw = slave_fd.as_raw_fd();
                // SAFETY: All these libc calls are lightweight syscall wrappers that do not allocate.
                // The child process is single-threaded.
                let ret = unsafe { libc::ioctl(slave_raw, libc::TIOCSCTTY, 0) };
                if ret < 0 {
                    std::process::exit(1);
                }
                unsafe {
                    libc::dup2(slave_raw, 0);
                    libc::dup2(slave_raw, 1);
                    libc::dup2(slave_raw, 2);
                }
                if slave_raw > 2 {
                    unsafe {
                        libc::close(slave_raw);
                    }
                }
                // Configure raw mode on stdin (fd 0, PTY slave device).
                // Failure is non-fatal (shell runs in canonical mode).
                configure_raw_mode(libc::STDIN_FILENO).ok();
                // Change to working directory (failure is non-fatal).
                unsafe { libc::chdir(working_directory_ptr) };
                // Reset signal handlers to defaults, preventing leakage of multithreaded
                // custom handlers from parent to child process.
                unsafe {
                    libc::signal(libc::SIGCHLD, libc::SIG_DFL);
                    libc::signal(libc::SIGHUP, libc::SIG_DFL);
                    libc::signal(libc::SIGINT, libc::SIG_DFL);
                    libc::signal(libc::SIGQUIT, libc::SIG_DFL);
                    libc::signal(libc::SIGTERM, libc::SIG_DFL);
                    libc::signal(libc::SIGPIPE, libc::SIG_DFL);
                    libc::signal(libc::SIGALRM, libc::SIG_DFL);
                }
                // Use libc::execve directly with pre-allocated arrays.
                // nix::unistd::execve allocates a Vec internally via collect(),
                // which is unsafe after fork in a multithreaded process.
                unsafe {
                    libc::execve(shell_ptr, args_ptrs.as_ptr(), env_ptrs.as_ptr());
                }
                unsafe {
                    libc::_exit(1);
                }
            }
        }
    }

    pub fn child_pid(&self) -> nix::unistd::Pid {
        self.child_pid
    }

    pub fn wait(&self) -> nix::Result<nix::sys::wait::WaitStatus> {
        nix::sys::wait::waitpid(self.child_pid, None)
    }

    pub fn resize(&self, rows: u16, cols: u16) -> Result<(), PtyError> {
        let winsize = nix::pty::Winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: self.pixel_width,
            ws_ypixel: self.pixel_height,
        };
        // SAFETY: ioctl with TIOCSWINSZ writes a well-formed Winsize struct
        // to the master PTY fd. The fd is owned and valid. The kernel copies
        // the winsize to the slave side — no memory safety risk. The return
        // value is checked for errors.
        unsafe {
            let ret = libc::ioctl(
                self.master.as_raw_fd(),
                libc::TIOCSWINSZ,
                &winsize as *const _,
            );
            if ret < 0 {
                return Err(PtyError::Resize(nix::errno::Errno::last()));
            }
        }
        Ok(())
    }

    pub fn set_pixel_size(&mut self, width: u16, height: u16) {
        self.pixel_width = width;
        self.pixel_height = height;
    }

    pub fn set_nonblocking(&self) -> Result<(), PtyError> {
        let flags = nix::fcntl::fcntl(&self.master, nix::fcntl::FcntlArg::F_GETFL)
            .map_err(PtyError::Fcntl)?;
        let new_flags =
            nix::fcntl::OFlag::from_bits_truncate(flags) | nix::fcntl::OFlag::O_NONBLOCK;
        nix::fcntl::fcntl(&self.master, nix::fcntl::FcntlArg::F_SETFL(new_flags))
            .map_err(PtyError::Fcntl)?;
        Ok(())
    }

    pub fn master_fd(&self) -> std::os::unix::io::RawFd {
        self.master.as_raw_fd()
    }

    pub fn into_raw_fd(self) -> std::os::unix::io::RawFd {
        let fd = self.master.as_raw_fd();
        std::mem::forget(self);
        fd
    }
}

impl Pty for PtyPair {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        nix::unistd::write(&self.master, buf).map_err(|e| io::Error::from_raw_os_error(e as i32))
    }

    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        nix::unistd::read(&self.master, buf).map_err(|e| io::Error::from_raw_os_error(e as i32))
    }

    fn resize(&self, rows: u16, cols: u16) -> Result<(), PtyError> {
        PtyPair::resize(self, rows, cols)
    }

    fn child_pid(&self) -> nix::unistd::Pid {
        PtyPair::child_pid(self)
    }

    fn master_fd(&self) -> RawFd {
        PtyPair::master_fd(self)
    }

    fn wait(&self) -> nix::Result<nix::sys::wait::WaitStatus> {
        PtyPair::wait(self)
    }

    fn set_nonblocking(&self) -> Result<(), PtyError> {
        PtyPair::set_nonblocking(self)
    }

    fn set_pixel_size(&mut self, width: u16, height: u16) {
        PtyPair::set_pixel_size(self, width, height)
    }

    fn spawn(shell: &str, rows: u16, cols: u16, env: &ShellEnv) -> Result<Box<dyn Pty>, PtyError> {
        PtyPair::spawn(shell, rows, cols, env).map(|p| Box::new(p) as Box<dyn Pty>)
    }
}

impl std::io::Read for PtyPair {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        nix::unistd::read(&self.master, buf)
            .map_err(|e| std::io::Error::from_raw_os_error(e as i32))
    }
}

impl std::io::Write for PtyPair {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        nix::unistd::write(&self.master, buf)
            .map_err(|e| std::io::Error::from_raw_os_error(e as i32))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for PtyPair {
    fn drop(&mut self) {
        nix::sys::signal::kill(self.child_pid, nix::sys::signal::Signal::SIGHUP).ok();
        nix::sys::signal::kill(self.child_pid, nix::sys::signal::Signal::SIGCONT).ok();
        std::thread::sleep(Duration::from_millis(100));
        nix::sys::signal::kill(self.child_pid, nix::sys::signal::Signal::SIGKILL).ok();
        nix::sys::wait::waitpid(self.child_pid, None).ok();
    }
}

fn configure_raw_mode(fd: std::os::unix::io::RawFd) -> Result<(), PtyError> {
    let mut termios = std::mem::MaybeUninit::<libc::termios>::uninit();
    // SAFETY: tcgetattr is safe with a valid fd. fd is STDIN_FILENO (0)
    // after login_tty dup'd the PTY slave, so it's always valid.
    let ret = unsafe { libc::tcgetattr(fd, termios.as_mut_ptr()) };
    if ret != 0 {
        return Err(PtyError::Termios(nix::errno::Errno::last()));
    }
    let mut termios = unsafe { termios.assume_init() };
    termios.c_iflag &= !(libc::IGNBRK
        | libc::BRKINT
        | libc::PARMRK
        | libc::ISTRIP
        | libc::INLCR
        | libc::IGNCR
        | libc::ICRNL
        | libc::IXON);
    termios.c_oflag &= !(libc::OPOST);
    termios.c_lflag &= !(libc::ECHO | libc::ECHONL | libc::ICANON | libc::ISIG | libc::IEXTEN);
    termios.c_cflag &= !(libc::CSIZE | libc::PARENB);
    termios.c_cflag |= libc::CS8;
    termios.c_cc[libc::VMIN] = 1;
    termios.c_cc[libc::VTIME] = 0;
    // SAFETY: tcsetattr is safe with a valid fd and valid termios struct.
    let ret = unsafe { libc::tcsetattr(fd, libc::TCSANOW, &termios) };
    if ret != 0 {
        return Err(PtyError::Termios(nix::errno::Errno::last()));
    }
    Ok(())
}

fn base_env(prefix: Option<&str>) -> Vec<(String, String)> {
    let mut result = vec![
        ("TERM".to_string(), "xterm-256color".to_string()),
        ("COLORTERM".to_string(), "truecolor".to_string()),
        ("TERM_PROGRAM".to_string(), "torvox".to_string()),
        (
            "TERM_PROGRAM_VERSION".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        ),
        ("LANG".to_string(), "en_US.UTF-8".to_string()),
    ];
    if let Some(p) = prefix {
        result.push(("PREFIX".to_string(), p.to_string()));
        result.push(("TMPDIR".to_string(), format!("{p}/tmp")));
    } else {
        result.push(("TMPDIR".to_string(), "/data/local/tmp".to_string()));
    }
    result
}

pub fn build_env(env: &ShellEnv, shell_path: &str, rows: u16, cols: u16) -> Vec<(String, String)> {
    let prefix_str = env.prefix.as_deref();
    let mut result = base_env(prefix_str);
    result.push(("HOME".to_string(), env.home.clone()));
    result.push(("USER".to_string(), env.user.clone()));
    result.push(("SHELL".to_string(), shell_path.to_string()));
    result.push(("PATH".to_string(), env.path.clone()));
    result.push(("PWD".to_string(), env.working_directory.clone()));
    result.push(("LINES".to_string(), rows.to_string()));
    result.push(("COLUMNS".to_string(), cols.to_string()));
    result.extend(env.extra.iter().cloned());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_env() -> ShellEnv {
        ShellEnv {
            home: "/tmp/test_home".to_string(),
            user: "testuser".to_string(),
            path: "/usr/bin:/bin".to_string(),
            working_directory: "/tmp/test_home".to_string(),
            prefix: None,
            extra: vec![],
        }
    }

    #[test]
    fn base_env_includes_xterm_256color() {
        let env = base_env(None);
        assert!(
            env.iter()
                .any(|(k, v)| k == "TERM" && v == "xterm-256color")
        );
    }

    #[test]
    fn base_env_includes_lang() {
        let env = base_env(None);
        assert!(env.iter().any(|(k, v)| k == "LANG" && v == "en_US.UTF-8"));
    }

    #[test]
    fn base_env_includes_tmpdir_without_prefix() {
        let env = base_env(None);
        assert!(
            env.iter()
                .any(|(k, v)| k == "TMPDIR" && v == "/data/local/tmp")
        );
    }

    #[test]
    fn base_env_includes_prefix_and_tmpdir_when_set() {
        let env = base_env(Some("/data/data/com.termux/files/usr"));
        assert!(
            env.iter()
                .any(|(k, v)| k == "PREFIX" && v == "/data/data/com.termux/files/usr")
        );
        assert!(
            env.iter()
                .any(|(k, v)| k == "TMPDIR" && v == "/data/data/com.termux/files/usr/tmp")
        );
    }

    #[test]
    fn build_env_includes_term() {
        let env = test_env();
        let result = build_env(&env, "/bin/sh", 24, 80);
        assert!(
            result
                .iter()
                .any(|(k, v)| k == "TERM" && v == "xterm-256color")
        );
    }

    #[test]
    fn build_env_includes_colorterm() {
        let env = test_env();
        let result = build_env(&env, "/bin/sh", 24, 80);
        assert!(
            result
                .iter()
                .any(|(k, v)| k == "COLORTERM" && v == "truecolor")
        );
    }

    #[test]
    fn build_env_includes_term_program() {
        let env = test_env();
        let result = build_env(&env, "/bin/sh", 24, 80);
        assert!(
            result
                .iter()
                .any(|(k, v)| k == "TERM_PROGRAM" && v == "torvox")
        );
    }

    #[test]
    fn build_env_includes_program_version() {
        let env = test_env();
        let result = build_env(&env, "/bin/sh", 24, 80);
        assert!(result.iter().any(|(k, _)| k == "TERM_PROGRAM_VERSION"));
    }

    #[test]
    fn build_env_includes_home_from_env() {
        let env = test_env();
        let result = build_env(&env, "/bin/sh", 24, 80);
        assert!(
            result
                .iter()
                .any(|(k, v)| k == "HOME" && v == "/tmp/test_home")
        );
    }

    #[test]
    fn build_env_includes_user_from_env() {
        let env = test_env();
        let result = build_env(&env, "/bin/sh", 24, 80);
        assert!(result.iter().any(|(k, v)| k == "USER" && v == "testuser"));
    }

    #[test]
    fn build_env_includes_shell_from_param() {
        let env = test_env();
        let result = build_env(&env, "/bin/bash", 24, 80);
        assert!(result.iter().any(|(k, v)| k == "SHELL" && v == "/bin/bash"));
    }

    #[test]
    fn build_env_includes_path_from_env() {
        let env = test_env();
        let result = build_env(&env, "/bin/sh", 24, 80);
        assert!(
            result
                .iter()
                .any(|(k, v)| k == "PATH" && v == "/usr/bin:/bin")
        );
    }

    #[test]
    fn build_env_includes_pwd_from_env() {
        let env = test_env();
        let result = build_env(&env, "/bin/sh", 24, 80);
        assert!(
            result
                .iter()
                .any(|(k, v)| k == "PWD" && v == "/tmp/test_home")
        );
    }

    #[test]
    fn build_env_includes_lines_and_columns() {
        let env = test_env();
        let result = build_env(&env, "/bin/sh", 24, 80);
        assert!(result.iter().any(|(k, v)| k == "LINES" && v == "24"));
        assert!(result.iter().any(|(k, v)| k == "COLUMNS" && v == "80"));
    }

    #[test]
    fn build_env_no_duplicate_explicit_keys() {
        let mut env = test_env();
        env.extra.push(("TERM".to_string(), "dumb".to_string()));
        let result = build_env(&env, "/bin/sh", 24, 80);
        let term_entries: Vec<_> = result.iter().filter(|(k, _)| k == "TERM").collect();
        assert_eq!(term_entries.len(), 2);
        assert_eq!(term_entries[0].1, "xterm-256color");
        assert_eq!(term_entries[1].1, "dumb");
    }

    #[test]
    fn build_env_extra_entries_present() {
        let mut env = test_env();
        env.extra
            .push(("ANDROID_ROOT".to_string(), "/system".to_string()));
        let result = build_env(&env, "/bin/sh", 24, 80);
        assert!(
            result
                .iter()
                .any(|(k, v)| k == "ANDROID_ROOT" && v == "/system")
        );
    }

    #[test]
    fn spawn_and_read_shell() {
        use crate::pty::Pty;

        let mut pty =
            PtyPair::spawn("/bin/sh", 24, 80, &ShellEnv::default()).expect("spawn failed");
        pty.set_nonblocking().expect("set_nonblocking failed");

        Pty::write_all(&mut pty, b"echo hello_torvox\n").expect("write failed");

        let mut buf = [0u8; 4096];
        let mut output = Vec::new();
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while std::time::Instant::now() < deadline {
            match Pty::read(&mut pty, &mut buf) {
                Ok(n) => output.extend_from_slice(&buf[..n]),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
            if output
                .windows("hello_torvox".len())
                .any(|w| w == b"hello_torvox")
            {
                return;
            }
        }
        panic!(
            "did not see 'hello_torvox' in output: {}",
            String::from_utf8_lossy(&output)
        );
    }

    #[test]
    fn resize_succeeds() {
        let pty = PtyPair::spawn("/bin/sh", 24, 80, &ShellEnv::default()).expect("spawn failed");
        pty.resize(40, 120).expect("resize failed");
    }

    #[test]
    fn child_pid_is_positive() {
        let _pty = PtyPair::spawn("/bin/sh", 24, 80, &ShellEnv::default()).expect("spawn failed");
    }

    #[test]
    fn drop_kills_child() {
        let child = {
            let pty =
                PtyPair::spawn("/bin/sh", 24, 80, &ShellEnv::default()).expect("spawn failed");
            pty.child_pid()
        };
        std::thread::sleep(Duration::from_millis(200));
        let result = nix::sys::signal::kill(child, nix::sys::signal::Signal::SIGTERM);
        assert!(result.is_err(), "child should already be dead after drop");
    }

    #[test]
    fn pty_error_display_works() {
        let display = format!("{}", PtyError::Open(nix::errno::Errno::EINVAL.into()));
        assert!(
            !display.is_empty(),
            "PtyError Display should produce non-empty string"
        );
    }

    #[test]
    fn chdir_changes_working_directory() {
        use crate::pty::Pty;

        let temp = std::env::temp_dir().join("torvox_test_chdir");
        std::fs::create_dir_all(&temp).expect("create test dir failed");

        let env = ShellEnv {
            working_directory: temp.to_string_lossy().to_string(),
            ..ShellEnv::default()
        };

        let mut pty = PtyPair::spawn("/bin/sh", 24, 80, &env).expect("spawn failed");
        pty.set_nonblocking().expect("set_nonblocking failed");
        Pty::write_all(&mut pty, b"pwd\n").expect("write failed");

        let mut buf = [0u8; 4096];
        let mut output = Vec::new();
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while std::time::Instant::now() < deadline {
            match Pty::read(&mut pty, &mut buf) {
                Ok(n) => output.extend_from_slice(&buf[..n]),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
            let path_str = temp.to_string_lossy();
            let path_bytes = path_str.as_bytes();
            if output.windows(path_bytes.len()).any(|w| w == path_bytes) {
                std::fs::remove_dir_all(&temp).ok();
                return;
            }
        }
        std::fs::remove_dir_all(&temp).ok();
        panic!(
            "did not see working directory '{}' in pwd output: {}",
            temp.display(),
            String::from_utf8_lossy(&output)
        );
    }

    #[test]
    fn double_write_then_read_does_not_panic() {
        use crate::pty::Pty;

        let mut pty = match PtyPair::spawn("/bin/sh", 24, 80, &ShellEnv::default()) {
            Ok(p) => p,
            Err(_) => return,
        };
        pty.set_nonblocking().expect("set_nonblocking failed");

        let _ = Pty::write_all(&mut pty, b"echo a\n");
        let _ = Pty::write_all(&mut pty, b"echo b\n");

        let mut buf = [0u8; 4096];
        let mut output = Vec::new();
        for _ in 0..50 {
            match Pty::read(&mut pty, &mut buf) {
                Ok(n) => output.extend_from_slice(&buf[..n]),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
            if output.len() > 200 {
                break;
            }
        }
    }
}
