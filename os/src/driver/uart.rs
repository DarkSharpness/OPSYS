#![allow(dead_code)]

use crate::console::print_separator;

type Uptr = * mut u8;
const UART : Uptr = 0x1000_0000 as Uptr; // 1 << (7 * 4)
const RBR : Uptr = UART.wrapping_add(0x0);
const THR : Uptr = UART.wrapping_add(0x0);
const DLL : Uptr = UART.wrapping_add(0x0);
const DLM : Uptr = UART.wrapping_add(0x1);
const IER : Uptr = UART.wrapping_add(0x1);
const IIR : Uptr = UART.wrapping_add(0x2);
const FCR : Uptr = UART.wrapping_add(0x2);
const LCR : Uptr = UART.wrapping_add(0x3);
const MCR : Uptr = UART.wrapping_add(0x4);
const LSR : Uptr = UART.wrapping_add(0x5);
const MSR : Uptr = UART.wrapping_add(0x6);
const SCR : Uptr = UART.wrapping_add(0x7);

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

pub unsafe fn init() {
    // Disable all interrupts
    IER.write_volatile(ier::DISABLE);

    // Special mode to set baud rate
    LCR.write_volatile(lcr::BAUD_LATCH);

    // Set baud rate to 38.4 kbps
    DLL.write_volatile(dll::BPS_38400);
    DLM.write_volatile(dlm::BPS_38400);

    // Leave special mode and set word length to 8 bits
    LCR.write_volatile(lcr::WORD_LEN_8);

    // Reset and enable FIFOs
    FCR.write_volatile(fcr::ENABLE | fcr::CLEAR);

    // Enable receiver and transmitter
    IER.write_volatile(ier::RX_ENABLE | ier::TX_ENABLE);

    uart_println!("UART initialization done!");
    print_separator();
}

pub unsafe fn sync_putc(c : u8) {
    while (LSR.read_volatile() & lsr::TX_IDLE) == 0 {}
    THR.write_volatile(c);
}

pub unsafe fn sync_getc() -> i32 {
    while (LSR.read_volatile() & lsr::RX_DONE) == 0 {}
    RBR.read_volatile() as i32
}

/**
 * Get the char from UART
 * If there is no char in the buffer, return None
 */
pub unsafe fn uart_getc() -> Option<i32> {
    if (LSR.read_volatile() & lsr::RX_DONE) == 0 {
        None
    } else {
        Some(RBR.read_volatile() as i32)
    }
}

const BUFFER_SIZE : usize = 1024;
static mut BUFFER : [u8; 1024] = [0; BUFFER_SIZE];
static mut HEAD : usize = 0;
static mut TAIL : usize = 0;

unsafe fn uart_putc(c : u8) {
    // Acquire lock?

    while TAIL == HEAD + BUFFER_SIZE {
        // Buffer is full!
        // Should sleep
        todo!("Buffer is full!");
    }

    BUFFER[TAIL % BUFFER_SIZE] = c;
    TAIL += 1;

    uart_start();

    // Release lock?
}


/**
 * Lock holder should call uart_start() to start sending data
 */
unsafe fn uart_start() {
    while HEAD != TAIL {
        // The buffer is full, we cannot put more data
        if (LSR.read_volatile() & lsr::TX_IDLE) == 0 { return; }

        let c = BUFFER[HEAD % BUFFER_SIZE];
        HEAD += 1;

        // May be wake up some putc waiting for buffer first.
        THR.write_volatile(c);
    }
}

#[no_mangle]
unsafe fn uart_trap() {
    loop {
        match uart_getc() {
            Some(c) => {
                todo!("Console putchar {}", c as u8 as char);
                // Call console intr
            },
            None => break
        }
    }

    // Take the lock first.

    uart_start();

    // Release the lock.
}

