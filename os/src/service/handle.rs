use crate::proc::{PidType, Process};

#[derive(Clone)]
pub struct ServiceHandle(usize);

impl ServiceHandle {
    pub fn new(size : usize) -> Self { return Self(size); }
    pub(super) fn bits(&self) -> usize { return self.0; }
    pub(super) fn new_async() -> Self { return Self(0); }
    pub(super) fn is_async(&self) -> bool { return self.0 == 0; }
    pub(super) unsafe fn to_process(self) -> *mut Process {
        return handle_to_process(self);
    }
    pub(super) unsafe fn from_process(process : *mut Process) -> Self {
        return process_to_handle(process);
    }
}

use sys::syscall::{pid_to_handle,handle_to_pid};

unsafe fn process_to_handle(process : *mut Process) -> ServiceHandle {
    let pid = &(*process).get_pid();
    return ServiceHandle::new(pid_to_handle(pid.bits()));
}

unsafe fn handle_to_process(handle : ServiceHandle) -> *mut Process {
    let pid = PidType::new(handle_to_pid(handle.bits()));
    return pid.to_process();
}
