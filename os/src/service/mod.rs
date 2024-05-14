mod service;

extern crate alloc;
use alloc::{boxed::Box, collections::VecDeque, vec::Vec};

use crate::proc::{pid_to_process, PidType, Process, ProcessStatus};

pub(crate) struct ServiceHandle(usize);

enum Argument {
    Register(usize, usize),     // In 2 registers.
    Buffered(Box<[u8]>),          // In a kernel buffer.
    Pointer(*mut u8, usize),    // In a user buffer.
}

struct Request {
    kind    : usize,            // What kind of service?
    args    : Argument,         // Arguments
    handle  : ServiceHandle,    // The callback handle
}

struct Service {
    servant : *mut Process,     // Who is accepting?
    waiting : VecDeque<Request> // Pending requests
}

impl Request {
    unsafe fn new_block(args : &[usize], process : &mut Process) -> Self {
        process.sleep_as(ProcessStatus::SERVICE);
        let kind = args[3];
        let args = Argument::new(args, process);
        let handle = process_to_handle(process);
        return Self { kind, args, handle };
    }

    unsafe fn new_async(args : &[usize], process : &mut Process) -> Self {
        let kind = args[3];
        let args = Argument::new(args, process);
        let handle = ServiceHandle::new_async();
        return Self { kind, args, handle };
    }

    unsafe fn forward(&mut self, target : &mut Process) -> bool {
        let trap_frame = &mut *target.trap_frame;
        match &mut self.args {
            Argument::Register(a0, a1) => {
                trap_frame.a0 = *a0;
                trap_frame.a1 = *a1;
                trap_frame.a2 = 0;
                trap_frame.a4 = self.kind;
                trap_frame.a5 = self.handle.bits();
            },
            Argument::Buffered(buffer) => {
                assert!(trap_frame.a2 == 1, "Invalid register value");

                trap_frame.a4 = self.kind;
                if trap_frame.a1 < buffer.len() {
                    trap_frame.a1 = 0;
                    trap_frame.a5 = buffer.len().wrapping_neg();
                } else {
                    trap_frame.a1 = buffer.len();
                    trap_frame.a5 = self.handle.bits();
                    return false;
                }

                target.root.core_to_user(trap_frame.a0, buffer.len() , buffer);
            },
            Argument::Pointer(_, _) => {
                // This is a zero-copy optimization.
                // But not implemented yet.
                todo!("Copy user buffer to user");
            }
        }
        return true;
    }
}

impl Argument {
    unsafe fn new(args : &[usize], process : &mut Process) -> Self {
        match args[2] {
            0 => {
                Self::Register(args[0], args[1])
            },
            1 => {
                let mut tmp : Vec<u8> = Vec::new();
                tmp.resize(args[1], 0);

                let mut dst = tmp.into_boxed_slice();
                process.root.user_to_core(&mut dst, args[0], args[1]);
                Self::Buffered(dst)
            },
            _ => panic!("Invalid argument"),
        }
    }
}


impl ServiceHandle {
    pub fn new(size : usize) -> Self { return Self(size); }
    pub fn bits(&self) -> usize { return self.0; }
    fn new_async() -> Self { return Self(0); }
    fn is_async(&self) -> bool { return self.0 == 0; }
}

/// Now, handle = pid + MAGIC
const MAGIC : usize = 1919;

unsafe fn process_to_handle(process : *mut Process) -> ServiceHandle {
    let pid = &(*process).pid;
    return ServiceHandle::new(pid.bits() + MAGIC);
}

unsafe fn handle_to_process(handle : ServiceHandle) -> *mut Process {
    let pid = PidType::new(handle.bits() - MAGIC);
    return pid_to_process(pid);
}

impl Service {
    const fn new() -> Self {
        Service {
            servant : core::ptr::null_mut(),
            waiting : VecDeque::new(),
        }
    }
}
