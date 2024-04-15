mod proc;
mod schedule;

pub use proc::current_process;
pub use proc::init_process;
pub use schedule::run_process;

/// Top of user's stack
pub const USER_STACK : u64 = 1 << 38;
