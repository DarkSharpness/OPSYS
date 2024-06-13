use crate::proc::{Process, ProcessStatus};
extern crate alloc;
use alloc::collections::VecDeque;

use super::request::Request;

pub struct Service {
    servant : *mut Process,     // Who is accepting?
    waiting : VecDeque<Request> // Pending requests
}

impl Service {
    pub const fn new() -> Self {
        Service {
            servant : core::ptr::null_mut(),
            waiting : VecDeque::new(),
        }
    }

    unsafe fn set_servant(&mut self, process: *mut Process) {
        assert!(self.servant.is_null(), "Service already accepted");
        self.servant = process;
    }

    unsafe fn reset_servant(&mut self, process: *mut Process) {
        assert!(self.servant == process, "Invalid servant");
        self.servant = core::ptr::null_mut();
    }

    pub unsafe fn try_wake_up_servant(&self) -> Option<* mut Process> {
        if self.servant.is_null() { return None; }
        (*self.servant).wake_up_from(ProcessStatus::SERVING);
        return Some(self.servant);
    }

    pub unsafe fn wait_for_request(&mut self, process : &mut Process) -> &mut Request {
        self.set_servant(process);

        while self.waiting.is_empty() {
            process.sleep_as(ProcessStatus::SERVING);
            process.yield_to_scheduler();
        }

        self.reset_servant(process);

        return self.waiting.front_mut().expect("WTF no request!");
    }

    pub unsafe fn pop_front(&mut self) {
        self.waiting.pop_front();
    }

    pub unsafe fn push_back(&mut self, request: Request) {
        self.waiting.push_back(request);
    }
}
