use sys::syscall::*;

#[derive(Copy, Clone)]
pub struct IPCHandle(usize);
pub struct AcceptPacket {
    pub args    : [usize; 3],   // Not Visible
    pub kind    : usize,
    handle      : IPCHandle,    // Not Visible
}

pub fn sys_transfer(args : [usize; 3], port : usize, kind : usize, handle : IPCHandle) -> isize {
    let mut ret : isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a0") args[0],
            in("a1") args[1],
            in("a2") args[2],
            in("a4") kind,
            in("a5") handle.0,
            in("a6") port,
            in("a7") SYS_REQUEST,
            lateout("a0") ret,
        );
    }
    return ret;
}

pub fn sys_request(args : [usize; 3], port : usize, kind : usize) -> isize {
    return sys_transfer(args, port, kind, IPCHandle::dummy());
}

pub fn sys_accept(args : [usize; 3], port : usize) -> AcceptPacket {
    let mut kind    : usize;
    let mut handle  : usize;
    let mut arg0    : usize;
    let mut arg1    : usize;
    let mut arg2    : usize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a0") args[0],
            in("a1") args[1],
            in("a2") args[2],
            in("a6") port,
            in("a7") SYS_ACCEPT,
            lateout("a0") arg0,
            lateout("a1") arg1,
            lateout("a2") arg2,
            lateout("a4") kind,
            lateout("a5") handle,
        );
    }
    return AcceptPacket {
        args: [arg0, arg1, arg2],
        kind, handle: IPCHandle(handle)
    };
}

pub fn sys_response(handle : IPCHandle) -> isize {
    let mut ret : isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a0") handle.0,
            in("a7") SYS_RESPONSE,
            lateout("a0") ret,
        );
    }
    return ret;
}

impl AcceptPacket {
    /** The only method to access the handle. */
    pub fn get_handle(&self) -> IPCHandle { return self.handle; }
}

impl IPCHandle {
    fn dummy() -> IPCHandle { return IPCHandle(0); }
}
