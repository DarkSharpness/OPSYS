use crate::{alloc::PTEFlag, cpu::CPU,};

impl CPU {
    pub unsafe fn sys_read(&mut self){
        let process     = &mut *self.get_process();
        let trap_frame  = process.get_trap_frame();
        let fd  = trap_frame.a0;
        let buf = trap_frame.a1;
        let len = trap_frame.a2;

        process.address_check([buf, len], PTEFlag::WO);
        match fd {
            0 => {},
            _ => {
                warning!("Unknown file descriptor: {}", fd);
                process.exit(1);
            }
        }

        let result = process.console_read(buf, len);
        process.get_trap_frame().a0 = result;
    }

    pub unsafe fn sys_write(&mut self){
        let process     = &mut *self.get_process();
        let trap_frame  = process.get_trap_frame();
        let fd  = trap_frame.a0;
        let buf = trap_frame.a1;
        let len = trap_frame.a2;

        process.address_check([buf, len], PTEFlag::RO);
        match fd {
            1 => {},
            2 => {},
            _ => {
                warning!("Unknown file descriptor: {}", fd);
                process.exit(1);
            }
        }

        let result = process.console_write(buf, len);
        process.get_trap_frame().a0 = result;

        match fd {
            1 => {},
            2 => {},
            _ => {
                warning!("Unknown file descriptor: {}", fd);
                process.exit(1);
            }
        }
    }
}
