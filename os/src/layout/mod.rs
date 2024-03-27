pub mod clint;

// Count of CPU cores
pub const NCPU : usize = 1;

// Place of user's trampoline (both in user space & kernel space).
// Actually, since our kernel feat no page table,
// This is a physical address [0x80001000, 0x80002000)
pub const TRAMPOLINE : usize = 0x80001000;
