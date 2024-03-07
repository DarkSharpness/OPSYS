#![allow(dead_code)]

use core::arch;


pub unsafe fn init() {
    // Set mtvec to point to the handler function
    arch::asm!("la t0, handler");
    arch::asm!("csrw mtvec, t0");

    // Set mscratch to temporary storage
    arch::asm!("la t0, call_stack_top");
    arch::asm!("csrw mscratch, t0");

    // Enable machine mode interrupts
    arch::asm!("csrs mstatus, 0x8");
}

#[no_mangle]
unsafe fn handler() {
    // Save all registers.

    
    // Return to the caller.
    arch::asm!("mret");
}
