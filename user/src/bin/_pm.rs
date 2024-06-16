#![no_std]
#![no_main]

mod pm;
use pm::*;
use sys::syscall::{PM_DUMP, PM_EXIT, PM_FORK, PM_PORT, PM_WAIT};
use user_lib::{println, sys_receive, sys_respond, Argument, IPCEnum, IPCHandle, IPCKind};

#[no_mangle]
unsafe fn main() -> i32 {
    let args = [0; 3];
    let package = sys_receive(args, PM_PORT);
    match package.parse() {
        IPCEnum::IPCHandle(argument, kind, handle) => {
            handle_user_request(argument, kind, handle);
        },
        IPCEnum::IPCAsync(argument, kind) => {
            handle_async_request(argument, kind);
        },
        _ => todo!("Not implemented yet!")
    }
    return main();
}

fn handle_user_request(argument : Argument, kind : IPCKind, handle: IPCHandle) {
    match kind {
        PM_FORK => handle_fork(argument, handle),
        PM_EXIT => handle_exit(argument, handle),
        PM_WAIT => handle_wait(argument, handle),   
        _ => todo!("Not implemented yet!")
    }
}

fn handle_async_request(argument : Argument, kind : IPCKind) {
    match kind {
        PM_DUMP => process_dump(argument),
        _ => todo!("Not implemented yet!")
    }
}

fn handle_fork(argument : Argument, handle: IPCHandle) {
    let (x0, _) = match argument {
        Argument::Buffered(_, _) => panic!("Should not have any arguments!"),
        Argument::Register(x0, x1) => (x0, x1) 
    };

    let parent_pid = unsafe { handle.get_pid().bits() };
    let child_pid  = x0;

    println!("-- Received fork request from {} to {} --", parent_pid, child_pid);

    unsafe {
        let parent = get_node(parent_pid);
        let child  = get_node(child_pid);
        (*parent).insert_child(child);
    }
    sys_respond(Argument::Register(0, 0), handle);
}

fn handle_exit(argument : Argument, handle: IPCHandle) {
    let (x0, _) = match argument {
        Argument::Buffered(_, _) => panic!("Should not have any arguments!"),
        Argument::Register(x0, x1) => (x0, x1) 
    };

    let pid = unsafe { handle.get_pid().bits() };
    let exit_code = x0 as _;

    println!("-- Received exit request from {} with code {} --", pid, exit_code);

    let node = unsafe { get_node(pid) };
    node.exit(exit_code);
    sys_respond(Argument::Register(0, 0), handle);
}

fn handle_wait(argument : Argument, handle: IPCHandle) {
    let _ = argument; // Unused
    let pid = unsafe { handle.get_pid().bits() };
    let node = unsafe { get_node(pid) };

    println!("-- Received wait request from {} --", pid);

    node.wait(handle);
}

fn process_dump(argument : Argument) {
    let _ = argument; // Unused
    println!("-- Received dump request --");
    return pm_dump();
}
