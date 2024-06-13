use crate::{cpu::CPU, proc::Process, service::Argument};

impl Process {
    pub unsafe fn fork(&mut self) -> Process {
        let child       = Process::init();

        /* Syscall to make a new child. */
        use sys::syscall::*;
        let child_pid   = child.get_pid().raw_bits();
        self.service_request(Argument::Register(child_pid, 0), PM_FORK, PM_PORT);

        /* Return the arguments */
        let trap_frame  = self.get_trap_frame();
        child.get_trap_frame().copy_from(trap_frame);
        trap_frame.a0 = child.get_pid().raw_bits();
        child.get_trap_frame().a0 = 0;

        /* Copy the page take to children. */
        child.get_satp().copy_from(self.get_satp());

        return child;
    }

    pub unsafe fn exit_as(&mut self, status: usize) -> ! {
        use sys::syscall::*;
        self.service_request(Argument::Register(status, 0), PM_EXIT, PM_PORT);
        todo!("Implement exit")
    }
}

impl CPU {
    pub unsafe fn sys_yield(&mut self) {
        return self.process_yield();
    }

    pub(super) unsafe fn sys_fork(&mut self) {
        let process     = &mut *self.get_process();
        self.get_manager().add_process(process.fork());
    }

    pub(super) unsafe fn sys_exit(&mut self) -> ! {
        let process     = &mut *self.get_process();
        let trap_frame  = process.get_trap_frame();
        process.exit_as(trap_frame.a0);
    }

    pub(super) unsafe fn sys_wait(&mut self) {
        let process     = &mut *self.get_process();
        let trap_frame  = process.get_trap_frame();

        use sys::syscall::*;
        process.service_request(Argument::new_registers(trap_frame.a0, 0), PM_WAIT, PM_PORT);
    }
}
