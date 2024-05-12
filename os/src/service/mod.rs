extern crate alloc;
use core::usize;

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
enum Argument {
    Nothing,                    // No arguments in
    Pointer(*mut u8, usize),    // In a user buffer.
    Buffered(*mut u8, usize),   // In a kernel buffer.
    Register(usize, usize)      // In 2 registers
}

#[derive(Clone, Copy)]
struct Request {
    kind    : usize,        // What kind of service?
    args    : Argument,     // Arguments
    handle  : ServiceHandle,// The callback handle
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
    pub unsafe fn request_service(&mut self, port : usize,
        kind : usize, handle : ServiceHandle, args : [usize; 3], block : bool) {
        let service = &mut SERVICE[port];
        let request = Request {
            kind, handle,
            args    : Argument::new(args),
        };
        service.waiting.push_back(request);

        let servant = (*service).servant;
        if block {
            return self.blocking_request(servant);
        } else {
            return self.nonblocking_request(servant);
        }
    }

    unsafe fn blocking_request(&mut self, servant : *mut Process) {
        let process = self.get_process();
        (*process).sleep_as(ProcessStatus::SERVICE);
        if !servant.is_null() {
            (*servant).wake_up_from(ProcessStatus::SERVICE);
            self.switch_to(servant);
        } else {
            self.sys_yield();
        }
    }

    unsafe fn nonblocking_request(&mut self, servant : *mut Process) {
        if !servant.is_null() {
            (*servant).wake_up_from(ProcessStatus::SERVICE);
        }
    }

    pub unsafe fn accept_service(&mut self, port : usize) {
        let process = self.get_process();
        let service = &mut SERVICE[port];
        assert!(service.servant.is_null(), "Service already accepted");
        match service.waiting.pop_front() {
            Some(request) => {
                (*process).status = ProcessStatus::RUNNING;

                let trap_frame = &mut *(*process).trap_frame;

                trap_frame.a4 = request.kind;
                trap_frame.a5 = request.handle;

                todo!("Accept service");
            },
            None => {
                service.servant = process;
                (*process).sleep_as(ProcessStatus::SERVICE);
                self.sys_yield();
            }
        }
    }

    pub unsafe fn response_service(&mut self, _handle : ServiceHandle) {
        todo!("Response service");
    }
}

impl Argument {
    pub fn new(_args : [usize ; 3]) -> Self {
        todo!("Parsing argument");
        // match args[2] {
        //     0 => Argument::Nothing,
        //     1 => Argument::Pointer(args[0] as *mut u8, args[1] as usize),
        //     2 => Argument::Register(args[0], args[1]),
        //     _ => panic!("Invalid argument type")
        // }
    }
}
