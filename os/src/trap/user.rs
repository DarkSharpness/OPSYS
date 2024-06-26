use core::arch::asm;
use riscv::register::*;
use crate::alloc::PageAddress;
use crate::cpu::current_cpu;
use crate::driver::plic;
use crate::trap::exception::PageFaultType;
use crate::trap::{set_kernel_trap, set_user_trap};
use super::{user_handle, user_return, Interrupt, TRAMPOLINE};

/**
 * Function called from user_handle
 */
#[no_mangle]
pub unsafe fn user_trap() {
    assert!(sstatus::read().spp() == sstatus::SPP::User,
        "User trap from supervisor mode. WTF?");

    // Set the trap vector to the supervisor vector
    set_kernel_trap();
  
    // extern "C" { fn fault_test(); }
    // fault_test();

    let cpu = current_cpu();
    let process = &mut (*cpu.get_process());

    use scause::{Trap, Interrupt, Exception};
    match scause::read().cause() {
        Trap::Interrupt(interrupt) => match interrupt {
            // This is the time interrupt transfered
            // from time_handle.
            // We should yield out the time.
            Interrupt::SupervisorSoft => {
                asm!("csrci sip, 2");
                process.yield_to_scheduler();
            },
            Interrupt::SupervisorExternal => {
                // Acknowledge the external interrupt
                plic::resolve();
                asm!("csrc sip, {}", in(reg) 1 << 9);
            },
            _ => panic!("Unable to resolve interrupt {:?}", interrupt),   
        },

        Trap::Exception(exception) => match exception {
            Exception::UserEnvCall => {
                // Load out the syscall id in a7
                // Load out the arguments in a0, a1, a2
                cpu.syscall();
            },
            Exception::StorePageFault => {
                let addr = stval::read();
                process.handle_page_fault(addr, PageFaultType::Store);
            },
            Exception::LoadPageFault => {
                let addr = stval::read();
                process.handle_page_fault(addr, PageFaultType::Load);
            },
            Exception::InstructionPageFault => {
                let addr = stval::read();
                process.handle_page_fault(addr, PageFaultType::Instruction);
            },
            _ => panic!("Unable to resolve exception {:?}", exception),
        }
    }

    // TODO: Load the satp register of the user
    return user_trap_return();
}

pub unsafe fn user_trap_return() {
    /* Prepare to go back to user, so just set spie bit. */
    Interrupt::disable();
    sstatus::set_spp(sstatus::SPP::User);
    sstatus::set_spie();

    /* Set the trap vector back to user vector */
    set_user_trap();

    let process = &mut (*current_cpu().get_process());
    sepc::write(process.get_trap_frame().pc as _);

    return return_to_user(process.get_satp());
}

unsafe fn return_to_user(base : PageAddress) {
    let satp = base.bits() | (8 << 60); // Sv39
    let func = TRAMPOLINE + (user_return as usize - user_handle as usize);

    // message!("Returning to user space with satp: {:#x}", satp);
    // message!("Returning to user space with addr: {:#x}", func);

    type CallType = fn(usize);
    let ptr  = &func as *const _;
    let ptr  = ptr as *const CallType;

    return (*ptr)(satp);
}
