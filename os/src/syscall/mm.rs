use crate::cpu::CPU;

impl CPU {
    pub unsafe fn sys_sbrk(&mut self) {
        let process     = &mut *self.get_process();
        let trap_frame  = process.get_trap_frame();
        let increment   = trap_frame.a0 as isize;
        let result      = process.get_memory_area().sbrk(increment);
        process.get_trap_frame().a0 = result as usize;
    }
}
