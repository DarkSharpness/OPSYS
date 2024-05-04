use crate::cpu::current_cpu;
use crate::proc::*;
use crate::driver::timer::set_timer_next;

pub unsafe fn sys_yield() {
    set_timer_next();
    let cpu = current_cpu();
    let proc = cpu.get_process();

    let old_context = cpu.get_context();
    let new_context = (*proc).get_context();
    
    /* Switch back to previous content. */
    return switch_context(new_context as _, old_context as _);
}

pub unsafe fn sys_wake_up(_process : *mut Process) {
    
}
