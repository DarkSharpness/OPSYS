pub mod uart;
mod start;
mod timer;

pub use start::init;
use core::arch::{global_asm, asm};

global_asm!(include_str!("driver.asm"));

#[inline(always)]
pub fn get_tid() -> usize {
    let tid : usize;
    unsafe { asm!("mv {}, tp", out(reg) tid); }
    return tid;
}

#[inline(always)]
pub fn get_mem_end() -> usize {
    let mem_end : usize;
    unsafe { asm!("mv {}, gp", out(reg) mem_end); }
    return mem_end;
}

#[inline(never)]
pub unsafe fn shutdown() {
    warning!("Shutting down the machine...");
    let pos = 0x100000 as * mut u32;
    pos.write_volatile(0x5555);
}
