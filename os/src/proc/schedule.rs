use core::ptr::null_mut;
use crate::{alloc::{PTEFlag, PAGE_TABLE}, cpu::{current_cpu, CPU}, trap::{get_trampoline, Interrupt, TRAMPOLINE}};
use super::{PidType, Process, ProcessStatus};
extern crate alloc;
use alloc::collections::VecDeque;

pub struct ProcessManager {
    process_queue   : VecDeque<Process>,
    running_process : * mut Process,
    batch_iter      : usize,
    batch_size      : usize,
}

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
        if !process.has_status(ProcessStatus::RUNNABLE) { return null_mut(); }

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

impl ProcessManager {
    pub const fn new() -> Self {
        return Self {
            process_queue   : VecDeque::new(),
            running_process : null_mut(),
            batch_iter      : 0,
            batch_size      : 0,
        };
    }

    /// TODO:
    /// Currently, our implementation is problematic.
    /// When the queue is full, the old process will be replaced.
    /// We need a deque whose iterator will not be invalidated.
    /// To handle the problem here, we just reserve enough space.
    /// Plan to rewrite in the future.
    pub(super) fn init(&mut self) {
        self.process_queue.reserve(64);
    }

    pub(super) unsafe fn add_process(&mut self, process : Process) -> &mut Process {
        self.process_queue.push_back(process);
        let back = self.process_queue.back_mut().unwrap();
        PidType::register(back);
        return back;
    }
}

pub unsafe fn init_process() {
    /* Add the init process to the manager. */
    let trampoline = get_trampoline();
    PAGE_TABLE.smap(TRAMPOLINE, trampoline, PTEFlag::RX);
 
    let manager = current_cpu().get_manager();
 
    manager.init();
    manager.add_process(Process::new_test(0));
}
