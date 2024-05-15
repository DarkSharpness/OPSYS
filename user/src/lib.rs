#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod syscall;

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}