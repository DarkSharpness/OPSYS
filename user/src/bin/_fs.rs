#![no_std]
#![no_main]
mod fs;

use user_lib::*;
extern crate alloc;

#[no_mangle]
unsafe fn main() -> i32 {
    println!("fs demo start");
    return 0;
}
