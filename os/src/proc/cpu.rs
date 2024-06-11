use crate::driver::get_tid;
use crate::driver::timer::TimeScartch;
use crate::layout::NCPU;
use crate::proc::{Context, ProcessManager as Manager};

pub struct IMPLEMENTEDCPU {
    pub context : Context,
    pub manager : Manager,
    pub scratch : TimeScartch,
}

impl IMPLEMENTEDCPU {
    const fn new() -> Self {
        Self {
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

static mut CPUS : [IMPLEMENTEDCPU; NCPU] = [IMPLEMENTEDCPU::new(); NCPU];

pub fn current_cpu() -> &'static mut IMPLEMENTEDCPU {
    let tid = get_tid();
    return unsafe { &mut CPUS[tid] };
}
