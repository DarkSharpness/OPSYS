use crate::{alloc::PTEFlag, cpu::CPU, service::ServiceHandle};

impl CPU {
    /** Reset the timer and yield to another process. */
    pub unsafe fn sys_yield(&mut self) { return self.process_yield(); }

    /**
     * A blocking request sent by a trusted process to the kernel.
     * A request may be redirected to another process or kernel.
     * This process will continue to run after the request is processed.
     */
    pub unsafe fn sys_request(&mut self) {
        let process = self.get_process();
        let trap_frame  = (*process).get_trap_frame();

        let call_fn = if trap_frame.a5 == 0 {
            Self::service_request_block
        } else {
            Self::service_request_async
        };

        call_fn(self, [trap_frame.a0, trap_frame.a1, trap_frame.a2,
                       trap_frame.a4, trap_frame.a6]);
    }

    /**
     * A blocking accept sent by a trusted process to the kernel.
     * Only one process can accept one certain request.
     */
    pub unsafe fn sys_receive(&mut self) {
        let process     = self.get_process();
        let trap_frame  = (*process).get_trap_frame();
        let port        = (*trap_frame).a6;
        return self.service_receive(port);
    }

    /**
     * A response sent by a trusted process to the kernel.
     * This will send the response to the handle, which is the caller
     * of the request. After the reponse, the caller will continue to run.
     */
    pub unsafe fn sys_respond(&mut self) {
        let process     = self.get_process();
        let trap_frame  = (*process).get_trap_frame();
        let handle      = ServiceHandle::new(trap_frame.a5);
        return self.service_respond(handle);
    }

    pub unsafe fn address_check(&mut self, args : &[usize], flag : PTEFlag) {
        if args[2] != 0 {
            let process = &mut *self.get_process();
            if !process.get_satp().check_ptr(args[0], args[1], flag) {
                self.exit_as(1);
            }
        }
    }
}
