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
mod service;
mod cpu;
mod utility;

use core::arch::{asm, global_asm};

use alloc::PAGE_SIZE;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

extern "C" {
    fn _num_app();
}

#[no_mangle]
unsafe fn os_main() {
    init_tid_and_end_address();
    driver::init();
    proc::init_process();
    proc::run_process();
    driver::shutdown();
}

fn init_tid_and_end_address() {
    unsafe {
        asm!("mv tp, a0");  // Thread id
        asm!("mv gp, a1");  // End address
    };
}

const unsafe fn get_zero_page() -> &'static [u8] {
    let position = (0x80_000_000 as usize) + PAGE_SIZE;
    return core::slice::from_raw_parts(position as *const u8, PAGE_SIZE);
}
