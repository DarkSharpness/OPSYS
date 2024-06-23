#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod syscall;
mod memory;
pub mod inout;
pub use syscall::*;
pub use memory::*;

use inout::exit_stdout;

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    if let Some(location) = info.location() {
        errorln!("Panic at {}:{}:{}", location.file(), location.line(), location.column());
        errorln!("Panic message: {}", info.message().unwrap());
    } else {
        errorln!("Panic at unknown location");
    }
    unsafe { sys_exit(1) };
}

extern "C" { fn main() -> i32; }

#[no_mangle]
extern "C"
fn _start() -> ! {
    let result = unsafe { main() };
    exit_stdout();
    unsafe { sys_exit(result) };
}
