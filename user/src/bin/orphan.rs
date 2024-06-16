#![no_std]
#![no_main]

use user_lib::*;

#[no_mangle]
fn main() -> i32 {
    unsafe {
        // let stdin = FileDescriptor::new(0);
        let child = sys_fork();
        println!("Hello, World!");
        match child {
            ForkResult::Error => {
                println!("Fork failed");
                return -1;
            },
            ForkResult::Child => {
                sys_yield();
                sys_yield();
                sys_yield();
                println!("I'm the child, my parent is dead!");
                return 0;
            },
            ForkResult::Parent(_) => {
                println!("I'm the parent.");
                return 0;
            },
        }
    }
}
