mod syscall;

pub use syscall::*;
use crate::driver::shutdown;

pub unsafe fn syscall(index : usize , a0 : usize, a1 : usize, a2 : usize) {
    use sys::syscall::*;
    let _ = index as usize + a0 + a1 + a2;
    match index {
        SYS_SHUTDOWN => shutdown(),
        SYS_SLEEP    => sys_yield(),
        _ => panic!("Unknown syscall: {}", index)
    }
}
