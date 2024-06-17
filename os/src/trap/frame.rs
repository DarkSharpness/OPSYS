use core::ptr::null_mut;

use riscv::register::satp;

use crate::alloc::{PTEFlag, PageAddress, KERNEL_SATP, PAGE_SIZE};

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
    /** Copy only 32 registers from src to self. */
    pub unsafe fn copy_from(&mut self, src: &TrapFrame) {
        let dst = self as *mut TrapFrame as *mut usize;
        let src = src as *const TrapFrame as *const usize;
        dst.copy_from(src, 32);
    }
    pub unsafe fn free(&self) {
        let stack_top = self.kernel_stack;
        FRAME_ALLOCATOR.deallocate(stack_top);
    }
    pub unsafe fn debug(&self) {
        message!("    sepc = {:#x}", self.pc);
        message!("    satp = {:#x}", self.kernel_satp);
        message!("    kernel_stack = {:#x}", self.kernel_stack);
    }
}

struct FrameAllocator {
    last    : *mut usize,
    lowest  : usize,
}

static mut FRAME_ALLOCATOR : FrameAllocator = FrameAllocator::new();

impl PageAddress {
    /** Create a trap frame with all the private members initialized. */
    pub unsafe fn map_trap_frame(&self) -> (&'static mut TrapFrame, usize) {
        let trap_frame = self.new_smap(TRAP_FRAME, PTEFlag::RW);
        let trap_frame = &mut *(trap_frame.address() as *mut TrapFrame);

        let kernel_stack = FRAME_ALLOCATOR.allocate();
        let kernel_stack_top = kernel_stack + PAGE_SIZE;

        trap_frame.thread_number = 0;
        trap_frame.kernel_stack = kernel_stack_top;
        trap_frame.kernel_satp  = satp::read().bits();
        trap_frame.kernel_trap  = user_trap as _;

        return (trap_frame, kernel_stack_top);
    }
}

const KERNEL_STACK_BEGIN : usize = (PAGE_SIZE * 8).wrapping_neg();

impl FrameAllocator {
    pub const fn new() -> Self {
        Self {
            last    : null_mut(),
            lowest  : KERNEL_STACK_BEGIN,
        }
    }

    pub unsafe fn allocate(&mut self) -> usize {
        if self.last == null_mut() {
            self.lowest -= PAGE_SIZE * 4;
            let result = self.lowest + 3 * PAGE_SIZE;
            KERNEL_SATP.new_smap(result - PAGE_SIZE, PTEFlag::RW);
            KERNEL_SATP.new_smap(result, PTEFlag::RW);
            return result;
        } else {
            let result = self.last;
            self.last = *self.last as *mut usize;
            return result as _;
        }
    }

    pub unsafe fn deallocate(&mut self, stack_top : usize) {
        let stack = stack_top - PAGE_SIZE;
        let address = stack as *mut usize;
        *address = self.last as usize;
        self.last = address as _;
    }
}
