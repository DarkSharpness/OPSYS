#![allow(dead_code)]

pub fn sbi_call(eid : usize, arg0: usize) -> usize {
    let error;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") eid,
            inlateout("a0") arg0 => error,
        );
    }
    error
}

#[no_mangle]
pub fn putchar(c : usize) {
    // #[allow(deprecated)]
    // sbi_rt::legacy::console_putchar(c);
    sbi_call(1, c);
}


pub fn shutdown(failure: bool) -> ! {
    use sbi_rt::{system_reset, NoReason, Shutdown, SystemFailure};
    if !failure {
        system_reset(Shutdown, NoReason);
    } else {
        system_reset(Shutdown, SystemFailure);
    }
    unreachable!()
}
