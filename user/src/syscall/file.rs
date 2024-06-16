use super::call::*;
use sys::syscall::*;

#[derive(Clone, Copy)]
pub struct FileDescriptor(usize);

impl FileDescriptor {
    pub(crate) const fn new(fd : usize) -> FileDescriptor {
        return FileDescriptor(fd);
    }
    // Debug use only. Do not use in production code.
    pub unsafe fn new_debug(fd : usize) -> FileDescriptor {
        return FileDescriptor(fd);
    }
}

pub unsafe fn sys_open(path : *const u8, flags : usize) -> Option<FileDescriptor> {
    let result = syscall2(SYS_OPEN, [path as usize, flags]);
    if result < 0 {
        return None;
    } else {
        return Some(FileDescriptor(result as usize));
    }
}

pub unsafe fn sys_close(fd : FileDescriptor) -> isize {
    syscall1(SYS_CLOSE, [fd.0 as _])
}

pub unsafe fn sys_write(fd : FileDescriptor, buf : &[u8]) -> isize {
    syscall3(SYS_WRITE, [fd.0 as _, buf.as_ptr() as _, buf.len()])
}

pub unsafe fn sys_read(fd : FileDescriptor, buf : &mut [u8]) -> isize {
    syscall3(SYS_READ, [fd.0 as _, buf.as_mut_ptr() as _, buf.len()])
}

