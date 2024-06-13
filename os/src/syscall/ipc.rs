use crate::{cpu::CPU, service::Argument};

impl CPU {
    /**
     * A blocking request sent by a trusted process to the kernel.
     * A request may be redirected to another process or kernel.
     * This process will continue to run after the request is processed.
     */
    pub unsafe fn sys_request(&mut self) {
        todo!();
    }

    /**
     * A blocking accept sent by a trusted process to the kernel.
     * Only one process can accept one certain request.
     */
    pub unsafe fn sys_receive(&mut self) {
        let process     = &mut *self.get_process();
        let trap_frame  = process.get_trap_frame();
        let port        = trap_frame.a6;
        process.service_receive(port);
    }

    /**
     * A response sent by a trusted process to the kernel.
     * This will send the response to the handle, which is the caller
     * of the request. After the reponse, the caller will continue to run.
     */
    pub unsafe fn sys_respond(&mut self) {
        let process     = &mut *self.get_process();
        let trap_frame  = process.get_trap_frame();
        let arg_array   = [trap_frame.a0, trap_frame.a1, trap_frame.a2];
        let handle      = trap_frame.a5;
        let argument    = Argument::new(arg_array, process);
        process.service_respond(argument, handle);
    }
}
