mod proc;
mod schedule;

pub use proc::*;
pub use schedule::run_process;

/// Top of user's stack
pub const USER_STACK : usize = 1 << 38;
