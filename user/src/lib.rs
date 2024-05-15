#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod syscall;
pub use syscall::*;

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern "C" { fn main() -> i32; }


#[no_mangle]
extern "C"
fn _start() {
    unsafe { sys_exit(main()); }
}
