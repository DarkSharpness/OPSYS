#![allow(unused_imports)]
mod user;
mod kernel;

pub use user::*;
pub use kernel::*;

use riscv::register::*;

core::arch::global_asm!(include_str!("trap.asm"));

extern "C" {
    fn core_handle();
    
    fn user_handle();
    fn user_handle_end();
    
    fn user_return();
    fn user_return_end();
    
    fn return_to_user(satp : usize);
}

#[inline(always)]
unsafe fn set_kernel_trap() {
    stvec::write(core_handle as usize, stvec::TrapMode::Direct);
}
