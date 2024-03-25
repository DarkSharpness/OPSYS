use core::{arch, mem::size_of};
use riscv::register::*;
use crate::{uart_print, uart_println};
use crate::uart::init as init_uart;
use crate::layout::{clint, NCPU};

// In driver.asm and trap.asm
extern "C" { fn time_handle(); fn drop_mode(); }
static mut TIME_SCRATCH: [[u64 ; 5]; NCPU] = [[0 ; 5]; NCPU];

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

    sstatus::clear_sie();
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
    let id = mhartid::read(); // Get the hart id
    let interval = 1 << 22; // About 0.1s on QEMU
    let mtimecmp = clint::MTIMECMP.wrapping_add(id);
    let mtime    = clint::MTIME.wrapping_add(id);
    let time_scratch = TIME_SCRATCH[id].as_mut_ptr();

    // Set mtimecmp to mtime + interval
    *mtimecmp = *mtime + interval;

    // time_scratch[0..2]   = temporary storage
    // time_scratch[3]      = mtimecmp address
    // time_scratch[4]      = interval
    *time_scratch.wrapping_add(3) = mtimecmp as _;
    *time_scratch.wrapping_add(4) = interval as _;

    mscratch::write(time_scratch as _);
    mtvec::write(time_handle as _, mtvec::TrapMode::Direct);

    mie::set_mtimer();
    mstatus::set_mpie();
}
