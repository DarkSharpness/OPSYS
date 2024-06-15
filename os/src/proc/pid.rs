use core::{ptr::null_mut, sync::atomic::AtomicUsize};
extern crate alloc;
use alloc::vec::Vec;

use super::Process;

#[derive(Clone)]
pub struct PidType(usize);

static mut PID_MAP  : Vec<* mut Process> = Vec::new();

impl PidType {
    pub fn new(pid : usize) -> Self { Self(pid) }
    pub fn bits(&self) -> usize { self.0 }
    pub unsafe fn to_process(&self) -> * mut Process {
        let process = pid_to_process(self);
        assert!(!process.is_null());
        return process;
    }
    pub unsafe fn register(process : &mut Process) {
        register_process(process);
    }
    pub unsafe fn unregister(process : &mut Process) {
        unregister_process(process);
    }
}

unsafe fn allocate() -> PidType {
    static mut PID_POOL : AtomicUsize = AtomicUsize::new(0);
    let value = PID_POOL.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
    return PidType::new(value + 1); // Pid = 0 is reserved.
}

/** Register the process to the pid map. */
unsafe fn register_process(process : &mut Process) {
    process.set_pid(allocate());
    PID_MAP.push(process);
    assert!(PID_MAP.len() == process.get_pid().bits());
}

/** Unregister the process from the pid map. */
unsafe fn unregister_process(process : &mut Process) {
    PID_MAP[process.get_pid().bits() - 1] = null_mut();
}

/** Get the process from the pid map. */
unsafe fn pid_to_process(pid : &PidType) -> * mut Process {
    return PID_MAP[pid.bits() - 1];
}
