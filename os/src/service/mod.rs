extern crate alloc;
use alloc::collections::VecDeque;

use crate::{cpu::CPU, proc::{Process, ProcessStatus}};

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

#[derive(Clone, Copy)]
struct Request {
    kind    : usize,        // What kind of service?
    handle  : ServiceHandle // The callback handle
}

struct Service {
    servant : *mut Process,     // Who is accepting?
    waiting : VecDeque<Request> // Pending requests
}

const MAX_SERVICE : usize = 16;
const ARRAY_REPEAT_VALUE: Service = Service::new();
static mut SERVICE : [Service; MAX_SERVICE] = [ARRAY_REPEAT_VALUE; MAX_SERVICE];

impl Service {
    const fn new() -> Self {
        Service {
            servant : core::ptr::null_mut(),
            waiting : VecDeque::new()
        }
    }
}

impl CPU {
    pub unsafe fn request_service(&mut self,
        port : usize, kind : usize, handle : ServiceHandle) {
        let service = &mut SERVICE[port];
        let request = Request { kind, handle };
        service.waiting.push_back(request);
        let servant = (*service).servant;
        if !servant.is_null() {
            self.switch_to(servant);
        } else {
            self.process_yield();
        }
    }

    pub unsafe fn accept_service(&mut self, port : usize) {
        let process = self.get_process();
        let service = &mut SERVICE[port];
        assert!(service.servant.is_null(), "Service already accepted");
        match service.waiting.pop_front() {
            Some(_request) => {
                todo!("Accept service");
            },
            None => {
                service.servant = process;
                self.process_yield();
            }
        }
    }

    pub unsafe fn response_service(&mut self, _handle : ServiceHandle) {
        todo!("Response service");
    }
}
