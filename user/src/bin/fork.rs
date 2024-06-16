#![no_std]
#![no_main]

use user_lib::{println, sys_fork, sys_wait};

#[no_mangle]
fn main() -> i32 {
    unsafe {
        // let stdin = FileDescriptor::new(0);
        let child = sys_fork();
        println!("Hello, World!");
        if child == 0 {
            println!("Child");
            return 1;
        } else {
            println!("Parent");
            println!("Child pid: {}", child);
            match sys_wait() {
                Some((pid, status)) => {
                    println!("Child {} exited with status {}", pid.bits(), status);
                },
                None => println!("No child exited"),
            }
            return 0;
        }
    }
}
