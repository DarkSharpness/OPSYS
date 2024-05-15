extern crate alloc;

use alloc::{collections::VecDeque, vec::Vec};
use crate::proc::{Process, ProcessStatus};

use super::uart::sync_putc;

pub struct Console {
    pub(crate) stdin : VecDeque<u8>,
    buffer  : Vec<u8>,  // Input buffer
    length  : usize,    // Last written position
    queue   : VecDeque<*mut Process>,
}

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
            queue   : VecDeque::new(),
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

            while let Some(process) = self.queue.pop_front() {
                (*process).wake_up_from(ProcessStatus::SERVICE);
            }
        } else {
            sync_putc(c);
            self.buffer.push(c);
        }
    }

    pub unsafe fn putc(&mut self, c : u8) {
        sync_putc(c);
        self.length = self.buffer.len();
    }

    pub unsafe fn try_read(&mut self, process : *mut Process) {
        while self.stdin.len() == 0 {
            self.queue.push_back(process);
            (*process).sleep_as(ProcessStatus::SERVICE);
        }
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
