use axerrno::{AxError, AxResult};
use axfs::api::port::{FileExt, FileIO, FileIOType, OpenFlags};
use axhal::console::{getchar, write_bytes};
use axio::{Read, Seek, SeekFrom, Write};
use spinlock::SpinNoIrq;
use axlog::error;
use axsync::Mutex;
use axtask::yield_now;
/// stdin file for getting chars from console
pub struct Stdin {
    pub flags: Mutex<OpenFlags>,
}

/// stdout file for putting chars to console
pub struct Stdout {
    pub flags: Mutex<OpenFlags>,
}

/// stderr file for putting chars to console
pub struct Stderr {
    pub flags: Mutex<OpenFlags>,
}

const BUFFER_SIZE: usize = 128;
struct StdioRingBuffer {
    buffer: [u8; BUFFER_SIZE],
    head: usize,
    tail: usize,
    empty: bool,
    get_enter: bool,
}

impl StdioRingBuffer {
    const fn new() -> Self {
        StdioRingBuffer {
            buffer: [0_u8; BUFFER_SIZE],
            head: 0_usize,
            tail: 0_usize,
            empty: true,
            get_enter: false,
        }
    }

    fn push(&mut self, n: u8) {
        if self.tail != self.head || self.empty {
            self.buffer[self.tail] = n;
            self.tail = (self.tail + 1) % BUFFER_SIZE;
            self.empty = false;
            self.get_enter = n == b'\n' || n == 9;
        }
    }

    fn pop(&mut self) -> Option<u8> {
        if self.empty {
            None
        } else {
            let ret = self.buffer[self.head];
            self.head = (self.head + 1) % BUFFER_SIZE;
            if self.head == self.tail {
                self.empty = true;
                self.get_enter = false;
            }
            Some(ret)
        }
    }

    fn is_release(&self) -> bool {
        !self.empty && self.get_enter
    }
}

struct StdioDrv {
    buffer: SpinNoIrq<StdioRingBuffer>,
}

static AX_STDIO: StdioDrv = StdioDrv {
    buffer: SpinNoIrq::new(StdioRingBuffer::new()),
};

fn console_read_bytes() -> Option<u8> {
    let ret = getchar().map(|c| if c == b'\r' { b'\n' } else { c });
    ret
}

fn console_write_bytes(buf: &[u8]) -> AxResult<usize> {
    write_bytes(buf);
    Ok(buf.len())
}

fn stdin_read(buf: &mut [u8]) -> AxResult<usize> {
    let read_len = 0;
    while read_len < buf.len() {
        if let Some(c) = console_read_bytes() {
            AX_STDIO.buffer.lock().push(c);
        } else {
            break;
        }
    }
    Ok(read_len)
}

fn stdout_write(buf: &[u8]) -> AxResult<usize> {
    write_bytes(buf);
    Ok(buf.len())
}

impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> AxResult<usize> {
        stdin_read(buf)
    }
}

impl Write for Stdin {
    fn write(&mut self, _: &[u8]) -> AxResult<usize> {
        panic!("Cannot write to stdin!");
    }
    fn flush(&mut self) -> axio::Result {
        panic!("Flushing stdin")
    }
}

impl Seek for Stdin {
    fn seek(&mut self, _pos: SeekFrom) -> AxResult<u64> {
        Err(AxError::Unsupported) // 如果没有实现seek, 则返回Unsupported
    }
}

impl FileExt for Stdin {
    fn executable(&self) -> bool {
        false
    }
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        false
    }
}

impl FileIO for Stdin {
    fn read(&self, buf: &mut [u8]) -> AxResult<usize> {
        let ret = console_read_bytes();
        if let Some(c) = ret {
            buf[0] = c;
            return Ok(1);
        }
        loop {
            let ret = console_read_bytes();
            if let Some(c) = ret {
                AX_STDIO.buffer.lock().push(c);
                console_write_bytes(&[c]);
            }
            if AX_STDIO.buffer.lock().is_release() {
                buf[0] = AX_STDIO.buffer.lock().pop().unwrap();
                return Ok(1);
            }
            yield_now();
        }
    }

    fn get_type(&self) -> FileIOType {
        FileIOType::Stdin
    }

    fn ready_to_read(&self) -> bool {
        true
    }

    fn ready_to_write(&self) -> bool {
        false
    }

    fn readable(&self) -> bool {
        true
    }

    fn writable(&self) -> bool {
        false
    }

    fn executable(&self) -> bool {
        false
    }

    fn set_status(&self, flags: OpenFlags) -> bool {
        if flags.contains(OpenFlags::CLOEXEC) {
            *self.flags.lock() = OpenFlags::CLOEXEC;
            true
        } else {
            false
        }
    }

    fn get_status(&self) -> OpenFlags {
        *self.flags.lock()
    }

    fn set_close_on_exec(&self, is_set: bool) -> bool {
        if is_set {
            // 设置close_on_exec位置
            *self.flags.lock() |= OpenFlags::CLOEXEC;
        } else {
            *self.flags.lock() &= !OpenFlags::CLOEXEC;
        }
        true
    }
}

impl Read for Stdout {
    fn read(&mut self, _: &mut [u8]) -> AxResult<usize> {
        panic!("Cannot read from stdin!");
    }
}

impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> AxResult<usize> {
        error!("write to stdout: {:?}", buf);
        write_bytes(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> AxResult {
        // stdout is always flushed
        Ok(())
    }
}

impl Seek for Stdout {
    fn seek(&mut self, _pos: SeekFrom) -> AxResult<u64> {
        Err(AxError::Unsupported) // 如果没有实现seek, 则返回Unsupported
    }
}

impl FileExt for Stdout {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }
    fn executable(&self) -> bool {
        false
    }
}
impl FileIO for Stdout {
    fn write(&self, buf: &[u8]) -> AxResult<usize> {
        stdout_write(buf)
    }

    fn flush(&self) -> AxResult {
        // stdout is always flushed
        Ok(())
    }

    fn readable(&self) -> bool {
        false
    }

    fn writable(&self) -> bool {
        true
    }

    fn executable(&self) -> bool {
        false
    }

    fn get_type(&self) -> FileIOType {
        FileIOType::Stdout
    }

    fn ready_to_read(&self) -> bool {
        false
    }

    fn ready_to_write(&self) -> bool {
        true
    }

    fn set_status(&self, flags: OpenFlags) -> bool {
        if flags.contains(OpenFlags::CLOEXEC) {
            *self.flags.lock() = flags;
            true
        } else {
            false
        }
    }

    fn get_status(&self) -> OpenFlags {
        *self.flags.lock()
    }

    fn set_close_on_exec(&self, is_set: bool) -> bool {
        if is_set {
            // 设置close_on_exec位置
            *self.flags.lock() |= OpenFlags::CLOEXEC;
        } else {
            *self.flags.lock() &= !OpenFlags::CLOEXEC;
        }
        true
    }
}

impl Read for Stderr {
    fn read(&mut self, _: &mut [u8]) -> AxResult<usize> {
        panic!("Cannot read from stdout!");
    }
}

impl Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> AxResult<usize> {
        write_bytes(buf);
        Ok(buf.len())
    }

    /// Stderr is always flushed
    fn flush(&mut self) -> axio::Result {
        Ok(())
    }
}

impl Seek for Stderr {
    fn seek(&mut self, _pos: SeekFrom) -> AxResult<u64> {
        Err(AxError::Unsupported) // 如果没有实现seek, 则返回Unsupported
    }
}

impl FileExt for Stderr {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }
    fn executable(&self) -> bool {
        false
    }
}

impl FileIO for Stderr {
    fn write(&self, buf: &[u8]) -> AxResult<usize> {
        write_bytes(buf);
        Ok(buf.len())
    }

    /// Stderr is always flushed
    fn flush(&self) -> axio::Result {
        Ok(())
    }

    fn readable(&self) -> bool {
        false
    }

    fn writable(&self) -> bool {
        true
    }

    fn executable(&self) -> bool {
        false
    }

    fn get_type(&self) -> FileIOType {
        FileIOType::Stderr
    }

    fn ready_to_read(&self) -> bool {
        false
    }

    fn ready_to_write(&self) -> bool {
        true
    }
}
