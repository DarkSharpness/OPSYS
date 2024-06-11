use crate::proc::Process;

use super::{argv::Argument, handle::ServiceHandle};

pub struct Request {
    kind    : usize,            // What kind of service?
    args    : Argument,         // Arguments
    handle  : ServiceHandle,    // The callback handle
}

impl Request {
    /** Create a new blocking request. */
    pub unsafe fn new_block(args : &[usize], process : &mut Process) -> Self {
        let kind = args[3];
        let args = Argument::new(args, process);
        let handle = ServiceHandle::from_process(process);
        return Self { kind, args, handle };
    }

    /** Create a new asynchronized request.  */
    pub unsafe fn new_async(args : &[usize], process : &mut Process) -> Self {
        let kind = args[3];
        let args = Argument::new(args, process);
        let handle = ServiceHandle::new_async();
        return Self { kind, args, handle };
    }

    /**
     * Try to forward a request to %target process.
     * Return whether the request can be forwarded.
    */
    pub unsafe fn forward(&mut self, target : &mut Process) -> bool {
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
            Argument::Upointer(_, _) => {
                // This is a zero-copy optimization.
                // But not implemented yet.
                let _caller = self.handle.clone().to_process();
                todo!("Implement zero-copy optimization");
            }
        }
        return true;
    }
}
