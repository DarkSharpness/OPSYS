extern crate alloc;

use core::ptr::{addr_of, null_mut};
use core::sync::atomic::AtomicUsize;

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::str;
use alloc::vec::Vec;
use riscv::register::satp;

use crate::cpu::current_cpu;
use crate::driver::get_tid;
use crate::alloc::{PTEFlag, PageAddress, PAGE_SIZE, PAGE_TABLE};
use crate::proc::ProcessStatus;
use crate::trap::{get_trampoline, user_trap, user_trap_return, TrapFrame, TRAMPOLINE, TRAP_FRAME};
use super::{Context, PidType, Process, ProcessManager};

const USER_STACK    : usize     = 1 << 38;

/* Add the init process to the manager. */
pub unsafe fn init_process() {
    let trampoline = get_trampoline();
    PAGE_TABLE.smap(TRAMPOLINE, trampoline, PTEFlag::RX);

    let manager = current_cpu().get_manager();

    // Currently, our implementation is problematic.
    // When the queue is full, the old process will be replaced.
    // We need a deque whose iterator will not be invalidated.
    // To handle the problem here, we just reserve enough space.
    // Plan to rewrite in the future.
    manager.process_queue.reserve(32);

    manager.add_process(Process::new_test("Demo Program 0", 0));
    // manager.add_process(Process::new_test("Demo Program 1", 1));
}

impl Process {
    unsafe fn demo(name : &'static str, parent : * mut Process) -> Process {
        let root    = PageAddress::new_pagetable();

        // Map at least one page for user's stack
        let stack_page = PageAddress::new_rand_page();
        let user_stack = USER_STACK - (PAGE_SIZE as usize);
        root.umap(user_stack, stack_page, PTEFlag::RW | PTEFlag::OWNED);

        message!("Process {} created with root {:#x}", name, root.address() as usize);

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
            context, root, parent, name, trap_frame
        };
    }

    unsafe fn new_test(name : &'static str, which : usize) -> Process {
        let process = Process::demo(name, null_mut());
        let text = PageAddress::new_zero_page();
        process.root.umap(0, text, PTEFlag::RX | PTEFlag::OWNED);
        extern "C" { static _num_app : usize; }

        let num : usize = _num_app;
        assert!(which < num, "Invalid test number!");

        let addr = addr_of!(_num_app).wrapping_add(1);

        let program_start   = *addr.wrapping_add(which * 2);
        let program_finish  = *addr.wrapping_add(which * 2 + 1);

        let data : Box<[u8]> = {
            let mut data : Vec<u8> = Vec::with_capacity(program_finish - program_start);
            for ptr in program_start..program_finish {
                data.push(*(ptr as *const u8));
            }
            data.into_boxed_slice()
        };

        let elf = xmas_elf::ElfFile::new(&data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va    = ph.virtual_addr() as usize;
                let end_va      = (ph.virtual_addr() + ph.mem_size()) as usize;

                let mut permission = PTEFlag::INVALID;
                let ph_flags = ph.flags();
                if ph_flags.is_read() { permission |= PTEFlag::RO; }
                if ph_flags.is_write() { permission |= PTEFlag::RW; }
                if ph_flags.is_execute() { permission |= PTEFlag::WO; }
                message!("From {} to {}", start_va, end_va);

                todo!("map the segment to memory in page table");
            }
        }

        return process;
    }

    unsafe fn old_test(name : &'static str, which : bool) -> Process {
        let process = Process::demo(name, null_mut());
        let text = PageAddress::new_zero_page();
        process.root.umap(0, text, PTEFlag::RX | PTEFlag::OWNED);

        // Identical mapping for MMIO.
        let mmio = PageAddress::new_usize(0x10000000);
        process.root.umap(0x10000000, mmio, PTEFlag::RW);

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

    /** Return the inner context. */
    pub unsafe fn get_context(&mut self) -> &mut Context {
        return &mut self.context;
    }

    /** Sleep and set the status as given. */
    pub fn sleep_as(&mut self, status : ProcessStatus) {
        assert_eq!(self.status, ProcessStatus::RUNNING, "Invalid to sleep!");
        self.status = status;
    }

    /** Wake up from given status. */
    pub fn wake_up_from(&mut self, status : ProcessStatus) {
        assert_eq!(self.status, status, "Invalid to wake up!");
        self.status = ProcessStatus::RUNNING;
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

    unsafe fn add_process(&mut self, process : Process) {
        self.process_queue.push_back(process);
        let back = self.process_queue.back_mut().unwrap();
        register_process(back);
    }
}

static mut PID_POOL : AtomicUsize = AtomicUsize::new(0);
static mut PID_MAP  : Vec<* mut Process> = Vec::new();

/** Allocate an available pid for the process. */
unsafe fn allocate_pid() -> PidType {
    return PidType::new(PID_POOL.fetch_add(1, core::sync::atomic::Ordering::SeqCst));
}

/** Register the process to the pid map. */
unsafe fn register_process(process : * mut Process) {
    assert!(PID_MAP.len() == (*process).pid.bits());
    PID_MAP.push(process);
}

/** Get the process by the pid. */
pub unsafe fn pid_to_process(pid : PidType) -> * mut Process {
    return PID_MAP[pid.bits()];
}

/** Unregister the process from the pid map. */
pub unsafe fn unregister_process(process : * mut Process) {
    PID_MAP[(*process).pid.bits()] = null_mut();
}
