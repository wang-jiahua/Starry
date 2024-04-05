use axsync::{Mutex, MutexGuard};

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

use axio::{Read, Write, BufRead, BufReader, Result};
use axtask::yield_now;
use axerrno::AxResult;

#[cfg(feature = "fd")]
use {alloc::sync::Arc, axerrno::LinuxError, axerrno::LinuxResult, axio::PollState};

use crate::syscall_fs::ctypes::stat;
use crate::syscall_fs::imp::fd_ops::sys_sched_yield;

struct StdinRaw;
struct StdoutRaw;

impl Read for StdinRaw {
    // Non-blocking read, returns number of bytes read.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut read_len = 0;
        while read_len < buf.len() {
            if let Some(c) = arceos_api::stdio::ax_console_read_byte() {
                buf[read_len] = c;
                read_len += 1;
            } else {
                break;
            }
        }
        Ok(read_len)
    }
}

impl Write for StdoutRaw {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        arceos_api::stdio::ax_console_write_bytes(buf)
    }
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

/// A handle to the standard input stream of a process.
pub struct Stdin {
    inner: &'static Mutex<BufReader<StdinRaw>>,
}

/// A locked reference to the [`Stdin`] handle.
pub struct StdinLock<'a> {
    inner: MutexGuard<'a, BufReader<StdinRaw>>,
}

impl Stdin {
    /// Locks this handle to the standard input stream, returning a readable
    /// guard.
    ///
    /// The lock is released when the returned lock goes out of scope. The
    /// returned guard also implements the [`Read`] and [`BufRead`] traits for
    /// accessing the underlying data.
    pub fn lock(&self) -> StdinLock<'static> {
        // Locks this handle with 'static lifetime. This depends on the
        // implementation detail that the underlying `Mutex` is static.
        StdinLock {
            inner: self.inner.lock(),
        }
    }

    /// Locks this handle and reads a line of input, appending it to the specified buffer.
    #[cfg(feature = "alloc")]
    pub fn read_line(&self, buf: &mut String) -> io::Result<usize> {
        self.inner.lock().read_line(buf)
    }
}

impl Stdin {
    // Block until at least one byte is read.
    fn read_blocked(&self, buf: &mut [u8]) -> AxResult<usize> {
        let read_len = self.inner.lock().read(buf)?;
        if buf.is_empty() || read_len > 0 {
            return Ok(read_len);
        }
        // try again until we get something
        loop {
            let read_len = self.inner.lock().read(buf)?;
            if read_len > 0 {
                return Ok(read_len);
            }
            sys_sched_yield();
        }
    }
}

impl Read for Stdin {
    // Block until at least one byte is read.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let read_len = self.inner.lock().read(buf)?;
        if buf.is_empty() || read_len > 0 {
            return Ok(read_len);
        }
        // try again until we got something
        loop {
            let read_len = self.inner.lock().read(buf)?;
            if read_len > 0 {
                return Ok(read_len);
            }
            yield_now();
        }
    }
}

impl Read for StdinLock<'_> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read(buf)
    }
}

impl BufRead for StdinLock<'_> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        self.inner.fill_buf()
    }

    fn consume(&mut self, n: usize) {
        self.inner.consume(n)
    }

    #[cfg(feature = "alloc")]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> Result<usize> {
        self.inner.read_until(byte, buf)
    }

    #[cfg(feature = "alloc")]
    fn read_line(&mut self, buf: &mut String) -> Result<usize> {
        self.inner.read_line(buf)
    }
}

/// A handle to the global standard output stream of the current process.
pub struct Stdout {
    inner: &'static Mutex<StdoutRaw>,
}

/// A locked reference to the [`Stdout`] handle.
pub struct StdoutLock<'a> {
    inner: MutexGuard<'a, StdoutRaw>,
}

impl Stdout {
    /// Locks this handle to the standard output stream, returning a writable
    /// guard.
    ///
    /// The lock is released when the returned lock goes out of scope. The
    /// returned guard also implements the `Write` trait for writing data.
    pub fn lock(&self) -> StdoutLock<'static> {
        StdoutLock {
            inner: self.inner.lock(),
        }
    }
}

impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.inner.lock().write(buf)
    }
    fn flush(&mut self) -> Result<()> {
        self.inner.lock().flush()
    }
}

impl Write for StdoutLock<'_> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}

/// Constructs a new handle to the standard input of the current process.
pub fn stdin() -> Stdin {
    static INSTANCE: Mutex<BufReader<StdinRaw>> = Mutex::new(BufReader::new(StdinRaw));
    Stdin { inner: &INSTANCE }
}

/// Constructs a new handle to the standard output of the current process.
pub fn stdout() -> Stdout {
    static INSTANCE: Mutex<StdoutRaw> = Mutex::new(StdoutRaw);
    Stdout { inner: &INSTANCE }
}

#[doc(hidden)]
pub fn __print_impl(args: core::fmt::Arguments) {
    if cfg!(feature = "smp") {
        // synchronize using the lock in axlog, to avoid interleaving
        // with kernel logs
        arceos_api::stdio::ax_console_write_fmt(args).unwrap();
    } else {
        stdout().lock().write_fmt(args).unwrap();
    }
}


#[cfg(feature = "fd")]
impl super::fd_ops::FileLike for Stdin {
    fn read(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        Ok(self.read_blocked(buf)?)
    }

    fn write(&self, _buf: &[u8]) -> LinuxResult<usize> {
        Err(LinuxError::EPERM)
    }

    fn stat(&self) -> LinuxResult<stat> {
        let st_mode = 0o20000 | 0o440u32; // S_IFCHR | r--r-----
        Ok(stat {
            st_ino: 1,
            st_nlink: 1,
            st_mode,
            ..Default::default()
        })
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        Ok(PollState {
            readable: true,
            writable: true,
        })
    }

    fn set_nonblocking(&self, _nonblocking: bool) -> LinuxResult {
        Ok(())
    }
}

#[cfg(feature = "fd")]
impl super::fd_ops::FileLike for Stdout {
    fn read(&self, _buf: &mut [u8]) -> LinuxResult<usize> {
        Err(LinuxError::EPERM)
    }

    fn write(&self, buf: &[u8]) -> LinuxResult<usize> {
        Ok(self.inner.lock().write(buf)?)
    }

    fn stat(&self) -> LinuxResult<stat> {
        let st_mode = 0o20000 | 0o220u32; // S_IFCHR | -w--w----
        Ok(stat {
            st_ino: 1,
            st_nlink: 1,
            st_mode,
            ..Default::default()
        })
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        Ok(PollState {
            readable: true,
            writable: true,
        })
    }

    fn set_nonblocking(&self, _nonblocking: bool) -> LinuxResult {
        Ok(())
    }
}