use core::fmt::{self, Write};
use crate::sys_write;
use super::{buffer::StdBuf, STDOUT};

fn find_last_newline(buf: &[u8]) -> Option<usize> {
    buf.iter().rposition(|&c| c == b'\n')
}

impl StdBuf {
    /** Write to the standard output */
    fn write_buf(&mut self) {
        assert!(self.is_good());
        let slice = self.as_slice();
        let result = unsafe { sys_write(STDOUT, slice) };
        if result < 0 {
            self.set_error();
        } else if result < slice.len() as isize {
            self.set_eof();
        } else {
            self.set_clear();
        }
    }

    fn insert_buf_in_bound(&mut self, bytes: &[u8]) {
        match find_last_newline(bytes) {
            None => {
                self.push_str(bytes);
            }
            Some(pos) => {
                let (prefix, suffix) = bytes.split_at(pos + 1);
                self.push_str(prefix);
                self.write_buf();
                self.push_str(suffix);
            }
        }
    }

    fn flush_write(&mut self, bytes: &[u8]) {
        self.write_buf();
        unsafe { sys_write(STDOUT, bytes); }
    }

    fn insert_str(&mut self, bytes: &[u8]) {
        if bytes.len() <= self.remain().len() {
            self.insert_buf_in_bound(bytes);
        } else {
            self.flush_write(bytes);
        }
    }
}

#[allow(static_mut_refs)]
fn get_stdout() -> &'static mut StdBuf {
    static mut STDOUT_STREAM: StdBuf = StdBuf::new();
    unsafe { &mut STDOUT_STREAM }
}

#[allow(unused)]
pub fn print_fmt(args: fmt::Arguments) {
    get_stdout().write_fmt(args).unwrap();
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

impl Write for StdBuf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let stdout = get_stdout();
        stdout.insert_str(s.as_bytes());
        Ok(())
    }
}

pub fn flush_stdout() {
    get_stdout().write_buf();
}
