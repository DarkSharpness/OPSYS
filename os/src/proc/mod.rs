mod proc;
mod schedule;

pub use proc::init_process;
pub use schedule::run_process;

extern crate alloc;
use alloc::collections::VecDeque;
use crate::{alloc::PageAddress, trap::TrapFrame};

extern "C" { pub fn switch_context(x : *mut Context, y : *mut Context); }

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
