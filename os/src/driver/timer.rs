use crate::layout::clint;
use riscv::register::*;
static mut TIME_SCRATCH: [u64 ; 5] = [0 ; 5];
extern "C" { fn time_handle(); }

#[no_mangle]
pub unsafe fn timer_init() {
    let interval = 1 << 20; // About 0.1 s on QEMU
    clint::MTIMECMP.write_volatile(
        clint::MTIME.read_volatile() + interval,
    );

    TIME_SCRATCH[3] = clint::MTIMECMP as u64;
    TIME_SCRATCH[4] = interval;

    mscratch::write(TIME_SCRATCH.as_ptr() as usize);
    mtvec::write(time_handle as usize, mtvec::TrapMode::Direct);
}
