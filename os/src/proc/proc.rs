extern crate alloc;

use riscv::register::satp;

use crate::cpu::*;
use crate::driver::get_tid;
use crate::alloc::{PTEFlag, PageAddress, PAGE_SIZE};
use crate::trap::{get_trampoline, user_trap, TrapFrame, TRAMPOLINE, TRAP_FRAME};
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
    pub context     : Context,          // current context
}

impl Process {
    pub(super) unsafe fn init() -> Process {
        let root = PageAddress::new_pagetable();

        // Map at least one page for user's stack
        let stack_page = PageAddress::new_rand_page();
        let user_stack = USER_STACK - (PAGE_SIZE as usize);
        root.umap(user_stack, stack_page, PTEFlag::RW | PTEFlag::OWNED);

        message!("Process created with root {:#x}", root.address() as usize);

        // Map the trampoline page.
        let trampoline = get_trampoline();
        root.smap(TRAMPOLINE, trampoline, PTEFlag::RX);

        // Map the trap frame page.
        let trap_frame = PageAddress::new_rand_page();
        root.smap(TRAP_FRAME, trap_frame, PTEFlag::RW | PTEFlag::OWNED);

        // Map the kernel stack page.
        // Note that stack pointer should be set to the top of the page.
        let core_stack = PageAddress::new_rand_page().address() as usize + PAGE_SIZE;

        let trap_frame = trap_frame.address() as *mut TrapFrame;
        let trap_frame = &mut *trap_frame;

        trap_frame.pc = 0;
        trap_frame.sp = USER_STACK;
        trap_frame.thread_number = get_tid();
        trap_frame.kernel_stack  = core_stack;
        trap_frame.kernel_satp   = satp::read().bits();
        trap_frame.kernel_trap   = user_trap as _;

        let context = Context::new_with(core_stack);

        // Complete the resource initialization.
        return Process {
            status      : ProcessStatus::RUNNABLE,
            pid         : PidType::allocate(),
            context, root, trap_frame
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
        let process = Process::init();
        (*self.trap_frame).fork_copy_to(process.trap_frame);
        (*self.trap_frame).a0 = process.pid.raw_bits();
        process.get_trap_frame().a0 = 0;
        return process;
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
