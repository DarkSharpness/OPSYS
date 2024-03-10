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

use core::{arch::global_asm, mem::size_of};

use crate::driver::{start, uart};

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("trap.asm"));

#[no_mangle]
unsafe fn os_main() {
    clear_bss();

    uart::init();
    start::init();

    // play::play();
    uart::shutdown();
    trap::user_trap();
}

#[no_mangle]
#[inline(never)]
fn clear_bss() {
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
