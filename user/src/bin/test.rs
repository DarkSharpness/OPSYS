#![no_std]
#![no_main]

use user_lib::*;
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    let mut arr : [u8; 16] = [0; 16];
    unsafe {
        let stdin   = FileDescriptor::new(0);
        let stdout  = FileDescriptor::new(1);
        let buf = arr.as_mut_ptr() as *mut u8;
        sys_read(stdin, buf, 16);
        sys_write(stdout, buf, 16);
        sys_yield();
    }
    return 0;
}
