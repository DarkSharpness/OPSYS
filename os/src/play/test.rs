use crate::{driver::uart, uart_println};

#[inline(always)]
fn is_digit(c: i32) -> bool { c >= 48 && c <= 57 }

#[inline(never)]
pub unsafe fn play() {
    let mut val : u64 = 0;
    loop {
        let _c = uart::sync_getc();
        uart::sync_putc(_c as u8);
        if !is_digit(_c) { break; }
        val = val * 10 + (_c - 48) as u64;
    }
    uart::sync_putc('\n' as u8);
    uart_println!("You have entered: {}", val);
}
