use core::ptr::null_mut;

use crate::{cpu::CPU, proc::ProcessStatus, trap::user_trap_return};

use super::{current_cpu, Process};

#[repr(C)]
pub struct Context { stack_bottom : usize, }

impl Context {
    pub const fn new() -> Self {
        return Self { stack_bottom : 0, };
    }

    /** Create a with given ra and sp. */
    pub(super) fn new_with(sp : usize) -> Self {
        let ra = user_trap_return as usize;
        let ptr = sp as *mut usize;
        unsafe { ptr.wrapping_sub(1).write_volatile(ra); }
        return Self { stack_bottom : sp, };
    }
}

/** Off the process. */
unsafe fn off(process : &mut Process) {
    if process.has_status(ProcessStatus::RUNNING) {
        process.set_status(ProcessStatus::RUNNABLE);
    }
}

/** Run the process. */
unsafe fn run(process : &mut Process) {
    assert_eq!(process.get_status(), ProcessStatus::RUNNABLE);
    process.set_status(ProcessStatus::RUNNING);
}

unsafe fn switch_from_to(old : &mut Process, new : &mut Process, cpu : &mut CPU) {
    extern "C" { fn switch_context(old : *mut Context, new : *mut Context); }
    off(old); run(new);
    cpu.get_manager().switch_from_to(old, new);
    switch_context(old.get_context(), new.get_context());
}

unsafe fn switch_to(new : &mut Process, cpu : &mut CPU) {
    extern "C" { fn switch_context(old : *mut Context, new : *mut Context); }
    assert_eq!((*new).get_status(), ProcessStatus::RUNNABLE);
    run(new);
    cpu.get_manager().switch_from_to(null_mut(), new);
    switch_context(cpu.get_context(), new.get_context());
}

unsafe fn switch_from(old : &mut Process, cpu : &mut CPU) {
    extern "C" { fn switch_context(old : *mut Context, new : *mut Context); }
    off(old);
    cpu.get_manager().switch_from_to(old, null_mut());
    switch_context(old.get_context(), cpu.get_context());
}

impl CPU {
    /** Switch from current process to new process. Timer is untouched. */
    pub unsafe fn switch_to(&mut self, new : *mut Process) {
        let old = &mut *self.get_process();
        let new = &mut *new;
        switch_from_to(old, new, self);
    }

    /** Switch from current process to the scheduler. Timer is reset. */
    pub unsafe fn process_yield(&mut self) {
        let old = &mut *self.get_process();
        switch_from(old, self);
    }

    /** Switch from scheduler to the new process. Timer is untouched. */
    pub unsafe fn scheduler_yield(&mut self, new : *mut Process) {
        let new = &mut *new;
        switch_to(new, self);
    }
}

impl Process {
    /** Switch from current process to new process. Timer is untouched. */
    pub unsafe fn yield_to_process(&mut self, new : &mut Process) {
        let cpu = current_cpu();
        switch_from_to(self, new, cpu);
    }

    /** Switch from current process to the scheduler. Timer is reset. */
    pub unsafe fn yield_to_scheduler(&mut self) {
        let cpu = current_cpu();
        switch_from(self, cpu);
    }
}
