extern crate alloc;

use riscv::register::satp;

use crate::cpu::*;
use crate::driver::get_tid;
use crate::alloc::{PTEFlag, PageAddress, PAGE_SIZE};
use crate::trap::{user_trap, TrapFrame, TRAP_FRAME};
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
    root        : PageAddress,      // root of the page table
    trap_frame  : * mut TrapFrame,  // trap frame
    context     : Context,          // current context
}

impl PageAddress {
    unsafe fn map_trap_frame(self) -> &'static mut TrapFrame {
        let trap_frame = PageAddress::new_rand_page();
        self.smap(TRAP_FRAME, trap_frame, PTEFlag::RW | PTEFlag::OWNED);
        return &mut *(trap_frame.address() as *mut TrapFrame);
    }
    unsafe fn new_kernel_stack() -> usize {
        let stack = Self::new_rand_page();
        return stack.address() as usize + PAGE_SIZE;
    }
}

impl Process {
    /** Initialize those necessary resources first. */
    pub(super) unsafe fn init() -> Process {
        let root = PageAddress::new_pagetable();
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
            root, trap_frame
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
        return self.root.clone();
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

    pub unsafe fn fork(&self) -> Process {
        let child       = Process::init();
        let trap_frame  = self.get_trap_frame();

        trap_frame.duplicate(child.trap_frame);

        trap_frame.a0 = child.get_pid().raw_bits();
        child.get_trap_frame().a0 = 0;

        child.get_satp().duplicate(self.root);
        return child;
    }
}

impl CPU {
    pub unsafe fn sleep_as(&mut self, status : ProcessStatus) {
        let process = self.get_process();
        return (*process).sleep_as(status);
    }

    pub unsafe fn fork(&mut self) -> &mut Process {
        let process = &mut *self.get_process();
        return self.manager.add_process(process.fork());
    }
}
