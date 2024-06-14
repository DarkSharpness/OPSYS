#![no_std]
#![no_main]

use inout::read_int;
use user_lib::*;

#[no_mangle]
fn main() -> i32 {
    println!("Hello, world!");
    let x = read_int().unwrap();
    let y = read_int().unwrap();
    println!("You input: {} {}", x, y);
    panic!("Exit");
    // return 0;
}
