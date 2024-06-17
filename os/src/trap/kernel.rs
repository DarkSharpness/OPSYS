use core::arch::asm;
use riscv::register::*;
use crate::{driver::plic, proc::current_cpu, trap::{set_dead_trap, set_kernel_trap}};

#[no_mangle]
unsafe fn core_trap() {
    set_dead_trap();
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

    set_kernel_trap();
}

#[no_mangle]
unsafe fn dead_trap() {
    use scause::Trap;
    warning!("Deadly trap!");

    match scause::read().cause() {
        Trap::Interrupt(interrupt) => panic!("Unhandled interrupt: {:?}", interrupt),
        Trap::Exception(exception) => panic!("Unhandled exception: {:?}", exception),
    }
}
