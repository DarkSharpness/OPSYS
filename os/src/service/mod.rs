use crate::proc::{Process, ProcessStatus};

pub type ServiceHandle = usize;

impl Process {
    pub fn new_service(&mut self) -> ServiceHandle {
        assert_eq!(self.status, ProcessStatus::RUNNING, "Invalid process status");
        self.status = ProcessStatus::SERVICE;
        return self.pid;
    }
    pub fn end_service(&mut self, handle: ServiceHandle) {
        assert_eq!(handle, self.pid, "Invalid handle");
        assert_eq!(self.status, ProcessStatus::SERVICE, "Invalid process status");
        self.status = ProcessStatus::RUNNING;
    }
}
