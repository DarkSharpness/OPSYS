use core::arch::asm;
use riscv::register::*;
use crate::{driver::plic, proc::current_cpu};

#[no_mangle]
unsafe fn core_trap() {
    assert!(sstatus::read().spp() == sstatus::SPP::Supervisor,
        "User trap from supervisor mode. WTF?");

    use scause::{Trap, Interrupt};
    match scause::read().cause() {
        Trap::Interrupt(interrupt) => match interrupt {
            Interrupt::SupervisorSoft => {
                current_cpu().reset_timer_time();
                asm!("csrci sip, 2");
            },
            Interrupt::SupervisorExternal => {
                plic::resolve();
                asm!("csrc sip, {}", in(reg) 1 << 9);
            }
            _ => panic!("Unable to resolve interrupt {:?}", interrupt),   
        },

        Trap::Exception(exception) => panic!("Unhandled exception: {:?}", exception),
    }
}
