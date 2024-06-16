use sys::syscall::*;
use super::call::*;

#[derive(Clone)]
pub struct PidType(usize);

impl PidType {
    pub const fn new(pid : usize) -> PidType {
        PidType(pid)
    }
    pub const fn bits(&self) -> usize {
        self.0
    }
}

pub unsafe fn sys_exit(code : i32) -> ! {
    syscall1(SYS_EXIT, [code as usize]);
    panic!("unreachable in sys_exit");
}

pub unsafe fn sys_fork() -> isize {
    syscall0(SYS_FORK)
}

pub unsafe fn sys_wait() -> Option<(PidType, i32)> {
    let (ret0, ret1) = syscall0_2(SYS_WAIT);
    if ret0 == -1 {
        None
    } else {
        Some((PidType::new(ret0 as usize), ret1 as i32))
    }
}

pub unsafe fn sys_exec(path : *const u8, argv : *const *const u8, envp : *const *const u8) -> isize {
    syscall3(SYS_EXEC, [path as usize, argv as usize, envp as usize])
}

pub unsafe fn sys_shutdown() -> ! {
    syscall0(SYS_SHUTDOWN);
    loop {}
}

pub unsafe fn sys_yield() { syscall0(SYS_YIELD); }

pub unsafe fn sys_getpid() -> isize {
    syscall0(SYS_GETPID)
}

pub unsafe fn sys_sbrk(increment : isize) -> *mut u8 {
    syscall1(SYS_SBRK, [increment as usize]) as _
}
