mod fs;
mod mm;
mod ipc;
mod process;

use crate::{cpu::CPU, driver::shutdown, trap::TrapFrame};

/// Handle those unknown syscalls
unsafe fn unknown_syscall(index : usize, trap_frame : &mut TrapFrame) {
    trap_frame.a0 = !0;
    warning!("Unknown syscall: {}", index);
}

impl CPU {
    pub unsafe fn syscall(&mut self) {
        use sys::syscall::*;
        let process = self.get_process();
        let trap_frame = (*process).get_trap_frame();

        trap_frame.pc += 4; // Skip the ecall instruction

        let index = trap_frame.a7;
        match index {
            SYS_SHUTDOWN    => shutdown(),
            SYS_YIELD       => self.sys_yield(),
            SYS_REQUEST     => self.sys_request(),
            SYS_RECEIVE     => self.sys_receive(),
            SYS_RESPOND     => self.sys_respond(),
            SYS_READ        => self.sys_read(),
            SYS_WRITE       => self.sys_write(),
            SYS_FORK        => self.sys_fork(),
            SYS_EXIT        => self.sys_exit(),
            SYS_WAIT        => self.sys_wait(),
            SYS_EXEC        => self.sys_exec(),
            SYS_SBRK        => self.sys_sbrk(),
            _ => {
                unknown_syscall(index, trap_frame);
                (*process).handle_fatal_error("Unknown syscall");
            }
        }
    }
}
