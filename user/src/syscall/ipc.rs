use sys::syscall::*;

#[derive(Copy, Clone)]
pub struct IPCHandle(usize);
pub struct AcceptPacket {
    pub kind    : usize,
    handle      : IPCHandle,    // Not Visible
    pub args    : [usize; 3],   // Not Visible
}

pub fn sys_request(port : usize, kind : usize, args : [usize; 3]) -> isize {
    let mut ret : isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a0") args[0],
            in("a1") args[1],
            in("a2") args[2],
            in("a4") kind,
            in("a6") port,
            in("a7") SYS_REQUEST,
            lateout("a0") ret,
        );
    }
    return ret;
}

pub fn sys_accept(port : usize) -> AcceptPacket {
    let mut kind    : usize;
    let mut handle  : usize;
    let mut arg0    : usize;
    let mut arg1    : usize;
    let mut arg2    : usize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a6") port,
            in("a7") SYS_ACCEPT,
            lateout("a0") kind,
            lateout("a1") handle,
            lateout("a2") arg0,
            lateout("a3") arg1,
            lateout("a4") arg2,
        );
    }
    return AcceptPacket {
        kind, handle: IPCHandle(handle), args: [arg0, arg1, arg2]
    };
}

pub fn sys_transfer(port : usize, kind : usize, handle : IPCHandle, args : [usize; 3]) -> isize {
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
            in("a7") SYS_TRANSFER,
            lateout("a0") ret,
        );
    }
    return ret;
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
