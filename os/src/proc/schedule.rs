use crate::trap::Interrupt;
use super::proc::*;

extern "C" { fn switch_context(prev : * const u8, next : * const u8 ); }

pub unsafe fn run_process() {
    logging!("Starting process scheduler...");
    loop {
        Interrupt::disable();
        let prev_task = current_process();
        assert!(prev_task.is_null(), "Task should be null");
        let next_task = next_process();
        switch_context(get_context() as _, (*next_task).context as _);
        complete_process(next_task);
        Interrupt::enable();
    }
}

/**
 * We use a deque to store the process queue.
 * Currently, we do not support process deletion.
 */
unsafe fn next_process() -> *mut Process {
    let manager = get_manager();
    if manager.batch_iter == manager.batch_size {
        manager.batch_iter = 0;
        manager.batch_size = manager.process_queue.len();
        assert!(manager.batch_size > 0, "No process to run");
    }

    let process = &mut manager.process_queue[manager.batch_iter];

    manager.running_process = process;
    manager.batch_iter += 1;

    return process;
}

pub unsafe fn complete_process(process : *mut Process) {
    let manager = get_manager();
    assert!(manager.running_process == process, "Invalid process to complete");
    manager.running_process = core::ptr::null_mut();
}