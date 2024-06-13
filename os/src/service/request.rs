use sys::syscall::{ARGS_BUFFERED, ARGS_REGISTER};

use crate::{alloc::PTEFlag, proc::Process, utility::SliceIter};

use super::{argv::Argument, handle::ServiceHandle};

pub struct Request {
    args    : Argument,         // Arguments
    kind    : usize,            // What kind of service?
    handle  : ServiceHandle,    // The callback handle
}

impl Request {
    pub unsafe fn new(args : Argument, kind : usize, process : &mut Process) -> Self {
        let handle = ServiceHandle::from_process(process);
        return Self { kind, args, handle };
    }

    /**
     * Try to forward a request to %target process.
     * Return whether the request can be forwarded.
    */
    pub unsafe fn try_forward(&mut self, target : &mut Process) -> bool {
        let trap_frame = target.get_trap_frame();
        match &mut self.args {
            Argument::Register(a0, a1) => {
                trap_frame.a0 = *a0;
                trap_frame.a1 = *a1;
                trap_frame.a2 = ARGS_REGISTER;
                trap_frame.a4 = self.kind;
                trap_frame.a5 = self.handle.bits();
            },
            Argument::Buffered(buffer) => {
                // Not enough space to write, so return false.
                if trap_frame.a2 != ARGS_BUFFERED || trap_frame.a1 < buffer.len() {
                    trap_frame.a4 = self.kind;
                    trap_frame.a5 = buffer.len().wrapping_neg();
                    return false;
                }

                trap_frame.a1 = buffer.len();
                trap_frame.a2 = ARGS_BUFFERED;
                trap_frame.a4 = self.kind;
                trap_frame.a5 = self.handle.bits();

                let buf = trap_frame.a0;
                let len = trap_frame.a1;

                target.address_check([buf, len], PTEFlag::WO);
                target.get_satp().core_to_user(buf, len, SliceIter::new(buffer));
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
