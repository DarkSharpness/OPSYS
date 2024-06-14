use core::fmt::{self, Write};

use crate::sys_write;

use super::STDERR;

struct Stderr;

impl Write for Stderr {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        unsafe { sys_write(STDERR, bytes); }
        Ok(())
    }
}

fn get_stderr() -> Stderr {
    Stderr
}

#[allow(unused)]
pub fn error_fmt(args: fmt::Arguments) {
    get_stderr().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! error {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::inout::error_fmt(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! errorln {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::inout::error_fmt(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
