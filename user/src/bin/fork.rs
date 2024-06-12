#![no_std]
#![no_main]

use user_lib::*;

#[no_mangle]
fn main() -> i32 {
    unsafe {
        // let stdin = FileDescriptor::new(0);
        let child = sys_fork();
        println!("Hello, World!");
        if child == 0 {
            println!("Child");
        } else {
            println!("Parent");
            println!("Child pid {}:", child);
        }
        sys_yield();
        sys_yield();
    }
    return 0;
}
