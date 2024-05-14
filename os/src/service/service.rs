use core::ptr::null_mut;
use crate::{cpu::CPU, proc::ProcessStatus, service::handle_to_process};

use super::{Request, Service, ServiceHandle};

const MAX_SERVICE : usize = 16;
const ARRAY_REPEAT_VALUE: Service = Service::new();
static mut SERVICE : [Service; MAX_SERVICE] = [ARRAY_REPEAT_VALUE; MAX_SERVICE];

impl CPU {
    pub unsafe fn service_receive(&mut self, port : usize) {
        let process = &mut *self.get_process();
        let service = &mut SERVICE[port];
        assert!(service.servant.is_null(), "Service already accepted");
        while service.waiting.is_empty() {
            service.servant = process;
            (*process).sleep_as(ProcessStatus::SERVING);
            self.sys_yield();
        }

        service.servant = null_mut();
        let request = (service.waiting).front_mut().expect("WTF no request!");
        if request.forward(process) {
            service.waiting.pop_front();
        }
    }

    pub unsafe fn service_respond(&mut self, handle : ServiceHandle) {
        if handle.is_async() { return; }
        let process = handle_to_process(handle);
        assert!(!process.is_null(), "Invalid handle");
        (*process).wake_up_from(ProcessStatus::SERVICE);
    }

    pub unsafe fn service_request_block(&mut self, args : [usize; 5]) {
        let process = &mut *self.get_process();
        let port    = args[4];
        let service = &mut SERVICE[port];

        service.waiting.push_back(Request::new_block(&args[ 0..3 ], process));

        if !service.servant.is_null() {
            let process = &mut *service.servant;
            process.wake_up_from(ProcessStatus::SERVING);
            return self.switch_to(process);
        } else {
            return self.sys_yield();
        }
    }

    pub unsafe fn service_request_async(&mut self, args : [usize; 5]) {
        let process = &mut *self.get_process();
        let port    = args[4];
        let service = &mut SERVICE[port];

        service.waiting.push_back(Request::new_async(&args[ 0..3 ], process));

        if !service.servant.is_null() {
            let process = &mut *service.servant;
            process.wake_up_from(ProcessStatus::SERVING);
        }
    }
}
