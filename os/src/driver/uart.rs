use alloc::collections::VecDeque;

use crate::console::print_separator;
extern crate alloc;

struct Uart <const BASE : usize>;
struct CharBuffer(VecDeque<u8>);

const UART : Uart<0x10_000_000> = Uart{};
static mut READ_BUFFER  : CharBuffer = CharBuffer::new();
static mut WRITE_BUFFER : CharBuffer = CharBuffer::new();

/** Initialize the UART module. */
pub unsafe fn init() { UART.init(); }

/** Handle an uart trap */
#[no_mangle]
pub unsafe fn handle() {
    uart_try_read();
    uart_try_send();
}


/**
 * Synchornized putc.
 * Used for kernel debugging.
 */
pub unsafe fn sync_putc(c : u8) {
    while UART.can_write() == false {}
    return UART.putc(c);
}

/**
 * Try to put a char to the console.
 * The char will be sent when there's available time.
 */
pub unsafe fn putc(c : u8) {
    WRITE_BUFFER.push_char(c);
    uart_try_send();
}

unsafe fn uart_try_send() {
    while UART.can_write() {
        match WRITE_BUFFER.take_char() {
            None    => break,
            Some(c) => UART.putc(c),
        }
    }
}

unsafe fn uart_try_read() {
    while UART.can_read() {
        let c = UART.getc();
        READ_BUFFER.push_char(c);
        WRITE_BUFFER.push_char(c);
        uart_try_send();
    }
}

impl <const BASE : usize> Uart <BASE> {
    const IER : * mut u8 = (BASE as * mut u8).wrapping_add(0x1);
    const LCR : * mut u8 = (BASE as * mut u8).wrapping_add(0x3);
    const DLL : * mut u8 = (BASE as * mut u8).wrapping_add(0x0);
    const DLM : * mut u8 = (BASE as * mut u8).wrapping_add(0x1);
    const FCR : * mut u8 = (BASE as * mut u8).wrapping_add(0x2);
    const THR : * mut u8 = (BASE as * mut u8).wrapping_add(0x0);
    const LSR : * mut u8 = (BASE as * mut u8).wrapping_add(0x5);
    const RBR : * mut u8 = (BASE as * mut u8).wrapping_add(0x0);

    unsafe fn init(&self) {
        Self::IER.write_volatile(ier::DISABLE);
        Self::LCR.write_volatile(lcr::BAUD_LATCH);
        Self::DLL.write_volatile(dll::BPS_38400);
        Self::DLM.write_volatile(dlm::BPS_38400);
        Self::LCR.write_volatile(lcr::WORD_LEN_8);
        Self::FCR.write_volatile(fcr::ENABLE | fcr::CLEAR);
        Self::IER.write_volatile(ier::RX_ENABLE | ier::TX_ENABLE);

        logging!("UART initialization done!");
        print_separator();
    }

    unsafe fn can_write(&self) -> bool {
        (Self::LSR.read_volatile() & lsr::TX_IDLE) != 0
    }

    unsafe fn can_read(&self) -> bool {
        (Self::LSR.read_volatile() & lsr::RX_DONE) != 0
    }

    unsafe fn putc(&self, c : u8) {
        Self::THR.write_volatile(c);
    }

    unsafe fn getc(&self) -> u8 {
        return Self::RBR.read_volatile();
    }
}

impl CharBuffer {
    const fn new() -> Self { CharBuffer(VecDeque::new()) }

    /** Whether the buffer is empty. */
    unsafe fn is_empty(&self) -> bool { self.0.is_empty() }

    /** Put a char to the back of the buffer. */
    unsafe fn push_char(&mut self, c : u8) { self.0.push_back(c); }

    /** Take a char from the front of the buffer. */
    unsafe fn take_char(&mut self) -> Option<u8> { self.0.pop_front() }
}

mod fcr {
    pub const ENABLE : u8 = 0x1 << 0;       // Enable FIFOs
    pub const RX_CLR : u8 = 0x1 << 1;       // Clear receiver FIFO
    pub const TX_CLR : u8 = 0x1 << 2;       // Clear transmitter FIFO
    pub const CLEAR  : u8 = RX_CLR | TX_CLR;// Clear both FIFOs
}
mod lcr {
    pub const BAUD_LATCH : u8 = 0x1 << 7;   // Special mode to set baud rate
    pub const WORD_LEN_8 : u8 = 0x3 << 0;   // 8bit per word
}
mod ier {
    pub const DISABLE   : u8 = 0x0;         // Disable all interrupts
    pub const RX_ENABLE : u8 = 0x1 << 0;    // Enable receiver holding register
    pub const TX_ENABLE : u8 = 0x1 << 1;    // Enable transmitter holding register
}
mod dll {
    pub const BPS_38400 : u8 = 0x3;        // 38.4K baud rate
    pub const BPS_57600 : u8 = 0x2;        // 57.6K baud rate
    pub const BPS_115200 : u8 = 0x1;        // 115.2K baud rate
}
mod dlm {
    pub const BPS_38400 : u8 = 0x0;        // 38.4K baud rate
    pub const BPS_57600 : u8 = 0x0;        // 57.6K baud rate
    pub const BPS_115200 : u8 = 0x0;        // 115.2K baud rate
}
mod lsr {
    pub const TX_IDLE : u8 = 0x1 << 5;     // Transmitter idle
    pub const RX_DONE : u8 = 0x1 << 0;     // Receiver FIFO not empty
}
