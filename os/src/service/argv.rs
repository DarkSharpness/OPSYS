extern crate alloc;
use alloc::{boxed::Box, vec::Vec};

use crate::{proc::Process, utility::SliceIter};

pub enum Argument {
    Register(usize, usize),     // In 2 registers.
    Buffered(Box<[u8]>),        // In a kernel buffer.
    Upointer(*mut u8, usize),   // In a user pointer
}

impl Argument {
    pub unsafe fn new(args : &[usize], process : &mut Process) -> Self {
        match args[2] {
            0 => {
                Self::Register(args[0], args[1])
            },
            1 => {
                let mut tmp : Vec<u8> = Vec::new();
                tmp.resize(args[1], 0);
                let mut dst = tmp.into_boxed_slice();
                process.get_satp().user_to_core(SliceIter::new(&mut dst), args[0], args[1]);
                Self::Buffered(dst)
            },
            _ => panic!("Invalid argument"),
        }
    }
}

