use sys::syscall::*;

pub struct IPCHandle(usize);
pub enum Argument {
    Register(usize, usize),     // In 2 registers.
    Buffered(*mut u8, usize),   // In a user buffer.
}
pub enum IPCEnum {
    IPCFail(usize),                 // Buffer too small, give you the needed size.
    IPCAsync(Argument),             // Asynchronous IPC.
    IPCHandle(Argument, IPCHandle), // With argument and handle.
}

pub struct AcceptPacket {
    args    : [usize; 3],
    kind    : usize,
    result  : isize,
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
    pub fn parse(&self) -> IPCEnum {
        if self.result < 0 {
            return IPCEnum::IPCFail(-self.result as usize);
        }

        let argument = self.parse_argument();
        if self.result == 0 {
            return IPCEnum::IPCAsync(argument);
        } else {
            return IPCEnum::IPCHandle(argument, IPCHandle(self.result as usize));
        }
    }

    fn parse_argument(&self) -> Argument {
        match self.kind {
            ARGS_REGISTER => Argument::Register(self.args[0], self.args[1]),
            ARGS_BUFFERED => Argument::Buffered(self.args[0] as *mut u8, self.args[1]),
            _ => panic!("Unknown kind of argument."),
        }
    }
}

impl IPCHandle {
    /** Get the process id of the process who have requested. */
    pub unsafe fn get_pid(&self) -> usize { return handle_to_pid(self.0); }
}
