extern crate alloc;

use riscv::register::satp;

use crate::cpu::*;
use crate::driver::get_tid;
use crate::alloc::{PTEFlag, PageAddress, PAGE_SIZE};
use crate::service::Argument;
use crate::trap::{user_trap, TrapFrame, TRAP_FRAME};
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
    memory      : MemoryArea,       // memory area
    trap_frame  : * mut TrapFrame,  // trap frame
    context     : Context,          // current context
    response    : Option<Argument>, // response from service
}

impl PageAddress {
    unsafe fn map_trap_frame(self) -> &'static mut TrapFrame {
        let trap_frame = PageAddress::new_rand_page();
        self.smap(TRAP_FRAME, trap_frame, PTEFlag::RW);
        return &mut *(trap_frame.address() as *mut TrapFrame);
    }
    unsafe fn new_kernel_stack() -> usize {
        let stack = Self::new_rand_page();
        return stack.address() as usize + PAGE_SIZE;
    }
}

impl Process {
    /** Initialize those necessary resources first. */
    pub unsafe fn init() -> Process {
        let memory = MemoryArea::new();
        let root = memory.get_satp();
        message!("Process created with root {:#x}", root.address() as usize);

        root.map_trampoline();
        let trap_frame = root.map_trap_frame();
        let core_stack = PageAddress::new_kernel_stack();

        trap_frame.thread_number = get_tid();
        trap_frame.kernel_stack = core_stack;
        trap_frame.kernel_satp  = satp::read().bits();
        trap_frame.kernel_trap  = user_trap as _;

        // Complete the resource initialization.
        return Process {
            status  : ProcessStatus::RUNNABLE,
            pid     : PidType::allocate(),
            context : Context::new_with(core_stack),
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
}

impl CPU {
    pub unsafe fn sleep_as(&mut self, status : ProcessStatus) {
        let process = self.get_process();
        return (*process).sleep_as(status);
    }
}
