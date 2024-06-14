mod argv;
mod handle;
mod service;
mod request;

extern crate alloc;
pub use argv::Argument;
use handle::ServiceHandle;
use request::Request;
use service::Service;

use crate::proc::{Process, ProcessStatus};

const MAX_SERVICE : usize = 16;
const ARRAY_REPEAT_VALUE: Service = Service::new();
static mut SERVICE : [Service; MAX_SERVICE] = [ARRAY_REPEAT_VALUE; MAX_SERVICE];

impl Process {
    pub unsafe fn service_request(&mut self, args : Argument, kind : usize, port : usize) {
        let service = &mut SERVICE[port];
        self.sleep_as(ProcessStatus::SERVICE);
        service.push_back(Request::new(args, kind, self));

        match service.try_wake_up_servant() {
            Some(process)   => self.yield_to_process(&mut *process),
            None            => self.yield_to_scheduler(),
        }
    }

    pub unsafe fn service_receive(&mut self, port : usize) {
        let service = &mut SERVICE[port];
        let request = service.wait_for_request(self);
        if request.try_forward(self) {
            service.pop_front();
        }
    }

    pub unsafe fn service_respond(&mut self, args : Argument, handle : usize) {
        let handle = ServiceHandle::new(handle);
        let target = &mut *handle.to_process();
        target.set_response(args);
        target.wake_up_from(ProcessStatus::SERVICE);
        self.yield_to_process(target);
    }
}
