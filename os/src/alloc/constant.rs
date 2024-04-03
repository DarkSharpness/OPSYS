use crate::alloc::node::*;

// Buddy allocator data structure
pub const ALLOC  :  usize       = 0x80200000;
// Buddy allocator rank list address
pub const RKLIST : *mut List    = ALLOC as _;
// Buddy allocator bitmap address
pub const BITMAP : *mut u8      = ALLOC.wrapping_add(PAGE_SIZE) as _;
// Buddy allocator base address.
pub const BASE   : *mut u8      = (ALLOC + 0x00200000) as _;

pub const PAGE_BITS : usize = 12;               // Page bits
pub const PAGE_SIZE : usize = 1 << PAGE_BITS;   // Page size
pub const WORD_BITS : usize = 8;                // byte level bitmap

pub const MAX_BITS  : usize = 7 + 10 + 10;      // Maximum buddy rank
pub const MAX_SIZE  : usize = 1 << MAX_BITS;    // Maximum buddy byte (128MB)
pub const MAX_RANK  : usize = MAX_BITS - PAGE_BITS; // Maximum buddy rank

pub const MAP_SIZE  : usize = (2 << MAX_RANK) / WORD_BITS; // Bitmap size
