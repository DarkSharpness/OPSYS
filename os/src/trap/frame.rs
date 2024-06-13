use riscv::register::satp;

use crate::alloc::{PTEFlag, PageAddress, PAGE_SIZE};

use super::{user_trap, TRAP_FRAME};

#[repr(C)]
pub struct TrapFrame {
    pub ra  : usize,
    pub gp  : usize,
    pub tp  : usize,
    pub t0  : usize,
    pub t1  : usize,
    pub t2  : usize,
    pub s0  : usize,
    pub s1  : usize,

    pub a0  : usize,
    pub a1  : usize,
    pub a2  : usize,
    pub a3  : usize,
    pub a4  : usize,
    pub a5  : usize,
    pub a6  : usize,
    pub a7  : usize,

    pub s2  : usize,
    pub s3  : usize,
    pub s4  : usize,
    pub s5  : usize,
    pub s6  : usize,
    pub s7  : usize,
    pub s8  : usize,
    pub s9  : usize,

    pub s10 : usize,
    pub s11 : usize,
    pub t3  : usize,
    pub t4  : usize,
    pub t5  : usize,
    pub t6  : usize,
    pub sp  : usize,
    pub pc  : usize,

    thread_number   : usize,  // real thread number
    kernel_satp     : usize,  // kernel page table
    kernel_trap     : usize,  // kernel trap handler
    kernel_stack    : usize,  // kernel stack pointer
}

impl TrapFrame {
    // Copy only 32 registers
    pub unsafe fn copy_from(&self, src: &TrapFrame) {
        let dst = self as *const TrapFrame as *mut usize;
        let src = src as *const TrapFrame as *const usize;
        dst.copy_from(src, 32);
    }
}

impl PageAddress {
    /** Create a trap frame with all the private members initialized. */
    pub unsafe fn map_trap_frame(&self) -> (&'static mut TrapFrame, usize) {
        let trap_frame = PageAddress::new_rand_page();
        self.smap(TRAP_FRAME, trap_frame, PTEFlag::RW);
        let trap_frame = &mut *(trap_frame.address() as *mut TrapFrame);

        let kernel_stack = PageAddress::new_rand_page();
        let kernel_stack_top = kernel_stack.address() as usize + PAGE_SIZE;

        trap_frame.thread_number = 0;
        trap_frame.kernel_stack = kernel_stack_top;
        trap_frame.kernel_satp = satp::read().bits();
        trap_frame.kernel_trap  = user_trap as _;

        return (trap_frame, kernel_stack_top);
    }
}
