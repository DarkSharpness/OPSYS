mod cpu;
mod pid;
mod elf;
mod proc;
mod test;
mod memory;
mod context;
mod schedule;

pub use cpu::*;
pub use proc::{Process, ProcessStatus};
pub use pid::PidType;
pub use schedule::run_process;

use context::Context;
use schedule::ProcessManager;

use crate::alloc::KERNEL_SATP;

pub unsafe fn init_process() {
    // Add trampoline to the page table
    KERNEL_SATP.map_trampoline();

    let manager = current_cpu().get_manager();

    manager.insert_process(Process::new_test(0));
    manager.insert_process(Process::new_test(1));
}
