use sys::syscall::ARGS_BUFFERED;

use crate::{alloc::{CheckError, PTEFlag}, cpu::CPU, proc::{current_cpu, Process}, service::Argument};

impl Process {
    pub unsafe fn address_check(&mut self, args : [usize; 2], permission : PTEFlag) {
        loop {
            let result = self.get_satp().check_ptr(args[0], args[1], permission);
            match result {
                CheckError::Nothing => return,
                CheckError::MissingPage(addr) => {
                    warning!("address check fail at {:x}", addr);

                    if permission.contains(PTEFlag::RW) {
                        self.handle_page_fault(addr, crate::trap::PageFaultType::Store)
                    } else {
                        self.handle_page_fault(addr, crate::trap::PageFaultType::Load)
                    }
                },
            }
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

    unsafe fn exit(&mut self, status: usize) -> ! {
        use sys::syscall::*;
        self.service_request(Argument::Register(status, 0), PM_EXIT, PM_PORT);
        current_cpu().get_manager().remove_process(self);
        self.yield_to_scheduler();
        unreachable!("unreachable");
    }

    unsafe fn wait(&mut self, pid : usize) {
        use sys::syscall::*;
        self.service_request(Argument::Register(pid, 0), PM_WAIT, PM_PORT);
        match self.get_response() {
            Some(arugment) => {
                match arugment.get_register() {
                    Some((pid, status)) => {
                        let trap_frame = self.get_trap_frame();
                        trap_frame.a0 = pid;
                        trap_frame.a1 = status;
                        return;
                    },
                    None => {}
                }
            },
            None => {},
        }
        panic!("invalid response from PM_WAIT");
    }

    unsafe fn exec(&mut self, name : &[u8]) {
        self.exec_test(name);
    }

    pub unsafe fn handle_fatal_error(&mut self, msg: &str) -> ! {
        warning!("process {} fatal error: {}", self.get_pid().bits(), msg);
        self.exit(1);
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

    pub(super) unsafe fn sys_exec(&mut self) {
        let process     = &mut *self.get_process();
        let trap_frame  = process.get_trap_frame();
        // We ignore a2 and a3 now.
        let args = Argument::new([trap_frame.a0, trap_frame.a1, ARGS_BUFFERED], process);
        match args {
            Argument::Buffered(data) => {
                return process.exec(&data[..]);
            }
            _ => panic!("Impossible argument type")
        }
    }
}
