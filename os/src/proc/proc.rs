extern crate alloc;

use core::ptr::null_mut;
use core::sync::atomic::AtomicUsize;

use alloc::collections::VecDeque;
use alloc::str;
use riscv::register::satp;

use crate::cpu::current_cpu;
use crate::driver::get_tid;
use crate::alloc::{ummap, vmmap, PTEFlag, PageAddress, PAGE_SIZE, PAGE_TABLE};
use crate::proc::ProcessStatus;
use crate::service::Iterator;
use crate::trap::{get_trampoline, user_trap, user_trap_return, TrapFrame, TRAMPOLINE, TRAP_FRAME};
use super::{Context, PidType, Process, ProcessManager};

const USER_STACK    : usize     = 1 << 38;

/* Add the init process to the manager. */
pub unsafe fn init_process() {
    let trampoline = get_trampoline();
    vmmap(PAGE_TABLE, TRAMPOLINE, trampoline, PTEFlag::RX);
    let manager = current_cpu().get_manager();
    // Currently, our implementation is problematic.
    // When the queue is full, the old process will be replaced.
    // We need a deque whose iterator will not be invalidated.
    // To handle the problem here, we just reserve enough space.
    // Plan to rewrite in the future.
    manager.process_queue.reserve(32);
    manager.process_queue.push_back(Process::new_test("Demo Program 0", true));
    manager.process_queue.push_back(Process::new_test("Demo Program 1", true));
}

impl Process {
    unsafe fn demo(name : &'static str, parent : * mut Process) -> Process {
        let root    = PageAddress::new_pagetable();

        // Map at least one page for user's stack
        let stack_page = PageAddress::new_rand_page();
        let user_stack = USER_STACK - (PAGE_SIZE as usize);
        ummap(root, user_stack, stack_page, PTEFlag::RW);

        message!("Process {} created with root {:#x}", name, root.address() as usize);

        // Map the trampoline page.
        let trampoline = get_trampoline();
        vmmap(root, TRAMPOLINE, trampoline, PTEFlag::RX);

        // Map the trap frame page.
        let trap_frame = PageAddress::new_rand_page();
        vmmap(root, TRAP_FRAME, trap_frame, PTEFlag::RW);

        // Map the kernel stack page.
        // Note that stack pointer should be set to the top of the page.
        let core_stack = PageAddress::new_rand_page().address() as usize + PAGE_SIZE;

        let trap_frame = trap_frame.address() as *mut TrapFrame;
        let trap_frame = &mut *trap_frame;

        trap_frame.pc = 0;
        trap_frame.sp = USER_STACK;
        trap_frame.thread_number = get_tid() as _;
        trap_frame.kernel_stack  = core_stack as _;
        trap_frame.kernel_satp   = satp::read().bits() as _;
        trap_frame.kernel_trap   = user_trap as _;

        let context = Context::new_with(user_trap_return as _, core_stack);

        // Complete the resource initialization.
        return Process {
            exit_code   : 0,
            status      : ProcessStatus::RUNNABLE,
            pid         : allocate_pid(),
            service     : Iterator::new(),
            context, root, parent, name, trap_frame
        };
    }

    unsafe fn new_test(name : &'static str, which : bool) -> Process {
        let process = Process::demo(name, null_mut());
        let text = PageAddress::new_zero_page();
        ummap(process.root, 0 , text, PTEFlag::X);
        let mmio = PageAddress::new_usize(0x10000000);
        ummap(process.root, 0x10000000 , mmio , PTEFlag::RW);

        let addr = text.address() as *mut u32;
        let program = if which {[
            0x140413,       // addi s0, s0 1
            0x140413,       // addi s0, s0 1
            0x140413,       // addi s0, s0 1
            0x0000bfd5,     // j 0
        ] } else { [
            0x10000537,     // lui a0,0x10000
            0x0320059b,     // addiw a1,zero,0x32
            0x00b50023,     // sb a1,0(a0)
            0x0000bfd5      // j 0
        ] };

        for i in 0..program.len() {
            addr.wrapping_add(i).write_volatile(program[i]);
        }

        return process;
    }

    /* Return the inner context. */
    pub unsafe fn get_context(&mut self) -> &mut Context {
        return &mut self.context;
    }
}

impl Context {
    pub const fn new() -> Self {
        return Self { stack_bottom : 0, };
    }
    /** Create a new context with the given ra and sp. */
    fn new_with(ra : usize, sp : usize) -> Self {
        let ptr = sp as *mut usize;
        unsafe { ptr.wrapping_sub(1).write_volatile(ra); }
        return Self { stack_bottom : sp, };
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
}

static mut PID_POOL : AtomicUsize = AtomicUsize::new(0);

/** Allocate an available pid for the process. */
unsafe fn allocate_pid() -> PidType {
    return PID_POOL.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
}
