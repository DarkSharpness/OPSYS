use crate::cpu::CPU;

impl CPU {
    pub(super) unsafe fn exit_as(&mut self, status: usize) {
        use sys::syscall::*;
        if true { todo!() }

        self.service_request_block([status, 0, 0, PM_EXIT, PM_PORT]);
    }

    pub unsafe fn sys_fork(&mut self){
        let child       = self.fork();
        let child_pid   = child.get_pid().raw_bits();

        use sys::syscall::*;
        self.service_request_block([child_pid, 0, 0, PM_FORK, PM_PORT]);
    }

    pub unsafe fn sys_exit(&mut self){
        let process     = self.get_process();
        let trap_frame  = (*process).get_trap_frame();
        return self.exit_as(trap_frame.a0);
    }

    pub unsafe fn sys_wait(&mut self){
        let process     = self.get_process();
        let trap_frame  = (*process).get_trap_frame();

        use sys::syscall::*;
        self.service_request_block([trap_frame.a0, 0, 0, PM_WAIT, PM_PORT]);
    }
}
