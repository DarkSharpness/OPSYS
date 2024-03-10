type Uptr = * mut u64;

pub const BASE      : u64   = 0x2000000;
pub const MTIME     : Uptr  = (BASE + 0xBFF8) as Uptr;
pub const MTIMECMP  : Uptr  = (BASE + 0x4000) as Uptr;
