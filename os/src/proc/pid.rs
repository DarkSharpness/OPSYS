use core::{ptr::null_mut, sync::atomic::AtomicUsize};
extern crate alloc;
use alloc::vec::Vec;

use super::Process;

#[derive(Clone)]
pub struct PidType(usize);

static mut PID_POOL : AtomicUsize = AtomicUsize::new(0);
static mut PID_MAP  : Vec<* mut Process> = Vec::new();

impl PidType {
    pub fn new(pid : usize) -> Self { Self(pid) }
    pub fn raw_bits(&self) -> usize { self.0 }
    pub unsafe fn allocate() -> Self {
        let value = PID_POOL.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        return PidType::new(value);
    }
    pub unsafe fn to_process(&self) -> * mut Process {
        return PID_MAP[self.raw_bits()];
    }
    pub unsafe fn register(process : & mut Process) {
        register_process(process);
    }
    pub unsafe fn unregister(process : & mut Process) {
        unregister_process(process);
    }
}

/** Register the process to the pid map. */
unsafe fn register_process(process : & mut Process) {
    assert!(PID_MAP.len() == process.get_pid().raw_bits());
    PID_MAP.push(process);
}

/** Unregister the process from the pid map. */
unsafe fn unregister_process(process : & mut Process) {
    PID_MAP[process.get_pid().raw_bits()] = null_mut();
}
