#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod lang_items;
mod uart;
mod ecall;

use core::arch::{self, global_asm};

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub unsafe fn bios_main() {
    // uart::init();
    // ecall::init();
    exit();
}

#[inline(always)]
#[no_mangle]
pub unsafe fn exit() {
    arch::asm!("li t0, 0x80200000");
    arch::asm!("jr t0");
}
