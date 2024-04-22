use riscv::register::*;

use crate::{proc::{self, get_process}, trap::Interrupt};

#[no_mangle]
unsafe fn core_trap() {
    assert!(sstatus::read().spp() == sstatus::SPP::Supervisor,
        "Kernel trap from user mode. WTF?");
    let cause = scause::read().cause();
    use scause::{Trap, Interrupt, Exception};
    warning!("core_trap\n");
    warning!("- cause {:?}", cause);
    warning!("- epc {:#x}", riscv::register::sepc::read());
    let process = get_process();
    if process.is_null() {
        panic!("Kernel process in core_trap");
    } else {
        panic!("Process {} in core_trap", (*process).pid);
    }
}
