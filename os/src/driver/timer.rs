use riscv::register::*;

type Uptr = * mut usize;
const BASE     :       usize  = 0x2000000;
const MTIME    : * mut usize  = (BASE + 0xBFF8) as _;
const MTIMECMP : * mut usize  = (BASE + 0x4000) as _;

use crate::cpu::*;

use super::get_tid;

#[repr(C)]
pub struct TimeScartch {
    temporary   : [usize ; 3],
    mtimecmp    : usize,
    interval    : usize,
}
#[repr(C)]
pub struct Time(usize);

pub unsafe fn init() {
    extern "C" { fn time_handle(); }

    let tid = get_tid();
    let interval = 1 << 20; // About 0.1s on QEMU
    let mtimecmp = MTIMECMP.wrapping_add(tid);
    let mtime    = MTIME.wrapping_add(tid);
    let time_scratch = current_cpu().get_timer();

    // Set mtimecmp to mtime + interval
    *mtimecmp = *mtime + interval;

    time_scratch.mtimecmp = mtimecmp as _;
    time_scratch.interval = interval;

    let time_scratch = time_scratch as * mut _;

    mscratch::write(time_scratch as _);
    mtvec::write(time_handle as _, mtvec::TrapMode::Direct);

    mie::set_mtimer();
    mstatus::set_mpie();
}

unsafe fn set_timer_next() {
    let tid = get_tid();
    let mtimecmp = MTIMECMP.wrapping_add(tid);
    let mtime    = MTIME.wrapping_add(tid);
    let time_scratch = current_cpu().get_timer();
    *mtimecmp = *mtime + time_scratch.interval;
}

impl Time {
    pub fn second(s : usize) -> Self { Time(s * 10000000) }
    pub fn millisecond(ms : usize) -> Self { Time(ms * 10000) }
}

impl From<Time> for usize {
    fn from(time : Time) -> usize { time.0 }
}

impl TimeScartch {
    pub const fn new() -> Self {
        return Self {
            temporary : [0; 3],
            mtimecmp : 0,
            interval : 0,
        }
    }
}

impl CPU {
    pub fn set_timer_interval(&mut self, time : Time) {
        self.scratch.interval = usize::from(time);
    }
    pub fn reset_timer_time(&mut self) {
        return unsafe { set_timer_next(); }
    }
}
