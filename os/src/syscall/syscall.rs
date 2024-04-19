use crate::proc::*;
use crate::driver::timer::set_timer_next;

pub unsafe fn sys_yield() {
    set_timer_next();
    extern "C" { fn switch_context(x : * mut u8, y : * mut u8); }
    let proc = get_process();
    
    let old_context = get_context();
    let new_context = (*proc).get_context();

    /* Switch back to previous content. */
    return switch_context(new_context as _, old_context as _);
}
