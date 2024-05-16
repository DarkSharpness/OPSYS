use core::arch::asm;
use riscv::register::*;
use crate::driver::plic;

#[no_mangle]
unsafe fn core_trap() {
    assert!(sstatus::read().spp() == sstatus::SPP::Supervisor,
        "User trap from supervisor mode. WTF?");

    use scause::{Trap, Interrupt};
    match scause::read().cause() {
        Trap::Interrupt(interrupt) => match interrupt {
            Interrupt::SupervisorSoft => {
                // Simply acknowledge the software interrupt
                asm!("csrci sip, 2");
            },
            Interrupt::SupervisorExternal => {
                // Acknowledge the external interrupt
                plic::resolve();
                asm!("csrc sip, {}", in(reg) 1 << 9);
            }
            _ => panic!("Unable to resolve interrupt {:?}", interrupt),   
        },

        Trap::Exception(exception) => panic!("Unhandled exception: {:?}", exception),
    }
}
