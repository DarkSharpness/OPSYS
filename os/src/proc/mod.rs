mod cpu;
mod pid;
mod elf;
mod proc;
mod test;
mod context;
mod schedule;

pub use cpu::*;
pub use proc::{Process, ProcessStatus};
pub use pid::PidType;
pub use schedule::{run_process, init_process};

use crate::cpu::CPU;
use context::Context;
use schedule::ProcessManager;

extern crate alloc;

extern "C" { fn switch_context(old : *mut Context, new : *mut Context); }

/** Run the process. */
unsafe fn run(process : *mut Process) {
    assert_eq!((*process).get_status(), ProcessStatus::RUNNABLE);
    (*process).set_status(ProcessStatus::RUNNING);
}

/** Run the process. */
unsafe fn off(process : *mut Process) {
    if (*process).has_status(ProcessStatus::RUNNING) {
        (*process).set_status(ProcessStatus::RUNNABLE);
    }
}

impl CPU {
    /** Switch from current process to new process. Timer is untouched. */
    pub unsafe fn switch_to(&mut self, new : *mut Process) {
        let old = self.get_process();
        off(old); run(new);
        switch_context((*old).get_context(), (*new).get_context());
    }

    /** Switch from current process to the scheduler. Timer is reset. */
    pub unsafe fn process_yield(&mut self) {
        self.reset_timer_time();
        let old = self.get_process();
        off(old);
        switch_context((*old).get_context(), self.get_context());
    }

    /** Switch from scheduler to the new process. Timer is untouched. */
    pub unsafe fn scheduler_yield(&mut self, new : *mut Process) {
        assert_eq!((*new).get_status(), ProcessStatus::RUNNABLE);
        run(new);
        switch_context(self.get_context(), (*new).get_context());
    }
}
