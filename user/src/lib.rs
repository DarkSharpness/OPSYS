#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod syscall;
pub mod inout;
pub use syscall::*;

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    println!("Panic: {:?}", info);
    loop {}
}

extern "C" { fn main() -> i32; }

#[no_mangle]
extern "C"
fn _start() { unsafe { sys_exit(main()); } }
