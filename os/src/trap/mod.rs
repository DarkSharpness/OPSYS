mod user;
mod kernel;
mod frame;

pub use user::*;
pub use frame::TrapFrame;

use riscv::register::*;
use crate::alloc::{PTEFlag, PageAddress, PAGE_SIZE};

core::arch::global_asm!(include_str!("trap.asm"));

const TRAMPOLINE : usize = (0 as usize).wrapping_sub(PAGE_SIZE);
pub const TRAP_FRAME : usize = TRAMPOLINE - (PAGE_SIZE as usize);

extern "C" {
    fn core_handle();
    
    fn user_handle();
    fn user_handle_end();
    
    fn user_return(satp : usize);
    fn user_return_end();
}

#[inline(always)]
unsafe fn set_kernel_trap() {
    stvec::write(core_handle as _, stvec::TrapMode::Direct);
}
#[inline(always)]
unsafe fn set_user_trap() {
    stvec::write(TRAMPOLINE  as _, stvec::TrapMode::Direct);
}

/** Return the trampoline physical address */
unsafe fn get_trampoline_physical() -> PageAddress {
    return PageAddress::new_usize(user_handle as _)
}

impl PageAddress {
    pub unsafe fn map_trampoline(self) {
        self.smap(TRAMPOLINE, get_trampoline_physical(), PTEFlag::RX | PTEFlag::OTHER);
    }
}

pub struct Interrupt;

impl Interrupt {
    pub unsafe fn enable()  { sstatus::set_sie();   }
    pub unsafe fn disable() { sstatus::clear_sie(); }
}
