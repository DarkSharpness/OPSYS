#![allow(unused_imports)]
mod user;
mod kernel;
mod frame;

pub use user::user_trap;
pub use frame::TrapFrame;

use riscv::register::*;
use crate::alloc::{vmmap, PTEFlag, PageAddress, PAGE_SIZE};

core::arch::global_asm!(include_str!("trap.asm"));

pub const TRAMPOLINE : u64 = (!PAGE_SIZE + 1) as u64;

extern "C" {
    fn core_handle();
    
    fn user_handle();
    fn user_handle_end();
    
    fn user_return(x : usize);
    fn user_return_end();
}

#[inline(always)]
unsafe fn set_kernel_trap() {
    stvec::write(core_handle as usize, stvec::TrapMode::Direct);
}

/**
 * Set the trampoline for kernel or user.
 * In every case, trampoline is only executable in supervisor mode,
 * so the U bit in PTE should be set to 0.
 */
pub unsafe fn set_trampoline(root : PageAddress) {
    // Trampoline is laid at [-4096, 0)
    vmmap(root, TRAMPOLINE, user_handle as _, PTEFlag::RX);
}

pub struct Interrupt;

impl Interrupt {
    pub unsafe fn enable()  { sstatus::set_sie();   }
    pub unsafe fn disable() { sstatus::clear_sie(); }
}

