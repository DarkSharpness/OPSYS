pub(super) fn syscall0(id : usize) -> isize {
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

pub(super) fn syscall0_2(id : usize) -> (isize, isize) {
    let mut ret0 : isize;
    let mut ret1 : isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") id,
            lateout("a0") ret0,
            lateout("a1") ret1,
        );
    }
    return (ret0, ret1);
}

pub(super) fn syscall1(id : usize, args : [usize; 1]) -> isize {
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

pub(super) fn syscall2(id : usize, args : [usize; 2]) -> isize {
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

pub(super) fn syscall3(id : usize, args : [usize; 3]) -> isize {
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