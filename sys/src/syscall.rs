#![allow(dead_code)]

pub const SYS_FORK      : usize   = 1;
pub const SYS_EXEC      : usize   = 2;
pub const SYS_EXIT      : usize   = 3;
pub const SYS_WAIT      : usize   = 4;
pub const SYS_OPEN      : usize   = 5;
pub const SYS_CLOSE     : usize   = 6;
pub const SYS_READ      : usize   = 7;
pub const SYS_WRITE     : usize   = 8;
pub const SYS_YIELD     : usize   = 9;

pub const SYS_REQUEST   : usize   = 10;
pub const SYS_RECEIVE   : usize   = 11;
pub const SYS_RESPOND   : usize   = 12;

pub const SYS_SHUTDOWN  : usize   = 114;

pub const PM_PORT : usize = 0;
pub const PM_EXIT : usize = 0;
pub const PM_FORK : usize = 1;
pub const PM_EXEC : usize = 2;
pub const PM_WAIT : usize = 3;

const MAGIC : usize = 1919;

pub unsafe fn pid_to_handle(x : usize) -> usize { x + MAGIC }
pub unsafe fn handle_to_pid(x : usize) -> usize { x - MAGIC }

pub const ARGS_REGISTER : usize = 0; // Argument * 2
pub const ARGS_BUFFERED : usize = 1; // Buffer + Length
