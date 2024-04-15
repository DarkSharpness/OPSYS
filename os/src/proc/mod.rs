mod proc;
mod schedule;

pub unsafe fn init_proc() {
    proc::init_process();
}

/// Top of user's stack
pub const USER_STACK : u64 = 1 << 38;
