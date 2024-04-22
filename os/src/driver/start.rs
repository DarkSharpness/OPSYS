use core::{arch::asm, mem::size_of};
use riscv::register::*;
use crate::driver::get_mem_end;
use crate::driver::timer;
use crate::driver::uart;
use crate::alloc;
use super::get_tid;

pub unsafe fn init() {
    extern "C" { fn drop_mode(); }
    // Clear the bss section first
    init_bss();

    // Only initialize once (by the first core)
    if get_tid() == 0 {
        // Initialize basic uart for input and output.
        uart::init();
        // Set up the buddy allocator and establish page table.
        alloc::init(get_mem_end());
    }

    // Set the interrupt delegation
    init_intr();
    // Set the page table
    init_page();
    // Set the timer
    timer::init();
    logging_inline!("Dropping to supervisor mode...");

    // Set the return mode to supervisor mode
    init_mode();
    drop_mode();
    uart_println!("Done!");

    // Now, the kernel is running on supervisor mode.
    logging!("Kernel is running on supervisor mode.");
}

/** Clear the bss section. */
unsafe fn init_bss() {
    extern "C" { fn sbss(); fn ebss(); }
    // A relatively faster way to clear the bss section
    // Since each section is 4096-byte aligned,
    // 8-byte stepping is safe enough.
    let mut beg = sbss as usize;
    let     end = ebss as usize;
    while beg != end {
        (beg as *mut usize).write(0);
        beg += size_of::<usize>() as usize;
    }
}

/** Set the return mode to supervisor mode. */
unsafe fn init_mode() {
    mstatus::set_mpp(mstatus::MPP::Supervisor);
}

/**
 * Enable all interrupts in supervisor mode.
 * Delegate interrupts and exceptions to supervisor mode.
 */
unsafe fn init_intr() {
    let val = 0xffff;
    asm!("csrw mideleg, {}", in(reg) val);
    asm!("csrw medeleg, {}", in(reg) val);
    sie::set_sext();    // External interrupt
    sie::set_stimer();  // Timer interrupt
    sie::set_ssoft();   // Software interrupt

    sstatus::clear_sie(); // Disable interrupt first.
}

/**
 * Allow to use all physical address
 * In supervisor mode, you don't have access
 * to all physical address by default.
 */
unsafe fn init_page() {
    pmpaddr0::write(0x3fffffffffffff);
    pmpcfg0::write(0xf);
    // Start using page table
    asm!("sfence.vma");
    satp::set(satp::Mode::Sv39, 0, alloc::PAGE_TABLE.bits() as _);
    asm!("sfence.vma");
}
