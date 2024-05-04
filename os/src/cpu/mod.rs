use crate::driver::get_tid;
use crate::driver::timer::TimeScartch;
use crate::layout::NCPU;
use crate::proc::{Context, ProcessManager as Manager};

pub struct CPU {
    pub context : Context,
    pub manager : Manager,
    pub scratch : TimeScartch,
}

impl CPU {
    const fn new() -> CPU {
        CPU {
            context : Context::new(),
            manager : Manager::new(),
            scratch : TimeScartch::new(),
        }
    }

    pub fn get_timer(&mut self) -> &mut TimeScartch {
        return &mut self.scratch;
    }
    pub fn get_manager(&mut self) -> &mut Manager {
        return &mut self.manager;
    }
    pub fn get_context(&mut self) -> &mut Context {
        return &mut self.context;
    }
}

static mut CPU : [CPU; NCPU] = [CPU::new(); NCPU];

pub fn current_cpu() -> &'static mut CPU {
    let tid = get_tid();
    return unsafe { &mut CPU[tid] };
}
