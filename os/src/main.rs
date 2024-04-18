#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![allow(dead_code)]

#[macro_use]
mod console;
mod lang_items;
mod driver;
mod trap;
mod layout;
mod alloc;
mod proc;
mod syscall;

use core::arch::{asm, global_asm};
use proc::{init_process, run_process};

global_asm!(include_str!("entry.asm"));

#[no_mangle]
unsafe fn os_main() {
    init_tid_and_end_address();
    driver::init();
    init_process();
    run_process();
    driver::shutdown();
}


#[inline(always)]
fn init_tid_and_end_address() {
    unsafe {
        asm!("mv tp, a0");  // Thread id
        asm!("mv gp, a1");  // End address
    }
}
