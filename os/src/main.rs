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

use core::arch::global_asm;

use crate::driver::{start, uart};

global_asm!(include_str!("entry.asm"));

#[no_mangle]
unsafe fn os_main() {
    start::init();
    // play::play();
    uart::shutdown();
    trap::user_trap();
}
