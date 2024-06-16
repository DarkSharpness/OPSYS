use sys::syscall::*;
use super::call::*;

#[derive(Clone, PartialEq, Eq)]
pub struct PidType(usize);

pub enum WaitResult {
    Some(PidType, i32),
    None,
    Error,
}

pub enum ForkResult {
    Parent(PidType),
    Child,
    Error,
}

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

pub unsafe fn sys_fork() -> ForkResult {
    let ret = syscall0(SYS_FORK);
    if ret == -1 {
        ForkResult::Error
    } else if ret == 0{
        ForkResult::Child
    } else {
        ForkResult::Parent(PidType::new(ret as _))
    }
}

pub unsafe fn sys_wait() -> WaitResult {
    let (ret0, ret1) = syscall0_2(SYS_WAIT);
    if ret0 == -1 {
        WaitResult::Error
    } else if ret0 == 0 {
        WaitResult::None
    } else {
        WaitResult::Some(PidType::new(ret0 as _), ret1 as i32)
    }
}

pub unsafe fn sys_exec(name : &[u8], argv : *const *const u8) -> isize {
    let buf = name.as_ptr();
    let len = name.len();
    syscall3(SYS_EXEC, [buf as _, len, argv as _])
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
