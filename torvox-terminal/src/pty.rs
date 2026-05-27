use std::os::unix::io::{AsRawFd, OwnedFd};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PtyError {
    #[error("fork failed: {0}")]
    Fork(#[from] nix::errno::Errno),
    #[error("failed to open pseudoterminal: {0}")]
    Open(std::io::Error),
    #[error("ioctl TIOCSWINSZ failed: {0}")]
    Resize(nix::errno::Errno),
}

pub struct Pty {
    fd: OwnedFd,
    child_pid: nix::unistd::Pid,
}

impl Pty {
    pub fn open(rows: u16, cols: u16, command: &str, args: &[&str]) -> Result<Self, PtyError> {
        let winsize = nix::pty::Winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        let result = nix::pty::openpty(Some(&winsize), None)?;
        let fd = result.master;
        let slave_fd = result.slave;

        match unsafe { nix::unistd::fork()? } {
            nix::unistd::ForkResult::Parent { child } => {
                nix::unistd::close(slave_fd).ok();
                Ok(Self {
                    fd,
                    child_pid: child,
                })
            }
            nix::unistd::ForkResult::Child => {
                nix::unistd::close(fd).ok();
                nix::unistd::setsid().ok();
                let ret = unsafe { libc::login_tty(slave_fd.as_raw_fd()) };
                if ret != 0 {
                    std::process::exit(1);
                }
                nix::unistd::execvp(
                    std::ffi::CString::new(command).unwrap().as_c_str(),
                    &args
                        .iter()
                        .map(|a| std::ffi::CString::new(*a).unwrap())
                        .collect::<Vec<_>>(),
                )
                .unwrap_err();
                std::process::exit(1);
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
        unsafe {
            let ret = libc::ioctl(self.fd.as_raw_fd(), libc::TIOCSWINSZ, &winsize as *const _);
            if ret < 0 {
                return Err(PtyError::Resize(nix::errno::Errno::last()));
            }
        }
        Ok(())
    }
}

impl std::io::Read for Pty {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        nix::unistd::read(&self.fd, buf).map_err(|e| std::io::Error::from_raw_os_error(e as i32))
    }
}

impl std::io::Write for Pty {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        nix::unistd::write(&self.fd, buf).map_err(|e| std::io::Error::from_raw_os_error(e as i32))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for Pty {
    fn drop(&mut self) {
        nix::sys::signal::kill(self.child_pid, nix::sys::signal::Signal::SIGHUP).ok();
    }
}
