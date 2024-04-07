use crate::driver::uart::sync_putc as putchar;
use core::fmt::{self, Write};

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            unsafe { putchar(c as _); }
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! uart_print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! uart_println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! warning {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::uart_println!("\x1b[33m[WARNING]\x1b[0m: {}", format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! warning_inline {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::uart_print!("\x1b[33m[WARNING]\x1b[0m: {}", format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! logging {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::uart_println!("\x1b[32m[LOGGING]\x1b[0m: {}", format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! logging_inline {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::uart_print!("\x1b[32m[LOGGING]\x1b[0m: {}", format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! normal {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::uart_println!("\x1b[35m[MESSAGE]\x1b[0m: {}", format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! normal_inline {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::uart_print!("\x1b[35m[MESSAGE]\x1b[0m: {}", format_args!($fmt $(, $($arg)+)?));
    }
}

#[inline(always)]
pub fn print_separator() {
    uart_println!("----------------------------------------");
}

