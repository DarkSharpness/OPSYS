use core::arch;
use riscv::register::*;
use crate::{driver::timer, uart_print, uart_println};

extern "C" { fn drop_down(); }

/**
 * Drop from machine mode to supervisor mode
 */
pub unsafe fn init() {
    uart_print!("Dropping to supervisor mode...");

    // Set the supervisor stack pointer
    mstatus::set_mpp(mstatus::MPP::Supervisor);

    // Delegate all interrupts and exceptions to supervisor mode
    let val = 0xffff;
    arch::asm!("csrw mideleg, {}", in(reg) val);
    arch::asm!("csrw medeleg, {}", in(reg) val);
    sie::set_sext();
    sie::set_stimer();
    sie::set_ssoft();

    // Allow to use all physical address
    pmpaddr0::write(0x3fffffffffffff);
    pmpcfg0::write(0xf);

    // Set the time related.
    timer::timer_init();

    // Code above is running in machine mode
    drop_down();
    // Code below is running in supervisor mode

    uart_println!("Done!");
    uart_println!("Kernel is running on supervisor mode......");
}
