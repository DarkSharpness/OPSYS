use core::{arch, mem::size_of};
use riscv::register::*;
use crate::{uart_print, uart_println};
use crate::uart::init as init_uart;
use crate::layout::clint;

// In driver.asm and trap.asm
extern "C" { fn time_handle(); fn drop_mode(); }
static mut TIME_SCRATCH: [u64 ; 5] = [0 ; 5];

#[inline(never)]
pub unsafe fn init() {
    // Clear the bss section
    init_bss();
    // Initialize the uart for console I/O
    init_uart();

    uart_print!("Dropping to supervisor mode... ");

    // Set the return mode to supervisor mode
    init_mode();
    // Set the interrupt delegation
    init_intr();
    // Set the page table
    init_page();
    // Set the timer
    init_timer();

    // Code above is running in machine mode
    drop_mode();
    // Code below is running in supervisor mode

    uart_println!("Done!");
    uart_println!("Kernel is running on supervisor mode...");
}


/* Clear the bss section. */
#[no_mangle]
#[inline(never)]
fn init_bss() {
    extern "C" { fn sbss(); fn ebss(); }
    // A relatively faster way to clear the bss section
    // Since each section is 4096-byte aligned,
    // 8-byte stepping is safe enough.
    let mut beg = sbss as u64;
    let     end = ebss as u64;
    while beg != end {
        unsafe { (beg as *mut u64).write_volatile(0) }
        beg += size_of::<u64>() as u64;
    }
}

/* Set the return mode to supervisor mode. */
#[no_mangle]
#[inline(never)]
unsafe fn init_mode() {
    mstatus::set_mpp(mstatus::MPP::Supervisor);
}

/**
 * Enable all interrupts in supervisor mode.
 * Delegate interrupts and exceptions to supervisor mode.
 */
#[no_mangle]
#[inline(never)]
unsafe fn init_intr() {
    let val = 0xffff;
    arch::asm!("csrw mideleg, {}", in(reg) val);
    arch::asm!("csrw medeleg, {}", in(reg) val);
    sie::set_sext();    // External interrupt
    sie::set_stimer();  // Timer interrupt
    sie::set_ssoft();   // Software interrupt
}

/**
 * Allow to use all physical address
 * In supervisor mode, you don't have access
 * to all physical address by default.
 * 
 * Our kernel don't feat a page table.
 */
#[no_mangle]
#[inline(never)]
unsafe fn init_page() {
    pmpaddr0::write(0x3fffffffffffff);
    pmpcfg0::write(0xf);
}

/**
 * Set the timer interrupt.
 * Initialize the interval to about 0.1s.
 */
#[no_mangle]
#[inline(never)]
unsafe fn init_timer() {
    let interval = 1 << 20; // About 0.1s on QEMU
    clint::MTIMECMP.write_volatile(
        clint::MTIME.read_volatile() + interval,
    );

    TIME_SCRATCH[3] = clint::MTIMECMP as u64;
    TIME_SCRATCH[4] = interval;

    mscratch::write(TIME_SCRATCH.as_ptr() as usize);
    mtvec::write(time_handle as usize, mtvec::TrapMode::Direct);
}
