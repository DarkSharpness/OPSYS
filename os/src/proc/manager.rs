use core::ptr::null_mut;
use crate::{alloc::{allocate_one_page, deallocate_one_page, PAGE_SIZE}, cpu::{current_cpu, CPU}, trap::Interrupt};
use super::{schedule::Schuduler, PidType, Process};
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
    schduler        : Schuduler,
    dead_stack      : TaskQueue,
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
            schduler        : Schuduler::new(),
            dead_stack      : Vec::new(),
            running_task    : null_mut(),
        };
    }

    pub fn insert_runnable(&mut self, process : &mut Process) {
        let process = &mut *process;
        self.schduler.register(process);
    }

    pub fn remove_runnable(&mut self, process : &mut Process) {
        let process = &mut *process;
        self.schduler.unregister(process);
    }

    pub unsafe fn insert_process(&mut self, process : Process) {
        let real_process = &mut *POOL.add_process(process);
        PidType::register(real_process);
        self.insert_runnable(real_process);
        message!("New process created: {:?}", real_process.get_pid().bits());
    }

    pub unsafe fn remove_process(&mut self, process : &mut Process) {
        self.schduler.unregister(process);
        self.dead_stack.push(process);
    }

    fn next_process(&mut self) -> *mut Process {
        unsafe { self.remove_dead(); }
        return self.schduler.next_process();
    }

    unsafe fn remove_dead(&mut self) {
        for process in self.dead_stack.iter() {
            POOL.remove_process(*process);
        }
        self.dead_stack.clear();
    }

    pub unsafe fn switch_from_to(&mut self, old : *mut Process, new : *mut Process) {
        if self.running_task != old {
            assert!(self.running_task == old, "Invalid process to reset");
        }
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
}
