use riscv::register::*;

use crate::cpu::current_cpu;

#[no_mangle]
unsafe fn core_trap() {
    assert!(sstatus::read().spp() == sstatus::SPP::Supervisor,
        "Kernel trap from user mode. WTF?");
    let cause = scause::read().cause();
    warning!("core_trap\n");
    warning!("- cause {:?}", cause);
    warning!("- epc {:#x}", riscv::register::sepc::read());
    let process = current_cpu().get_process();
    if process.is_null() {
        panic!("Kernel process in core_trap");
    } else {
        panic!("Process {} in core_trap", (*process).pid);
    }
}
