extern crate alloc;

use crate::alloc::PageAddress;
use crate::service::Argument;
use crate::trap::TrapFrame;
use super::memory::MemoryArea;
use super::{Context, PidType};

const USER_STACK : usize = 1 << 38;

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
    isalive     : bool,             // is the process alive
    memory      : MemoryArea,       // memory area
    trap_frame  : * mut TrapFrame,  // trap frame
    context     : Context,          // current context
    response    : Option<Argument>, // response from service
}

impl Process {
    /** Initialize those necessary resources first. */
    pub unsafe fn init() -> Process {
        let memory = MemoryArea::new();
        let root = memory.get_satp();
        message!("Process created with root {:#x}", root.address() as usize);

        root.map_trampoline();
        let (trap_frame, kernel_stack) = root.map_trap_frame();

        // Complete the resource initialization.
        return Process {
            status  : ProcessStatus::RUNNABLE,
            pid     : PidType::new(0), // As a placeholder.
            isalive : true,
            context : Context::new_with(kernel_stack),
            response : None,
            memory, trap_frame
        };
    }

    /** Return the inner context. */
    pub(super) unsafe fn get_context(&mut self) -> &mut Context {
        return &mut self.context;
    }

    pub unsafe fn get_trap_frame(&self) -> &mut TrapFrame {
        return &mut *self.trap_frame;
    }

    pub(super) fn set_pid(&mut self, pid : PidType) {
        self.pid = pid;
    }

    pub fn get_pid(&self) -> PidType {
        return self.pid.clone();
    }

    pub fn get_status(&self) -> ProcessStatus {
        return self.status.clone();
    }

    pub fn has_status(&self, status : ProcessStatus) -> bool {
        return self.status == status;
    }

    pub fn set_status(&mut self, status : ProcessStatus) {
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
    }

    /** Wake up from given status. */
    pub fn wake_up_from(&mut self, status : ProcessStatus) {
        assert_eq!(self.status, status, "Invalid to wake up!");
        self.status = ProcessStatus::RUNNABLE;
    }

    pub fn set_response(&mut self, response : Argument) {
        self.response = Some(response);
    }

    pub fn get_response(&mut self) -> Option<Argument> {
        return self.response.take();
    }

    pub fn is_alive(&self) -> bool {
        return self.isalive;
    }

    pub fn set_dead(&mut self) {
        self.isalive = false;
    }

    pub unsafe fn destroy(&mut self) {
        PidType::unregister(self);

        self.get_memory_area().free();
        self.get_trap_frame().free();

        let _ = *self; // Drop the process.
        todo!("Destroy the process.");
    }
}
