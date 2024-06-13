use crate::cpu::CPU;

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
        todo!();
    }

    /**
     * A response sent by a trusted process to the kernel.
     * This will send the response to the handle, which is the caller
     * of the request. After the reponse, the caller will continue to run.
     */
    pub unsafe fn sys_respond(&mut self) {
        todo!()
    }
}
