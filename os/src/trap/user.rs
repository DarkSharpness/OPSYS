use core::arch;
use riscv::register::*;
use crate::{alloc::PageAddress, proc::current_process, trap::{set_kernel_trap, set_user_trap}};

use super::{user_handle, user_return, Interrupt, TRAMPOLINE};

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
    // return user_trap_return(satp);
}

pub unsafe fn user_trap_return() {
    /* Prepare to go back to user, so just set spie bit. */
    Interrupt::disable();
    sstatus::set_spp(sstatus::SPP::User);
    sstatus::set_spie();

    /* Set the trap vector back to user vector */
    set_user_trap();

    let process = &mut *current_process();
    sepc::write((*process.trap_frame).pc as _);

    return return_to_user(process.root);
}

unsafe fn return_to_user(base : PageAddress) {
    let satp = base.bits() | (8 << 60); // Sv39
    let func = TRAMPOLINE + (user_return as u64 - user_handle as u64);

    message!("Returning to user space with satp: {:#x}", satp);
    message!("Returning to user space with addr: {:#x}", func);

    type CallType = fn(u64);
    let ptr  = &func as *const _;
    let ptr  = ptr as *const CallType;

    return (*ptr)(satp);
}
