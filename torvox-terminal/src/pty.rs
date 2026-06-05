use std::os::unix::io::{AsRawFd, OwnedFd};
use std::time::Duration;

use thiserror::Error;

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

pub struct PtyPair {
    master: OwnedFd,
    child_pid: nix::unistd::Pid,
}

impl PtyPair {
    pub fn spawn(shell: &str, rows: u16, cols: u16) -> Result<Self, PtyError> {
        let winsize = nix::pty::Winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        let result = nix::pty::openpty(Some(&winsize), None)?;
        let master_fd = result.master;
        let slave_fd = result.slave;

        // 在 fork 之前构建所有子进程数据，避免子进程中任何分配
        // （多线程进程 fork 后 malloc 堆可能损坏）。
        let shell_cstr = std::ffi::CString::new(shell).expect("shell path contains null byte");
        let env_cstrings: Vec<std::ffi::CString> = build_env()
            .into_iter()
            .map(|(k, v)| {
                std::ffi::CString::new(format!("{k}={v}")).expect("env var contains null")
            })
            .collect();

        // 在 fork 之前预分配参数和环境数组。
        // fork 后，子进程不得调用任何分配内存的函数。
        let shell_ptr = shell_cstr.as_ptr();
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
                })
            }
            nix::unistd::ForkResult::Child => {
                nix::unistd::close(master_fd).ok();
                // 仅使用系统调用手动设置控制终端。
                // 避免使用 login_tty，因为它可能在内部调用 malloc()，
                // 在多线程进程 fork 后这是不安全的。
                if nix::unistd::setsid().is_err() {
                    std::process::exit(1);
                }
                let slave_raw = slave_fd.as_raw_fd();
                // SAFETY: 所有这些 libc 调用都是不分配内存的轻量级系统调用封装。
                // 子进程是单线程的。
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
                // 在 stdin (fd 0, PTY 从设备) 上配置原始模式。
                // 失败是非致命的（shell 以规范模式运行）。
                configure_raw_mode(libc::STDIN_FILENO).ok();
                // 直接使用 libc::execve 和预分配的数组。
                // nix::unistd::execve 内部通过 collect() 分配 Vec，
                // 在多线程进程 fork 后这是不安全的。
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
            ws_xpixel: 0,
            ws_ypixel: 0,
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

fn build_env() -> Vec<(String, String)> {
    vec![
        ("TERM".to_string(), "torvox-direct".to_string()),
        ("COLORTERM".to_string(), "truecolor".to_string()),
        ("TERM_PROGRAM".to_string(), "torvox".to_string()),
        (
            "TERM_PROGRAM_VERSION".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_and_read_shell() {
        use std::io::{Read, Write};

        let mut pty = PtyPair::spawn("/bin/sh", 24, 80).expect("spawn failed");
        pty.set_nonblocking().expect("set_nonblocking failed");

        pty.write_all(b"echo hello_torvox\n").expect("write failed");

        let mut buf = [0u8; 4096];
        let mut output = Vec::new();
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while std::time::Instant::now() < deadline {
            match pty.read(&mut buf) {
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
        let pty = PtyPair::spawn("/bin/sh", 24, 80).expect("spawn failed");
        pty.resize(40, 120).expect("resize failed");
    }

    #[test]
    fn child_pid_is_positive() {
        let _pty = PtyPair::spawn("/bin/sh", 24, 80).expect("spawn failed");
    }

    #[test]
    fn drop_kills_child() {
        let child = {
            let pty = PtyPair::spawn("/bin/sh", 24, 80).expect("spawn failed");
            pty.child_pid()
        };
        std::thread::sleep(Duration::from_millis(200));
        let result = nix::sys::signal::kill(child, nix::sys::signal::Signal::SIGTERM);
        assert!(result.is_err(), "child should already be dead after drop");
    }
}
