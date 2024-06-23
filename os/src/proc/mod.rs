mod cpu;
mod pid;
mod elf;
mod proc;
mod test;
mod memory;
mod context;
mod manager;
mod schedule;

pub use cpu::*;
pub use proc::{Process, ProcessStatus};
pub use pid::PidType;
pub use manager::run_process;

use context::Context;
use manager::ProcessManager;

use crate::alloc::{PTEFlag, KERNEL_SATP, PAGE_SIZE};

pub unsafe fn init_process() {
    // Add trampoline to the page table
    KERNEL_SATP.map_trampoline();
    KERNEL_SATP.new_smap((PAGE_SIZE * 3).wrapping_neg(), PTEFlag::RW);
    KERNEL_SATP.new_smap((PAGE_SIZE * 4).wrapping_neg(), PTEFlag::RW);

    let manager = current_cpu().get_manager();

    manager.insert_process(Process::new_test(0));
    manager.insert_process(Process::new_test(1));
}
