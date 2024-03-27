#![allow(unused_imports)]
mod user;
mod kernel;

pub use user::*;
pub use kernel::*;

use riscv::register::*;
use crate::layout::TRAMPOLINE;

core::arch::global_asm!(include_str!("trap.asm"));

extern "C" {
    fn core_handle();
    
    fn user_handle();
    fn user_handle_end();
    
    fn user_return();
    fn user_return_end();
    
    fn return_to_user(satp : usize);
}

pub unsafe fn init_trap() {
    // Set the trap vector to the supervisor vector.
    set_kernel_trap();

    // Copy the trampoline page to given region
    // This is used to initialize the trampoline page for user.
    copy_trampoline();
}

unsafe fn copy_trampoline() {
    let mut trampoline = TRAMPOLINE as * mut u64;
    let mut handle_beg = user_handle as * const u64;
    let     handle_end = user_handle_end as * const u64;

    while handle_beg < handle_end {
        trampoline.write(handle_beg.read());
        trampoline = trampoline.add(1);
        handle_beg = handle_beg.add(1);
    }

    trampoline = (TRAMPOLINE + 0x800) as _;

    let mut return_beg = user_return as * const u64;
    let     return_end = user_return_end as * const u64;

    while return_beg < return_end {
        trampoline.write(return_beg.read());
        trampoline = trampoline.add(1);
        return_beg = return_beg.add(1);
    }
}

#[inline(always)]
unsafe fn set_kernel_trap() {
    stvec::write(core_handle as usize, stvec::TrapMode::Direct);
}
