mod argv;
mod handle;
mod service;
mod request;

extern crate alloc;
pub use argv::Argument;
use request::Request;
use service::Service;

use crate::{alloc::PTEFlag, proc::{Process, ProcessStatus}};

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
        if request.forward(self) {
            service.pop_front();
        }
    }

    pub unsafe fn address_check(&mut self, args : &[usize], permission : PTEFlag) {
        if args[2] != 0 {
            if !self.get_satp().check_ptr(args[0], args[1], permission) {
                self.exit_as(1);
            }
        }
    }
}
