#![allow(dead_code)]

pub const SYS_FORK      : usize   = 1;
pub const SYS_EXEC      : usize   = 2;
pub const SYS_EXIT      : usize   = 3;
pub const SYS_SHUTDOWN  : usize   = 4;
pub const SYS_OPEN      : usize   = 5;
pub const SYS_CLOSE     : usize   = 6;
pub const SYS_READ      : usize   = 7;
pub const SYS_WRITE     : usize   = 8;
pub const SYS_YIELD     : usize   = 9;

// Higher 32 bits are for IPC syscalls.
pub const SYS_REQUEST   : usize   = 1;
pub const SYS_ACCEPT    : usize   = 2;
pub const SYS_TRANSFER  : usize   = 3;
pub const SYS_RESPONSE  : usize   = 4;
