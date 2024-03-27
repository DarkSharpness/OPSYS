#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![allow(dead_code)]

mod lang_items;
mod console;
mod sbi;
mod driver;
mod trap;
mod play;
mod layout;
mod alloc;
mod debug;

use core::arch::{asm, global_asm};

use driver::get_tid;

use crate::driver::{start, uart};

global_asm!(include_str!("entry.asm"));

#[no_mangle]
unsafe fn os_main() {
    init_tid_and_end_address();
    start::init();
    if get_tid() == 0 {
        trap::init_trap();
        uart::shutdown();
        trap::user_trap();
    }
}

#[inline(always)]
fn init_tid_and_end_address() {
    unsafe {
        asm!("mv tp, a0");  // Thread id
        asm!("mv gp, a1");  // End address
    }
}
