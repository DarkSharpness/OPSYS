use crate::{alloc::PTEFlag, cpu::CPU, proc::{current_cpu, Process}, service::Argument};

impl Process {
    pub unsafe fn address_check(&mut self, args : [usize; 2], permission : PTEFlag) {
        if !self.get_satp().check_ptr(args[0], args[1], permission) {
            self.exit(1);
        }
    }

    unsafe fn fork(&mut self) -> Process {
        let child = Process::init();

        /* Request to make a new child at children manager. */
        use sys::syscall::*;
        let child_pid = child.get_pid().bits();
        self.service_request(Argument::Register(child_pid, 0), PM_FORK, PM_PORT);

        /* Return the arguments */
        let trap_frame = self.get_trap_frame();
        child.get_trap_frame().copy_from(trap_frame);
        trap_frame.a0 = child.get_pid().bits();
        child.get_trap_frame().a0 = 0;

        /* Copy the page take to children. */
        child.get_satp().copy_from(self.get_satp());
        return child;
    }

    pub unsafe fn exit(&mut self, status: usize) -> ! {
        use sys::syscall::*;
        self.service_request(Argument::Register(status, 0), PM_EXIT, PM_PORT);
        current_cpu().get_manager().remove_process(self);
        self.yield_to_scheduler();
        unreachable!("unreachable");
    }

    unsafe fn wait(&mut self, pid : usize) {
        use sys::syscall::*;
        self.service_request(Argument::Register(pid, 0), PM_WAIT, PM_PORT);
    }
}

impl CPU {
    pub unsafe fn sys_yield(&mut self) {
        return self.process_yield();
    }

    pub(super) unsafe fn sys_fork(&mut self) {
        let process     = &mut *self.get_process();
        self.get_manager().insert_process(process.fork());
    }

    pub(super) unsafe fn sys_exit(&mut self) -> ! {
        let process     = &mut *self.get_process();
        let trap_frame  = process.get_trap_frame();
        process.exit(trap_frame.a0);
    }

    pub(super) unsafe fn sys_wait(&mut self) {
        let process     = &mut *self.get_process();
        let trap_frame  = process.get_trap_frame();
        process.wait(trap_frame.a0);
    }
}
