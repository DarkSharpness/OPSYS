use core::sync::atomic::AtomicUsize;
extern crate alloc;
use alloc::collections::BTreeMap;

use super::Process;

#[derive(Clone)]
pub struct PidType(usize);

static mut PID_MAP : BTreeMap<usize, * mut Process> = BTreeMap::new();

impl PidType {
    pub const fn new(pid : usize) -> Self {
        assert!(pid != 0);
        PidType(pid)
    }
    pub const fn bits(&self) -> usize {
        assert!(self.0 != 0);
        self.0
    }
    pub fn allocate() -> Self {
        unsafe { allocate() }
    }
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
    let result = PID_MAP.insert(process.get_pid().bits(), process);
    assert!(result.is_none(), "PID {} is already registered", process.get_pid().bits());
}

/** Unregister the process from the pid map. */
unsafe fn unregister_process(process : &mut Process) {
    let result = PID_MAP.remove(&process.get_pid().bits());
    assert!(result.is_some(), "PID {} is not registered", process.get_pid().bits());
}

/** Get the process from the pid map. */
unsafe fn pid_to_process(pid : &PidType) -> * mut Process {
    return *PID_MAP.get(&pid.bits()).unwrap();
}
