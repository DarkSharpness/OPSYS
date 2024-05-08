mod proc;
mod schedule;

pub use proc::init_process;
pub use schedule::run_process;

extern crate alloc;
use alloc::collections::VecDeque;
use crate::{alloc::PageAddress, cpu::CPU, trap::TrapFrame};

extern "C" { fn switch_context(old : *mut Context, new : *mut Context); }

#[repr(C)]
pub struct Context { stack_bottom : usize, }

pub type PidType = usize;

#[derive(Debug, PartialEq)]
pub enum ProcessStatus {
    RUNNING,    // running on CPU
    SERVICE,    // waiting for some service
    RUNNABLE,   // ready to run, but not running
    SLEEPING,   // blocked by some event
    ZOMBIE,     // exited but have to be waited by parent
    DEAD,       // exited and no need to be waited by parent
}

pub struct Process {
    pub pid         : PidType,          // process id
    pub exit_code   : i32,              // exit code
    pub status      : ProcessStatus,    // process status
    pub root        : PageAddress,      // root of the page table
    pub parent      : * mut Process,    // parent process
    pub trap_frame  : * mut TrapFrame,  // trap frame
    pub name        : &'static str,     // process name
    pub context     : Context,          // current context
}

pub struct ProcessManager {
    pub process_queue   : VecDeque<Process>,
    pub running_process : * mut Process,
    pub batch_iter      : usize,
    pub batch_size      : usize,
}

impl CPU {
    /** Switch from current process to new process. */
    pub unsafe fn switch_to(&mut self, new : *mut Process) {
        let old = self.get_process();
        switch_context((*old).get_context(), (*new).get_context());
    }

    /** Switch from current process to the scheduler. */
    pub unsafe fn process_yield(&mut self) {
        let old = self.get_process();
        switch_context((*old).get_context(), self.get_context());
    }

    /** Switch from scheduler to the new process. */
    pub unsafe fn scheduler_yield(&mut self, new : *mut Process) {
        switch_context(self.get_context(), (*new).get_context());
    }
}
