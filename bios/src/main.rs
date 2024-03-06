#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod lang_items;

use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub unsafe fn bios_main() {}
