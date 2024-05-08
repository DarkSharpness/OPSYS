use crate::cpu::CPU;

impl CPU {
    pub unsafe fn sys_yield(&mut self) {
        self.reset_timer_time();

        /* Switch back to previous content. */
        return self.process_yield();
    }

    /**
     * A blocking request sent by a trusted process to the kernel.
     * A request may be redirected to another process or kernel.
     * This process will continue to run after the request is processed.
     */
    pub unsafe fn sys_request(&mut self) {
        let process = self.get_process();
        let trap_frame = (*process).trap_frame;
        let kind    = (*trap_frame).a7;     // What service to request
        let port    = (*trap_frame).a6;     // Which port to request
        let handle  = (*process).new_service();     // Who call this syscall
        return self.request_service(port, kind, handle);
    }

    /**
     * A blocking accept sent by a trusted process to the kernel.
     * Only one process can accept one certain request.
     */
    pub unsafe fn sys_accept(&mut self) {
        let process = self.get_process();
        let trap_frame = (*process).trap_frame;
        let port = (*trap_frame).a6;    // Which port to accept
        return self.accept_service(port);
    }

    /**
     * A non-blocking transfer sent by a trusted process to the kernel.
     * This will transfer the request from handle, leaving it for the
     * target to accept and complete. The last response will be sent
     * from the target (or transfer again) to the handle.
     */
    pub unsafe fn sys_transfer(&mut self) {
        let process = self.get_process();
        let trap_frame = (*process).trap_frame;
        let kind    = (*trap_frame).a7; // What service to transfer
        let port    = (*trap_frame).a6; // Which port to transfer
        let handle  = (*trap_frame).a5; // Who call this syscall
        return self.request_service(port, kind, handle);
    }

    /**
     * A response sent by a trusted process to the kernel.
     * This will send the response to the handle, which is the caller
     * of the request. After the reponse, the caller will continue to run.
     */
    pub unsafe fn sys_response(&mut self) {
        let process = self.get_process();
        let trap_frame = (*process).trap_frame;
        let handle  = (*trap_frame).a7; // Who to response
        return self.response_service(handle);        
    }
}


