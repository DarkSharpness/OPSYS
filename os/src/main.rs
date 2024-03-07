#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod lang_items;
mod console;
mod sbi;
mod driver;

use core::{arch::global_asm, mem::size_of};

use crate::driver::uart;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
fn os_main() {
    clear_bss();
    unsafe {
        uart::init();
        uart_println!("Hello, world!");
        play();
        uart::shutdown();
    }
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

#[inline(always)]
fn is_digit(c: i32) -> bool { c >= 48 && c <= 57 }

#[inline(never)]
unsafe fn play() {
    let mut val : u64 = 0;
    loop {
        let _c = uart::getc();
        uart::putc(_c as u8);
        if !is_digit(_c) { break; }
        val = val * 10 + (_c - 48) as u64;
    }
    driver::uart::putc('\n' as u8);
    uart_println!("You have entered: {}", val);
}
