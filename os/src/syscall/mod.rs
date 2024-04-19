mod syscall;

pub use syscall::*;
use crate::driver::shutdown;

pub unsafe fn syscall(index : u64 , a0 : u64, a1 : u64, a2 : u64) {
    use sys::syscall::*;
    let _ = index as u64 + a0 + a1 + a2;
    match index {
        SYS_SHUTDOWN => shutdown(),
        SYS_SLEEP    => sys_yield(),
        _ => panic!("Unknown syscall: {}", index)
    }
}
