use riscv::register::*;
use crate::layout::NCPU;

type Uptr = * mut usize;
const BASE     :       usize  = 0x2000000;
const MTIME    : * mut usize  = (BASE + 0xBFF8) as _;
const MTIMECMP : * mut usize  = (BASE + 0x4000) as _;

use super::get_tid;

#[repr(C)]
struct Timer {
    temporary       : [usize ; 3],
    pub mtimecmp    : usize,
    pub interval    : usize,
}

static mut TIME_SCRATCH : [Timer; NCPU] =
    [Timer{ temporary : [0; 3], mtimecmp : 0, interval : 0, } ; NCPU];

pub unsafe fn init() {
    extern "C" { fn time_handle(); }

    let tid = get_tid();
    let interval = 1 << 20; // About 0.1s on QEMU
    let mtimecmp = MTIMECMP.wrapping_add(tid);
    let mtime    = MTIME.wrapping_add(tid);
    let time_scratch = &mut TIME_SCRATCH[tid];

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

pub struct Time(usize);

pub unsafe fn set_timer_interval(interval : Time) {
    let tid = get_tid();
    let time_scratch = &mut TIME_SCRATCH[tid];
    time_scratch.interval = interval.0;
}

pub unsafe fn set_timer_next() {
    let tid = get_tid();
    let mtimecmp = MTIMECMP.wrapping_add(tid);
    let mtime    = MTIME.wrapping_add(tid);
    *mtimecmp = *mtime + TIME_SCRATCH[tid].interval;
}

impl Time {
    pub fn second(s : usize) -> Self { Time(s * 10000000) }
    pub fn millisecond(ms : usize) -> Self { Time(ms * 10000) }
}
