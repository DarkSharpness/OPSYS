mod argv;
mod handle;
mod service;
mod request;

extern crate alloc;
pub use handle::ServiceHandle;
use request::Request;
use service::Service;

use crate::{cpu::CPU, proc::ProcessStatus};

const MAX_SERVICE : usize = 16;
const ARRAY_REPEAT_VALUE: Service = Service::new();
static mut SERVICE : [Service; MAX_SERVICE] = [ARRAY_REPEAT_VALUE; MAX_SERVICE];

impl CPU {
    pub unsafe fn service_receive(&mut self, port : usize) {
        let service = &mut SERVICE[port];
        let request = service.wait_for_request(self);
        if request.forward(&mut *self.get_process()) {
            service.pop_front();
        } else { // Set as failed
        }
    }

    pub unsafe fn service_respond(&mut self, handle : ServiceHandle) {
        if handle.is_async() { return; }
        let process = handle.to_process();
        assert!(!process.is_null(), "Invalid handle");
        (*process).wake_up_from(ProcessStatus::SERVICE);
    }

    pub unsafe fn service_request_block(&mut self, args : [usize; 5]) {
        let process = &mut *self.get_process();
        let port    = args[4];
        let service = &mut SERVICE[port];

        process.sleep_as(ProcessStatus::SERVICE);
        service.push_back(Request::new_block(&args, process));

        match service.try_wake_up_servant() {
            Some(process)   => self.switch_to(process),
            None            => self.sys_yield()
        }
    }

    pub unsafe fn service_request_async(&mut self, args : [usize; 5]) {
        let process = &mut *self.get_process();
        let port    = args[4];
        let service = &mut SERVICE[port];

        service.push_back(Request::new_async(&args, process));
        service.try_wake_up_servant();
    }
}
