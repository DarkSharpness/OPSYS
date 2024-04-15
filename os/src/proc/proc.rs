extern crate alloc;

use alloc::collections::VecDeque;
use alloc::str;
use crate::driver::get_tid;
use crate::layout::*;

use crate::alloc::PageAddress;

pub struct Context {
    ra  : u64,
    sp  : u64,
    saved_registers : [u64; 12],
}

type PidType = u64;

pub enum ProcessStatus  {
    SLEEPING, // blocked
    RUNNABLE, // ready to run, but not running
    RUNNING, // running on CPU
    ZOMBIE, // exited but have to be waited by parent
    DEAD    // exited and no need to be waited by parent
}

pub struct Process {
    pub pid         : PidType,          // process id
    pub exit_code   : i32,              // exit code
    pub status      : ProcessStatus,    // process status
    pub root        : PageAddress,      // root of the page table
    pub parent      : * mut Process,    // parent process
    pub context     : * mut Context,    // current context
    pub name        : &'static str,     // process name
}

pub struct ProcessManager {
    pub process_queue   : VecDeque<Process>,
    pub running_process : * mut Process,
    pub batch_iter      : usize,
    pub batch_size      : usize,
}

static mut MANAGER : [ProcessManager; NCPU] = [
    ProcessManager {
        process_queue   : VecDeque::new(),
        running_process : core::ptr::null_mut(),
        batch_iter      : 0,
        batch_size      : 0,
    }; NCPU];

static mut CONTEXT : [Context; NCPU] = [
    Context {
        ra              : 0,
        sp              : 0,
        saved_registers : [0; 12],
    }; NCPU];

const TEST_PROGRAM0 : [u32; 4] = [
    0x10000537, // lui a0,0x10000
    0x0310059b, // addiw a1,zero,0x31
    0x00b50023, // sb a1,0(a0)
    0x0000bfd5, // j 0
];

pub unsafe fn current_process() -> *mut Process {
    let manager = get_manager();
    return manager.running_process;
}

pub unsafe fn init_process() {
    // let manager = get_manager();
    // manager.process_queue.push_back();

    todo!();
}


impl Process {
    pub fn new(name : &'static str, parent : * mut Process) -> Process {
        let root = PageAddress::new_pagetable();
        return Process {
            pid         : 0,
            exit_code   : 0,
            status      : ProcessStatus::RUNNABLE,
            context     : core::ptr::null_mut(),
            name, root, parent,
        };
    }
}


/**
 * Return the current thread's manager.
 */
#[inline(always)]
pub unsafe fn get_manager() -> &'static mut ProcessManager {
    let tid = get_tid();
    return &mut MANAGER[tid];
}

/**
 * Return the context pointer of the current thread.
 */
#[inline(always)]
pub unsafe fn get_context() -> *mut Context {
    let tid = get_tid();
    return &mut CONTEXT[tid];
}
