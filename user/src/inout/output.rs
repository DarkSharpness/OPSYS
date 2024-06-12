struct Stdout;

use core::fmt::{self, Write};
use crate::sys_write;
use super::STDOUT;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let buf = s.as_bytes();
        unsafe { sys_write(STDOUT, buf); }
        Ok(())
    }
}


#[allow(unused)]
pub fn print_fmt(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::inout::print_fmt(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::inout::print_fmt(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
