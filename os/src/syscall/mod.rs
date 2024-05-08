mod syscall;
use crate::{cpu::CPU, driver::shutdown, trap::TrapFrame};

/// Handle those unknown syscalls
unsafe fn unknown_syscall(index : usize, trap_frame : *mut TrapFrame) {
    (*trap_frame).a0 = !0;
    warning!("Unknown syscall: {}", index);
}

impl CPU {
    pub unsafe fn syscall(&mut self) {
        use sys::syscall::*;
        let process = self.get_process();
        let trap_frame = (*process).trap_frame;
        let index = (*trap_frame).a7;
        let higher = index >> 32;
        if higher != 0 {
            let index = (index << 32) >> 32;
            (*trap_frame).a7 = index;
            match higher {
                SYS_REQUEST     => self.sys_request(),
                SYS_ACCEPT      => self.sys_accept(),
                SYS_TRANSFER    => self.sys_transfer(),
                SYS_RESPONSE    => self.sys_response(),
                _ => unknown_syscall(index, trap_frame),
            }
        } else {
            match index {
                SYS_SHUTDOWN => shutdown(),
                SYS_YIELD    => self.sys_yield(),
                SYS_REQUEST  => self.sys_request(),
                _ => unknown_syscall(index, trap_frame),
            }
        }
    }
}
