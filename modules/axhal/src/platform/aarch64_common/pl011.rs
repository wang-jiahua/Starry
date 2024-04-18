//! PL011 UART.

use arm_pl011::pl011::Pl011Uart;
use memory_addr::PhysAddr;
use spinlock::SpinNoIrq;
use cfg_if::cfg_if;

use crate::mem::phys_to_virt;

const UART_BASE: PhysAddr = PhysAddr::from(axconfig::UART_PADDR);

#[cfg(feature = "irq")]
const BUFFER_SIZE: usize = 128;

#[cfg(feature = "irq")]
struct AxRxRingBuffer {
    buffer: [u8; BUFFER_SIZE],
    head: usize,
    tail: usize,
    empty: bool,
}

#[cfg(feature = "irq")]
impl AxRxRingBuffer {
    const fn new() -> Self {
        AxRxRingBuffer {
            buffer: [0_u8; BUFFER_SIZE],
            head: 0_usize,
            tail: 0_usize,
            empty: true,
        }
    }

    fn push(&mut self, n: u8) {
        if self.tail != self.head || self.empty {
            self.buffer[self.tail] = n;
            self.tail = (self.tail + 1) % BUFFER_SIZE;
            self.empty = false;
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
            }
            Some(ret)
        }
    }
}

struct UartDrv {
    inner: SpinNoIrq<Pl011Uart>,
    #[cfg(feature = "irq")]
    buffer: SpinNoIrq<AxRxRingBuffer>,
}

static AxUart:UartDrv = UartDrv {
    inner: SpinNoIrq::new(Pl011Uart::new(phys_to_virt(UART_BASE).as_mut_ptr())),
    #[cfg(feature = "irq")]
    buffer: SpinNoIrq::new(AxRxRingBuffer::new()),
};

/// Writes a byte to the console.
pub fn putchar(c: u8) {
    let mut uart = AxUart.inner.lock();
    match c {
        b'\n' => {
            uart.putchar(b'\r');
            uart.putchar(b'\n');
        }
        127 => {
            uart.putchar(8);
            uart.putchar(b' ');
            uart.putchar(8);
        }
        c => uart.putchar(c),
    }
}

/// Reads a byte from the console, or returns [`None`] if no input is available.
pub fn getchar() -> Option<u8> {
    cfg_if! {
        if #[cfg(feature = "irq")] {
            AxUart.buffer.lock().pop()
        }else{
            AxUart.inner.lock().getchar()
        }
    }
}

/// Initialize the UART
pub fn init_early() {
    unsafe {
        crate::platform::aarch64_common::mem::idmap_device(UART_BASE.as_usize());
    }
    AxUart.inner.lock().init();
}

/// Set UART IRQ Enable
#[cfg(feature = "irq")]
pub fn init_irq() {
    crate::irq::register_handler_common(crate::platform::irq::UART_IRQ_NUM, handle);
}

/// UART IRQ Handler
pub fn handle() {
    let mut dev = AxUart.inner.lock();
    let is_receive_interrupt = dev.is_receive_interrupt();
    if is_receive_interrupt {
        dev.ack_interrupts();
        while let Some(c) = dev.getchar() {
            AxUart.buffer.lock().push(c);
        }
    }
}