#![no_std]
#![no_main]

use user_lib::{println, sys_fork, sys_wait, ForkResult, WaitResult};

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
                println!("Child");
                return 1;
            },
            ForkResult::Parent(pid) => {
                println!("Parent");
                println!("Child pid: {}", pid.bits());
                match sys_wait() {
                    WaitResult::Error => {
                        println!("Wait failed");
                        return -1;
                    },
                    WaitResult::None => {
                        println!("No child to wait");
                        return -1;
                    },
                    WaitResult::Some(pid, status) => {
                        println!("Child exited: pid={}, status={}", pid.bits(), status);
                        return 0;
                    },
                }
            },
        }
    }
}
