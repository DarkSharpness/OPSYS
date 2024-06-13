use sys::syscall::*;

pub struct IPCHandle(usize);
pub enum IPCEnum {
    IPCAsync,
    IPCHandle(IPCHandle),
    IPCRemain(usize),
}

pub struct AcceptPacket {
    pub args    : [usize; 3],
    pub kind    : usize,
    result      : isize,    // Not Visible
}

pub fn sys_request(args : [usize; 3], port : usize, kind : usize) -> isize {
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

pub fn sys_receive(args : [usize; 3], port : usize) -> AcceptPacket {
    let mut kind    : usize;
    let mut result  : isize;
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
            in("a7") SYS_RECEIVE,
            lateout("a0") arg0,
            lateout("a1") arg1,
            lateout("a2") arg2,
            lateout("a4") kind,
            lateout("a5") result,
        );
    }
    return AcceptPacket {
        args: [arg0, arg1, arg2], kind, result
    };
}

pub fn sys_respond(args : [usize; 3], handle : IPCHandle) -> isize {
    let mut ret : isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a0") args[0],
            in("a1") args[1],
            in("a2") args[2],
            in("a5") handle.0,
            in("a7") SYS_RESPOND,
            lateout("a0") ret,
        );
    }
    return ret;
}

impl AcceptPacket {
    pub fn parse_result(&self) -> IPCEnum {
         if self.result == 0 {
            return IPCEnum::IPCAsync;
        } else if self.result < 0 {
            return IPCEnum::IPCRemain(-self.result as usize);
        } else {
            return IPCEnum::IPCHandle(IPCHandle(self.result as usize));
        }
    }
}
