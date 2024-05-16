use core::ptr::null_mut;

use crate::{cpu::{current_cpu, CPU}, trap::Interrupt};
use super::{Process, ProcessStatus};

pub unsafe fn run_process() {
    logging!("Starting process scheduler...");
    let cpu = current_cpu();
    loop {
        Interrupt::disable();

        let prev_task   = cpu.get_process();
        assert!(prev_task.is_null(), "Task should be null");
        let next_task   = cpu.next_process();
        // Try to listen to the interrupt
        if !next_task.is_null() {
            cpu.scheduler_yield(next_task);
            cpu.complete_process(next_task);
        }

        Interrupt::enable(); 
    }
}

impl CPU {
    pub fn get_process(&mut self) -> *mut Process {
        return self.get_manager().running_process;
    }

    pub fn next_process(&mut self) -> *mut Process {
        let manager = self.get_manager();
        if manager.batch_iter == manager.batch_size {
            manager.batch_iter = 0;
            manager.batch_size = manager.process_queue.len();
            assert!(manager.batch_size > 0, "No process to run");
        }

        let process = &mut manager.process_queue[manager.batch_iter];
        if (*process).status != ProcessStatus::RUNNABLE { return null_mut(); }

        manager.running_process = process;
        manager.batch_iter += 1;

        return process;
    }

    pub fn complete_process(&mut self, process : *mut Process) {
        let manager = self.get_manager();
        assert!(manager.running_process == process, "Invalid process to complete");
        manager.running_process = core::ptr::null_mut();
    }
}
