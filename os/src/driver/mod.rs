use core::arch;

pub mod uart;
pub mod start;

core::arch::global_asm!(include_str!("driver.asm"));

// Get the thread ID.
#[inline(always)]
pub fn get_tid() -> usize {
    let tid : usize;
    unsafe { arch::asm!("mv {}, tp", out(reg) tid); }
    return tid;
}

#[inline(always)]
pub fn get_mem_end() -> usize {
    let mem_end : usize;
    unsafe { arch::asm!("mv {}, gp", out(reg) mem_end); }
    return mem_end;
}
