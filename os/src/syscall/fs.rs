use crate::{alloc::PTEFlag, cpu::CPU};

impl CPU {
    pub unsafe fn sys_read(&mut self){
        let process     = self.get_process();
        let trap_frame  = (*process).get_trap_frame();

        let buf = trap_frame.a1;
        let len = trap_frame.a2;

        self.address_check(&[buf, len, 1], PTEFlag::RW);
        trap_frame.a0 = self.console_read(buf, len);
    }

    pub unsafe fn sys_write(&mut self){
        let process     = &mut *self.get_process();
        let trap_frame  = process.get_trap_frame();

        let buf = trap_frame.a1;
        let len = trap_frame.a2;

        self.address_check(&[buf, len, 1], PTEFlag::RO);
        trap_frame.a0 = self.console_write(buf, len);
    }
}
