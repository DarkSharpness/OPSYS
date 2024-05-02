use alloc::collections::VecDeque;

use super::uart::sync_putc;

extern crate alloc;

pub struct Console {
    buffer  : VecDeque<u8>,
    length  : usize,    // Input length
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
        Console {
            buffer  : VecDeque::new(),
            length  : 0,
        }
    }

    pub unsafe fn getc(&mut self, c : u8) {
        if self.try_interpret(c) { return; }

        // Translate the character
        let c = if c == Self::ENTER { '\n' as u8 } else { c };

        sync_putc(c);
        self.buffer.push_back(c);

        if c == '\n' as u8 || c == Self::D {
            self.length = 0;
            todo!("Wake up the process");
        } else {
            self.length += 1;
        }
    }

    pub unsafe fn putc(&mut self, c : u8) {
        sync_putc(c);
        self.length = 0;
    }

    unsafe fn try_backspace(&mut self) {
        if self.length == 0 { return; }
        backspace();
        self.buffer.pop_back();
        self.length -= 1;
    }

    unsafe fn try_interpret(&mut self, c : u8) -> bool {
        match c {
            Self::P => todo!("Dump the process"),
            Self::U => todo!("Remove a line"),
            Self::H | Self::DELETE => self.try_backspace(),
            _ => return false,
        }
        return true;   // The character is interpreted.
    }
}
