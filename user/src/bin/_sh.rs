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
        let mut bytes = string.as_bytes();
        while !bytes.is_empty() && bytes[0].is_ascii_whitespace() {
            bytes = &bytes[1..];
        }
        while !bytes.is_empty() && bytes.last().unwrap().is_ascii_whitespace() {
            bytes = &bytes[1..];
        }

        if bytes.is_empty() {
            put_prefix();
            continue;
        }

        if bytes == b"exit" {
            println!("-- Shell process exited --");
            println!("-- Goodbye! --");
            return 0;
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
                sys_exec(&bytes, core::ptr::null());
                panic!("-- No such program {} --", string)
            }
        }
        put_prefix();
    }
    return 0;
}
