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
pub const TRAP_FRAME : u64 = TRAMPOLINE - (PAGE_SIZE as u64);

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

pub unsafe fn get_trampoline() -> PageAddress {
    return PageAddress::new_u64(user_handle as _)
}

pub struct Interrupt;

impl Interrupt {
    pub unsafe fn enable()  { sstatus::set_sie();   }
    pub unsafe fn disable() { sstatus::clear_sie(); }
}

