use crate::sys_read;

use super::STDIN;

#[derive(PartialEq, Eq)]
enum State {
    Normal      = 0,
    EndOfFile   = 1,
    Error       = 2,
}

struct Stdin {
    buffer : [u8; 256],
    begin  : u8,
    finish : u8,
    state  : State,
}

impl Stdin {
    pub const fn new() -> Self {
        Self {
            buffer  : [0; 256],
            begin   : 0,
            finish  : 0,
            state   : State::Normal,
        }
    }

    fn read_buf(&mut self) {
        let result = unsafe { sys_read(STDIN, &mut self.buffer) };
        if result < 0 {
            self.state = State::EndOfFile;
        } else if result == 0 {
            self.state = State::Error;
        } else { // Result > 0
            self.begin = 0;
            self.finish = result as _;
        }
    }

    fn is_good(&self) -> bool {
        self.state == State::Normal
    }

    fn is_empty(&self) -> bool {
        self.begin == self.finish
    }

    /** Whether there's no more possible white space. */
    fn skip_whitespace(&mut self) -> bool {
        let mut i = self.begin;
        while i != self.finish && self.buffer[i as usize].is_ascii_whitespace() {
            i += 1; 
        }
        self.begin = i;
        return self.is_empty();
    }

    fn skip_all_whitespace(&mut self) {
        while self.is_good() && self.skip_whitespace() {
            self.read_buf();
        }
    }

    fn as_slice(&self) -> &[u8] {
        &self.buffer[self.begin as usize..self.finish as usize]
    }

    fn pop(&mut self, n: u8) {
        self.begin += n;
    }

}

static mut STDIN_STREAM: Stdin = Stdin::new();

#[allow(static_mut_refs)]
fn get_stdin() -> &'static mut Stdin {
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

    stdin.pop(i as _);
    return Some(result);
}
