mod proc;
mod schedule;

pub use proc::current_process;

pub unsafe fn init_proc() {
    proc::init_process();
}

/// Top of user's stack
pub const USER_STACK : u64 = 1 << 38;
