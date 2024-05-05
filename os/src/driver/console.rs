use alloc::{collections::VecDeque, vec::Vec};

use super::{file::File, uart::sync_putc};

extern crate alloc;

pub struct Console {
    stdin   : VecDeque<u8>,
    buffer  : Vec<u8>,  // Input buffer
    length  : usize,    // Last written position
}

static mut CONSOLE : Console = Console::new();

unsafe fn backspace() {
    sync_putc(8 as u8);
    sync_putc(' ' as u8);
    sync_putc(8 as u8);
}

impl Console {
    const P : u8 = ('P' as u8) - ('@' as u8);   // Print
    const U : u8 = ('U' as u8) - ('@' as u8);   // Remove a line
    const H : u8 = ('H' as u8) - ('@' as u8);   // Delete a character
    const D : u8 = ('D' as u8) - ('@' as u8);   // End of file
    const DELETE : u8 = 127;                    // Delete a character
    const ENTER  : u8 = 13;                     // Enter

    pub const fn new() -> Console {
        return Console {
            stdin   : VecDeque::new(),
            buffer  : Vec::new(),
            length  : 0,
        }
    }

    pub unsafe fn getc(&mut self, c : u8) {
        if self.try_interpret(c) { return; }

        if c == Self::ENTER || c == Self::D || c == '\n' as u8 || c == '\r' as u8 {
            sync_putc('\n' as _);

            self.stdin.extend(self.buffer.iter());
            self.stdin.push_back('\n' as _);

            self.buffer.clear();
            self.length = 0;

            todo!("Wake up reading process");
        } else {
            sync_putc(c);
            self.buffer.push(c);
        }
    }

    pub unsafe fn putc(&mut self, c : u8) {
        sync_putc(c);
        self.length = self.buffer.len();
    }

    /// Remove a character from input
    unsafe fn try_backspace(&mut self) {
        if self.length < self.buffer.len() {
            backspace();
            self.buffer.pop();
        }
    }

    /// Remove a line of input
    unsafe fn try_flushline(&mut self) {
        let  remain = self.buffer.len() - self.length;
        self.length = 0;        // The length is reset.
        self.buffer.clear();    // Whatever input is cleared.
        for _ in 0..remain { backspace(); }
    }

    /// Try to interpret a control character
    unsafe fn try_interpret(&mut self, c : u8) -> bool {
        match c {
            Self::P                => todo!("Dump the process"),
            Self::U                => self.try_flushline(),
            Self::H | Self::DELETE => self.try_backspace(),
            _ => return false,
        }
        return true;   // The character is interpreted.
    }
}

impl File for Console {
    fn read(&mut self, buf: *mut u8, n : usize) -> usize {
        for i in 0..n {
            if let Some(c) = self.stdin.pop_front() {
                unsafe { buf.add(i).write(c); }
            } else {
                return i;
            }
        }
        return n;
    }
    fn write(&mut self, buf: *const u8, n : usize) -> usize {
        unsafe {
            CONSOLE.length = CONSOLE.buffer.len();
            for i in 0..n { sync_putc(buf.add(i).read()); }
        }
        return n;
    }
}

#[allow(static_mut_refs)]
pub fn get_console_file() -> *mut dyn File { return unsafe { &mut CONSOLE }; }
