use core::ptr::null_mut;
use crate::{alloc::{allocate_one_page, deallocate_one_page, PAGE_SIZE}, cpu::{current_cpu, CPU}, driver::shutdown, proc::ProcessStatus, trap::Interrupt};
use super::{PidType, Process};
extern crate alloc;
use alloc::vec::Vec;

const QLENGTH : usize = PAGE_SIZE / core::mem::size_of::<Process>();

struct Block {
    storage : *mut Process, // A 4096-byte block
    unused  : usize,        // Bitmap of those unused        
}

struct ProcessPool {
    process_pool : Vec<Block>,
}

type TaskQueue = Vec<*mut Process>;

pub struct ProcessManager {
    task_queue      : TaskQueue,
    dead_queue      : TaskQueue,
    running_iter    : usize,
    running_task    : * mut Process,
}

impl CPU {
    /** Return the current running process. */
    pub fn get_process(&mut self) -> *mut Process {
        return self.get_manager().running_task;
    }
    /** Return the next process to work on. May be null. */
    fn next_process(&mut self) -> *mut Process {
        return self.get_manager().next_process();
    }
}

static mut POOL : ProcessPool = unsafe { ProcessPool::new() };

impl ProcessManager {
    pub const fn new() -> Self {
        return Self {
            task_queue      : Vec::new(),
            dead_queue      : Vec::new(),
            running_iter    : 0,
            running_task    : null_mut(),
        };
    }

    pub unsafe fn insert_process(&mut self, process : Process) {
        let real_process = &mut *POOL.add_process(process);
        PidType::register(real_process);
        message!("New process created: {:?}", real_process.get_pid().bits());
    }

    pub unsafe fn remove_process(&mut self, process : &mut Process) {
        process.set_dead();
        self.dead_queue.push(process);
    }

    fn next_process(&mut self) -> *mut Process {
        if self.running_iter == self.task_queue.len() {
            unsafe {
                self.reset_running_iter();
                if self.task_queue.is_empty() {
                    message!("No process to run, shutting down...");
                    shutdown();
                }
            }
        }

        let process = self.task_queue[self.running_iter];
        self.running_iter += 1;

        assert!(!process.is_null(), "Invalid process");
        let process = unsafe { &mut *process };

        if process.is_alive() && process.has_status(ProcessStatus::RUNNABLE) {
            return process;
        } else {
            return core::ptr::null_mut();
        }
    }

    unsafe fn reset_running_iter(&mut self) {
        self.running_iter = 0;
        for process in self.dead_queue.iter() {
            POOL.remove_process(*process);
        }
        POOL.fill_task(&mut self.task_queue);
        self.dead_queue.clear();
    }

    pub unsafe fn switch_from_to(&mut self, old : *mut Process, new : *mut Process) {
        assert!(self.running_task == old, "Invalid process to reset");
        self.running_task = new;
    }
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
            assert!(cpu.get_process().is_null(), "Task should be null");
        }

        Interrupt::enable();
    }
}

impl Block {
    const UNUSED : usize  = (1 << QLENGTH)- 1;
    unsafe fn new() -> Block {
        let storage = allocate_one_page() as *mut Process;
        return Block { storage, unused : Self::UNUSED };
    }

    unsafe fn free(&self) {
        assert!(self.unused == Self::UNUSED, "Block not empty");
        deallocate_one_page(self.storage as _);
    }

    unsafe fn has_space(&self) -> bool {
        return self.unused != 0;
    }

    unsafe fn set_used(&mut self, index : usize) {
        self.unused &= !(1 as usize) << index;
    }

    unsafe fn set_unused(&mut self, index : usize) {
        self.unused |= (1 as usize) << index;
    }

    unsafe fn is_used(&self, index : usize) -> bool {
        return self.unused & ((1 as usize) << index) == 0;
    }

    unsafe fn add_process(&mut self, process : Process) -> *mut Process{
        assert!(self.has_space(), "No space to add process");
        let index = self.unused.trailing_zeros(); // Find first unused slot
        
        assert!(!self.is_used(index as usize), "Process already exists");
        self.set_used(index as usize);

        let address = self.storage.add(index as usize);
        address.write(process);
        return address;
    }

    unsafe fn try_remove_process(&mut self, process : *mut Process) -> bool {
        let offset = process as usize - self.storage as usize;
        if offset < PAGE_SIZE {
            assert!(offset % core::mem::size_of::<Process>() == 0, "Misaligned process");
            let index = offset / core::mem::size_of::<Process>();

            assert!(self.is_used(index), "Process not found");
            self.set_unused(index);

            (*process).destroy();
            return true;
        } else {
            return false;
        }
    }

    unsafe fn fill_task(&self, queue : &mut TaskQueue) {
        for index in 0..QLENGTH {
            if self.is_used(index) {
                queue.push(self.storage.add(index));
            }
        }
    }
}

impl ProcessPool {
    pub const unsafe fn new() -> Self {
        return Self { process_pool : Vec::new(), };
    }

    pub unsafe fn add_process(&mut self, process : Process) -> *mut Process {
        for block in self.process_pool.iter_mut() {
            if block.has_space() {
                return block.add_process(process);
            }
        }
        let mut block = Block::new();
        let result = block.add_process(process);
        self.process_pool.push(block);
        return result;
    }

    pub unsafe fn remove_process(&mut self, process : *mut Process) {
        for block in self.process_pool.iter_mut() {
            if block.try_remove_process(process) {
                return;
            }
        }
        assert!(false, "Process not found");
    }

    pub unsafe fn fill_task(&self, queue : &mut TaskQueue) {
        queue.clear();
        for block in self.process_pool.iter() {
            block.fill_task(queue);
        }
    }
}
