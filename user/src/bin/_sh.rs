#![no_std]
#![no_main]

use inout::{flush_stdout, read_line};
use user_lib::*;
extern crate alloc;
use alloc::string::String;

fn put_prefix() {
    print!("$ "); flush_stdout();
}

#[no_mangle]
unsafe fn main() -> i32 {
    let mut string = String::new();
    put_prefix();
    while read_line(&mut string) {
        if string.is_empty() {
            put_prefix();
            continue;
        }
        match sys_fork() {
            ForkResult::Error => {},
            ForkResult::Parent(pid) => {
                match sys_wait() {
                    WaitResult::Error => panic!("wait error"),
                    WaitResult::None => panic!("no child process"),
                    WaitResult::Some(_pid, code) => {
                        assert!(pid == _pid);
                        println!("-- Shell process exited with {} --", code)
                    }
                }
            },
            ForkResult::Child => {
                let bytes = string.as_bytes();
                sys_exec(&bytes, core::ptr::null());
                panic!("-- No such program {} --", string)
            }
        }
        put_prefix();
    }
    return 0;
}
