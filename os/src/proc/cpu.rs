use crate::driver::get_tid;
use crate::driver::timer::TimeScartch;
use crate::layout::NCPU;
use crate::proc::{Context, ProcessManager as Manager, ProcessStatus};

use super::Process;

pub struct IMPLEMENTEDCPU {
    context : Context,
    manager : Manager,
    scratch : TimeScartch,
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

/** Run the process. */
unsafe fn run(process : *mut Process) {
    assert_eq!((*process).get_status(), ProcessStatus::RUNNABLE);
    (*process).set_status(ProcessStatus::RUNNING);
}

/** Run the process. */
unsafe fn off(process : *mut Process) {
    if (*process).has_status(ProcessStatus::RUNNING) {
        (*process).set_status(ProcessStatus::RUNNABLE);
    }
}

extern "C" { fn switch_context(old : *mut Context, new : *mut Context); }

impl IMPLEMENTEDCPU {
    /** Switch from current process to new process. Timer is untouched. */
    pub unsafe fn switch_to(&mut self, new : *mut Process) {
        let old = self.get_process();
        off(old); run(new);
        switch_context((*old).get_context(), (*new).get_context());
    }

    /** Switch from current process to the scheduler. Timer is reset. */
    pub unsafe fn process_yield(&mut self) {
        let old = self.get_process();
        off(old);
        switch_context((*old).get_context(), self.get_context());
    }

    /** Switch from scheduler to the new process. Timer is untouched. */
    pub unsafe fn scheduler_yield(&mut self, new : *mut Process) {
        assert_eq!((*new).get_status(), ProcessStatus::RUNNABLE);
        run(new);
        switch_context(self.get_context(), (*new).get_context());
    }
}

impl Process {
    /** Switch from current process to new process. Timer is untouched. */
    pub unsafe fn yield_to_process(&mut self, new : &mut Process) {
        off(self); run(new);
        switch_context(self.get_context(), new.get_context());
    }
    /** Switch from current process to the scheduler. Timer is reset. */
    pub unsafe fn yield_to_scheduler(&mut self) {
        let cpu = current_cpu();
        off(self);
        switch_context(self.get_context(), cpu.get_context());
    }
}
