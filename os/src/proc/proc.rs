extern crate alloc;
extern "C" { fn user_return(satp : usize); }

use core::ptr::null_mut;
use core::sync::atomic::AtomicU64;

use alloc::collections::VecDeque;
use alloc::str;
use riscv::register::satp;
use crate::driver::get_tid;
use crate::layout::*;

use crate::alloc::{ummap, vmmap, PTEFlag, PageAddress, PAGE_SIZE, PAGE_TABLE};
use crate::trap::{get_trampoline, user_trap_return, TrapFrame, TRAMPOLINE, TRAP_FRAME};

use super::USER_STACK;

#[repr(C)]
pub struct Context {
    ra  : u64,
    sp  : u64,
    saved_registers : [u64; 12],
}

type PidType = u64;

pub enum ProcessStatus {
    SLEEPING, // blocked
    RUNNABLE, // ready to run, but not running
    RUNNING, // running on CPU
    ZOMBIE, // exited but have to be waited by parent
    DEAD    // exited and no need to be waited by parent
}

pub struct Process {
    pub pid         : PidType,          // process id
    pub exit_code   : i32,              // exit code
    pub status      : ProcessStatus,    // process status
    pub root        : PageAddress,      // root of the page table
    pub parent      : * mut Process,    // parent process
    pub trap_frame  : * mut TrapFrame,  // trap frame
    pub name        : &'static str,     // process name
    pub context     : Context,          // current context
}

pub struct ProcessManager {
    pub process_queue   : VecDeque<Process>,
    pub running_process : * mut Process,
    pub batch_iter      : usize,
    pub batch_size      : usize,
}

static mut MANAGER : [ProcessManager; NCPU] = [
    ProcessManager {
        process_queue   : VecDeque::new(),
        running_process : core::ptr::null_mut(),
        batch_iter      : 0,
        batch_size      : 0,
    }; NCPU];

static mut CONTEXT : [Context; NCPU] = [
    Context {
        ra              : 0,
        sp              : 0,
        saved_registers : [0; 12],
    }; NCPU];

const TEST_PROGRAM0 : [u32; 4] = [
    0x10000537, // lui a0,0x10000
    // 0x0310059b, // addiw a1,zero,0x31
    // 0x00b50023, // sb a1,0(a0)
    0x0000bfd5, // j 0
    0x0000bfd5, // j 0
    0x0000bfd5, // j 0
];

static mut PID_POOL : AtomicU64 = AtomicU64::new(0);

pub unsafe fn current_process() -> *mut Process {
    let manager = get_manager();
    return manager.running_process;
}

pub unsafe fn init_process() {
    let trampoline = get_trampoline();
    vmmap(PAGE_TABLE, TRAMPOLINE, trampoline, PTEFlag::RX);

    let manager = get_manager();
    manager.process_queue.push_back(Process::new_test("Demo Program"));
}


impl Process {
    pub unsafe fn new(name : &'static str, parent : * mut Process) -> Process {
        let root    = PageAddress::new_pagetable();

        // Map at least one page for user's stack
        let stack_page = PageAddress::new_rand_page();
        let user_stack = USER_STACK - (PAGE_SIZE as u64);
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

        let context = Context {
            ra              : user_trap_return as u64,
            sp              : core_stack as _,
            saved_registers : [0; 12],
        };

        // Complete the resource initialization.
        return Process {
            exit_code   : 0,
            status      : ProcessStatus::RUNNABLE,
            pid         : allocate_pid(),
            context, root, parent, name, trap_frame
        };
    }

    pub unsafe fn new_test(name : &'static str) -> Process {
        let process = Process::new(name, null_mut());
        let text = PageAddress::new_zero_page();
        ummap(process.root, 0 , text, PTEFlag::X);
        let mmio = PageAddress::new_u64(0x10000000);
        ummap(process.root, 0x10000000 , mmio , PTEFlag::RW);

        // Copy in TEST_PROGRAM0
        let addr = text.address() as *mut u32;
        for i in 0..TEST_PROGRAM0.len() {
            addr.wrapping_add(i).write_volatile(TEST_PROGRAM0[i]);
        }
        return process;
    }
}


/**
 * Return the current thread's manager.
 */
pub unsafe fn get_manager() -> &'static mut ProcessManager {
    return &mut MANAGER[get_tid()];
}

/**
 * Return the context pointer of the current thread.
 */
pub unsafe fn get_context() -> *mut Context {
    return &mut CONTEXT[get_tid()];
}

/**
 * Allocate an available pid for the process.
 */
unsafe fn allocate_pid() -> PidType {
    return PID_POOL.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
}
