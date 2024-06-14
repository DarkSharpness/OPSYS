use crate::sys_read;
use super::{buffer::StdBuf, STDIN};

impl StdBuf {
    /** Read from the standard input. */
    fn read_buf(&mut self) {
        assert!(self.is_good());
        let result = unsafe { sys_read(STDIN, self.get_mut_buffer()) };
        if result < 0 {
            self.set_error();
        } else if result == 0 {
            self.set_eof();
        } else { // Result > 0
            self.set_range(0, result as _);
        }
    }

    /** Whether there's more possible white space. */
    fn skip_whitespace(&mut self) -> bool {
        let slice = self.as_slice();
        for i in 0..slice.len() {
            if !slice[i].is_ascii_whitespace() {
                self.pop_n(i as _);
                return false;
            }
        }
        self.set_clear();
        return true;
    }

    /** Skip all white space. */
    fn skip_all_whitespace(&mut self) {
        while self.is_good() && self.skip_whitespace() {
            self.read_buf();
        }
    }
}

#[allow(static_mut_refs)]
fn get_stdin() -> &'static mut StdBuf {
    static mut STDIN_STREAM: StdBuf = StdBuf::new();
    unsafe { &mut STDIN_STREAM }
}

#[allow(unused)]
pub fn read_int() -> Option<isize> {
    let stdin = get_stdin();

    stdin.skip_all_whitespace();
    if !stdin.is_good() { return None; }

    let slice = stdin.as_slice();
    let mut i = 0;
    let mut is_negative = false;

    if slice[0] == b'-' {
        is_negative = true;
        i += 1;
    }

    let mut result = 0;
    while i < slice.len() && slice[i].is_ascii_digit() {
        result = result * 10 + (slice[i] - b'0') as isize;
        i += 1;
    }

    if is_negative { result = -result; }

    stdin.pop_n(i as _);
    return Some(result);
}
