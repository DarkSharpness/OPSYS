extern crate alloc;

use crate::alloc::PageAddress;
use crate::proc::current_cpu;
use crate::service::Argument;
use crate::trap::TrapFrame;
use super::memory::MemoryArea;
use super::{Context, PidType};

#[derive(Debug, PartialEq, Clone)]
pub enum ProcessStatus {
    RUNNING,    // running on CPU
    RUNNABLE,   // ready to run, but not running
    SERVING,    // serving some service
    SERVICE,    // waiting for some service
}

pub struct Process {
    pid         : PidType,          // process id
    status      : ProcessStatus,    // process status
    memory      : MemoryArea,       // memory area
    trap_frame  : * mut TrapFrame,  // trap frame
    context     : Context,          // current context
    response    : Option<Argument>, // response from service
    priority    : u16,              // priority
    timing      : usize,            // timing
}

impl Process {
    /** Initialize those necessary resources first. */
    pub unsafe fn init() -> Process {
        let memory  = MemoryArea::new();
        let root    = memory.get_satp();
        message!("Process created with root {:#x}", root.address() as usize);

        let (trap_frame, kernel_stack) = root.map_trap_frame();

        // Complete the resource initialization.
        return Process {
            status  : ProcessStatus::RUNNABLE,
            pid     : PidType::allocate(),
            context : Context::new_with(kernel_stack),
            response : None,
            priority : 1,
            timing   : 0,
            memory, trap_frame
        };
    }

    pub(super) unsafe fn reinit(&mut self) {
        self.get_memory_area().free();
        self.get_trap_frame().free();

        self.memory = MemoryArea::new();
        let root    = self.memory.get_satp();
        message!("Process re-created with root {:#x}", root.address() as usize);
        let (trap_frame, kernel_stack) = self.get_memory_area().get_satp().map_trap_frame();
        self.trap_frame = trap_frame;
        self.context    = Context::new_with(kernel_stack);
        self.response   = None;
        assert!(self.status == ProcessStatus::RUNNING);
    }

    /** Return the inner context. */
    pub(super) unsafe fn get_context(&mut self) -> &mut Context {
        return &mut self.context;
    }

    pub unsafe fn get_trap_frame(&self) -> &mut TrapFrame {
        return &mut *self.trap_frame;
    }

    pub fn get_pid(&self) -> PidType {
        return self.pid.clone();
    }

    pub(super) fn get_status(&self) -> ProcessStatus {
        return self.status.clone();
    }

    pub(super) fn has_status(&self, status : ProcessStatus) -> bool {
        return self.status == status;
    }

    pub(super) fn set_status(&mut self, status : ProcessStatus) {
        self.status = status;
    }

    pub fn get_satp(&self) -> PageAddress {
        return self.memory.get_satp();
    }

    pub fn get_memory_area(&mut self) -> &mut MemoryArea {
        return &mut self.memory;
    }

    /** Sleep and set the status as given. */
    pub fn sleep_as(&mut self, status : ProcessStatus) {
        assert_eq!(self.status, ProcessStatus::RUNNING, "Invalid to sleep!");
        self.status = status;
        current_cpu().get_manager().remove_runnable(self);
    }

    /** Wake up from given status. */
    pub fn wake_up_from(&mut self, status : ProcessStatus) {
        assert_eq!(self.status, status, "Invalid to wake up!");
        self.status = ProcessStatus::RUNNABLE;
        current_cpu().get_manager().insert_runnable(self);
    }

    pub fn set_response(&mut self, response : Argument) {
        self.response = Some(response);
    }

    pub fn get_response(&mut self) -> Option<Argument> {
        core::hint::black_box(&self.response);
        return self.response.take();
    }

    pub unsafe fn destroy(&mut self) {
        PidType::unregister(self);

        self.get_memory_area().free();
        self.get_trap_frame().free();

        let _ = *self; // Drop the process.
    }

    pub fn set_priority(&mut self, priority : u16) {
        self.priority = priority;
    }

    pub fn get_priority(&self) -> usize {
        return self.priority as usize;
    }

    pub fn set_timing(&mut self, timing : usize) {
        self.timing = timing;
    }

    pub fn get_timing(&self) -> usize {
        return self.timing;
    }

    pub const fn max_priority() -> usize {
        return core::u16::MAX as usize;
    }
}
