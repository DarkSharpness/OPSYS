#![no_std]
#![no_main]

use user_lib::{inout::read_int, println, sys_fork, sys_wait, ForkResult, WaitResult};

#[no_mangle]
fn main() -> i32 {
    unsafe {
        // let stdin = FileDescriptor::new(0);
        let child = sys_fork();
        println!("Hello, World!");
        let num = read_int().unwrap();
        match child {
            ForkResult::Error => {
                println!("Fork failed");
                return -1;
            },
            ForkResult::Child => {
                println!("Child input: {}", num);
                return 1;
            },
            ForkResult::Parent(pid) => {
                println!("Parent input: {}", num);
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
                        println!("Child exit: pid = {}, exit_code = {}", pid.bits(), status);
                        return 0;
                    },
                }
            },
        }
    }
}
