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

pub const SYS_SBRK      : usize   = 13;
pub const SYS_GETPID    : usize   = 14;
pub const SYS_KILL      : usize   = 15;

pub const SYS_EXEC_NO   : usize   = 100; // A debug use syscall

pub const SYS_SHUTDOWN  : usize   = 114;

pub const PM_PORT : usize = 0;
pub const PM_EXIT : usize = 0;
pub const PM_FORK : usize = 1;
pub const PM_EXEC : usize = 2;
pub const PM_WAIT : usize = 3;
pub const PM_DUMP : usize = 9;

pub const PM_MUTEX_CREATE   : usize = 10;
pub const PM_MUTEX_DESTROY  : usize = 11;
pub const PM_MUTEX_LOCK     : usize = 12;
pub const PM_MUTEX_UNLOCK   : usize = 13;

pub const PM_COND_CREATE    : usize = 14;
pub const PM_COND_DESTROY   : usize = 15;
pub const PM_COND_WAIT      : usize = 16;
pub const PM_COND_SIGNAL    : usize = 17;
pub const PM_COND_BROADCAST : usize = 18;

const MAGIC : usize = 1919;

pub unsafe fn pid_to_handle(x : usize) -> usize { x + MAGIC }
pub unsafe fn handle_to_pid(x : usize) -> usize { x - MAGIC }

pub const ARGS_REGISTER : usize = 0; // Argument * 2
pub const ARGS_BUFFERED : usize = 1; // Buffer + Length
