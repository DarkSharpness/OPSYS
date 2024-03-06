#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod lang_items;
mod console;
mod sbi;

use core::{arch::global_asm, mem::size_of};

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn os_main() {
    use crate::sbi::func::putchar;
    clear_bss();
    putchar('h' as usize);
    // println!("Hello, world!");
    sbi::func::shutdown(false);
}

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
