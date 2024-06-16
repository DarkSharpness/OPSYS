#![no_std]
#![no_main]

use inout::{flush_stdout, read_line};
use user_lib::*;
extern crate alloc;
use alloc::string::String;

#[no_mangle]
unsafe fn main() -> i32 {
    let mut string = String::new();    
    print!("$ "); flush_stdout();
    while read_line(&mut string) {
        match sys_fork() {
            ForkResult::Error => panic!("fork error"),
            ForkResult::Parent(pid) => {
                match sys_wait() {
                    WaitResult::Error => panic!("wait error"),
                    WaitResult::None => panic!("no child process"),
                    WaitResult::Some(_pid, code) => {
                        assert!(pid == _pid);
                        println!("shell process exited with {}", code)
                    }
                }
            },
            ForkResult::Child => {
                let bytes = string.as_bytes();
                sys_exec(&bytes, core::ptr::null());
                panic!("exec error")
            }
        }
        print!("$ "); flush_stdout();
    }
    return 0;
}
