use crate::alloc::node::List;
use super::page::PageAddress;

pub const PAGE_BITS : usize = 12;               // Page bits
pub const PAGE_SIZE : usize = 1 << PAGE_BITS;   // Page size
pub const WORD_BITS : usize = 8;                // byte level bitmap

pub const MAX_BITS  : usize = 7 + 10 + 10;          // Maximum buddy rank
pub const MAX_SIZE  : usize = 1 << MAX_BITS;        // Maximum buddy byte (128MB)
// pub
const MAX_RANK  : usize = MAX_BITS - PAGE_BITS; // Maximum buddy rank
pub const TOP_RANK  : usize = MAX_RANK + 1;     // Unreachable rank

// The word that the bitmap takes.
pub const MAP_SIZE  : usize = (2 << TOP_RANK) / WORD_BITS;

pub static mut RKLIST   : [List; MAX_RANK + 1]  = [List::new(); TOP_RANK];
pub static mut BITMAP   : [u8; MAP_SIZE]        = [0; MAP_SIZE];

// Buddy allocator data structure
pub const BASE_ADDRESS  :  usize    = 0x80000000;
// Buddy allocator base address.
pub const BUDDY_START   : *mut u8   = BASE_ADDRESS as _;

// Page table address.
pub const PAGE_TABLE_ADDR   : usize     = BASE_ADDRESS + PAGE_SIZE * 2;
// The page table physical address.
pub const PAGE_TABLE    : PageAddress   = PageAddress::new_usize(PAGE_TABLE_ADDR as _);
