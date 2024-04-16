use riscv::register::*;
use crate::layout::NCPU;

type Uptr = * mut u64;
const BASE     :       u64  = 0x2000000;
const MTIME    : * mut u64  = (BASE + 0xBFF8) as _;
const MTIMECMP : * mut u64  = (BASE + 0x4000) as _;

use super::get_tid;

#[repr(C)]
pub struct Timer {
    temporary       : [u64 ; 3],
    pub mtimecmp    : u64,
    pub interval    : u64,
}

pub static mut TIME_SCRATCH : [Timer; NCPU] =
    [Timer{ temporary : [0; 3], mtimecmp : 0, interval : 0, } ; NCPU];

pub unsafe fn init() {
    extern "C" { fn time_handle(); }

    let tid = get_tid();
    let interval = 1 << 22; // About 0.1s on QEMU
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

