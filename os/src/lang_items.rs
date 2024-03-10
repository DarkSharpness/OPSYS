use crate::{driver::uart, uart_println as println};
use core::panic::PanicInfo;

#[panic_handler]
#[inline(never)]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("Panicked: {}", info.message().unwrap());
    }
    unsafe { uart::shutdown() };
    loop {}
}
