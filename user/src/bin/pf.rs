#![no_std]
#![no_main]

use core::arch::asm;

use user_lib::*;

#[no_mangle]
fn main() -> i32 {
    unsafe { overflow_test(); }
    return recurse(1000);
}

unsafe fn overflow_test() {
    let mut addr : *mut u8;
    asm!("mv {}, sp", out(reg) addr);

    addr = addr.offset(-0x2000);
    let buf = core::slice::from_raw_parts_mut(addr, 0x1000);
    let len = sys_read(FileDescriptor::new_debug(0), buf);
    assert!(len > 0, "read error");
    sys_write(FileDescriptor::new_debug(1), &buf[0..(len as usize)]);
}

fn recurse(n: i32) -> i32 {
    if n == 0 {
       return 0;
    } else {
        return n + recurse(n - 1);
    }
}
