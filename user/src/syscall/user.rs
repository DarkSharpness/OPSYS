use sys::syscall::*;

pub struct FileDescriptor(isize);

fn syscall0(id : usize) -> isize {
    let mut ret : isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") id,
            lateout("a0") ret,
        );
    }
    return ret;
}

fn syscall1(id : usize, args : [usize; 1]) -> isize {
    let mut ret : isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("a0") args[0] => ret,
            in("a7") id,
        );
    }
    return ret;
}

fn syscall2(id : usize, args : [usize; 2]) -> isize {
    let mut ret : isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("a0") args[0] => ret,
            in("a1") args[1],
            in("a7") id,
        );
    }
    return ret;
}

fn syscall3(id : usize, args : [usize; 3]) -> isize {
    let mut ret : isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("a0") args[0] => ret,
            in("a1") args[1],
            in("a2") args[2],
            in("a7") id,
        );
    }
    return ret;
}

pub unsafe fn sys_exit(code : i32) -> ! {
    syscall1(SYS_EXIT, [code as usize]);
    loop {}
}

pub unsafe fn sys_fork() -> isize {
    syscall0(SYS_FORK)
}

pub unsafe fn sys_execve(path : *const u8, argv : *const *const u8, envp : *const *const u8) -> isize {
    syscall3(SYS_EXEC, [path as usize, argv as usize, envp as usize])
}

pub unsafe fn sys_shutdown() -> ! {
    syscall0(SYS_SHUTDOWN);
    loop {}
}

pub unsafe fn sys_open(path : *const u8, flags : usize) -> FileDescriptor {
    FileDescriptor(syscall2(SYS_OPEN, [path as _, flags]))
}

pub unsafe fn sys_close(fd : FileDescriptor) -> isize {
    syscall1(SYS_CLOSE, [fd.0 as _])
}

pub unsafe fn sys_write(fd : FileDescriptor, buf : *const u8, len : usize) -> isize {
    syscall3(SYS_WRITE, [fd.0 as _, buf as _, len])
}

pub unsafe fn sys_read(fd : FileDescriptor, buf : *mut u8, len : usize) -> isize {
    syscall3(SYS_READ, [fd.0 as _, buf as _, len])
}

pub unsafe fn sys_yield() { syscall0(SYS_YIELD); }
