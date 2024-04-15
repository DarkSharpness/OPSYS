use core::arch;
use riscv::register::*;
use crate::trap::set_kernel_trap;

use super::user_return;


/**
 * Function called from user_handle
 */
#[no_mangle]
pub unsafe fn user_trap() {
    if sstatus::read().spp() != sstatus::SPP::User {
        panic!("User trap from supervisor mode");
    }

    // Set the trap vector to the supervisor vector
    set_kernel_trap();

    use scause::{Trap, Interrupt, Exception};
    match scause::read().cause() {
        Trap::Interrupt(interrupt) => match interrupt {
            // This is the time interrupt transfered
            // from time_handle.
            // We should yield out the time.
            Interrupt::SupervisorSoft => {
                arch::asm!("csrci sip, 2");

                // todo!("Yield out the time interrupt here");
            },
            Interrupt::SupervisorExternal => {
                // Acknowledge the external interrupt
                let tmp = sip::read().bits();
                arch::asm!("csrw sip, {}", in(reg) tmp & !(1 << 9));

                todo!("Resolve the external interrupt");
            }

            _ => panic!("Unable to resolve interrupt {:?}", interrupt),   
        },

        Trap::Exception(exception) => match exception {
            Exception::UserEnvCall => {
                // Load out the syscall id in a7
                // Load out the arguments in a0, a1, a2

                todo!("Handle the user syscall");
            }
            _ => panic!("Unable to resolve exception {:?}", exception),
        }
    }

    // TODO: Load the satp register of the user
    // return user_return(satp);
}
