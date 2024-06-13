extern crate alloc;
use alloc::{boxed::Box, vec::Vec};
use sys::syscall::{ARGS_BUFFERED, ARGS_REGISTER};

use crate::{alloc::PTEFlag, proc::Process, utility::SliceIter};

pub enum Argument {
    Register(usize, usize),     // In 2 registers.
    Buffered(Box<[u8]>),        // In a kernel buffer.
    Upointer(*mut u8, usize),   // In a user pointer
}

fn create_sized_boxed(size : usize) -> Box<[u8]> {
    let mut tmp : Vec<u8> = Vec::new();
    tmp.resize(size, 0);
    return tmp.into_boxed_slice();
}

impl Argument {
    pub unsafe fn new(args : [usize; 3], process : &mut Process) -> Self {
        match args[2] {
            ARGS_REGISTER => {
                Self::Register(args[0], args[1])
            },
            ARGS_BUFFERED => {
                let buf = args[0];
                let len = args[1];
                let mut dst = create_sized_boxed(len);
                process.address_check([buf, len], PTEFlag::RO);
                process.get_satp().user_to_core(SliceIter::new(&mut dst), buf, len);
                Self::Buffered(dst)
            },
            _ => panic!("Invalid argument for syscall"),
        }
    }
}
