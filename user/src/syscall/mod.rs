#![allow(dead_code)]

unsafe fn syscall(id : usize, args : [usize; 3]) -> isize {
    let mut ret : isize;
    core::arch::asm!(
        "ecall",
        inlateout("a0") args[0] => ret,
        in("a1") args[1],
        in("a2") args[2],
        in("a7") id,
    );
    return ret
}
