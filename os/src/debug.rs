use crate::uart_println;

#[inline(always)]
pub fn print_separator() {
    uart_println!("----------------------------------------");
}
