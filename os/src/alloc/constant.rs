/**
 * Memory frame:
 * -----------------------------------
 *  Alloc = 0x80200000
 * 
 * Alloc - 2 page:
 *  Page table here.
 *  The reason why it took 2 rather than 1 page is that
 *  loading immediate value 0x801FF000 requires one more
 *  instruction than 0x801FE000.
 * 
 * Alloc + 0 page:
 *  Buddy allocator free list here.
 * 
 * Alloc + sizeof(free_list):
 *  Frame allocator list head here.
 * 
 * Alloc + 2 page:
 *  Buddy allocator bitmap here.
 * 
 * Alloc + 8 page:
 *  Frame allocator data here.
 * 
 * -----------------------------------
 * 
 *  Base = Alloc + 0x00200000
 *       = 0x80400000
 *
 * Base + 0 page:
 *  Buddy allocator base address here.
 * 
 * -----------------------------------
 */

use crate::alloc::node::List;

// A tiny helper funciton
const fn align_as(usize: usize, align: usize) -> usize {
    return (usize + align - 1) & !(align - 1);
}

pub const PAGE_BITS : usize = 12;               // Page bits
pub const PAGE_SIZE : usize = 1 << PAGE_BITS;   // Page size
pub const WORD_BITS : usize = 8;                // byte level bitmap

pub const MAX_BITS  : usize = 7 + 10 + 10;      // Maximum buddy rank
pub const MAX_SIZE  : usize = 1 << MAX_BITS;    // Maximum buddy byte (128MB)
pub const MAX_RANK  : usize = MAX_BITS - PAGE_BITS; // Maximum buddy rank

// Buddy allocator data structure
pub const ALLOC  :  usize       = 0x80200000;
// Buddy allocator rank list address
pub const RKLIST : *mut List    = ALLOC as _;
// Buddy allocator bitmap address
pub const BITMAP : *mut u8      = ALLOC.wrapping_add(2 * PAGE_SIZE) as _;

// The word that the bitmap takes.
pub const MAP_SIZE  : usize = (2 << MAX_RANK) / WORD_BITS;

// Buddy allocator base address.
pub const BUDDY_START   : *mut u8   = (ALLOC + 0x00200000) as _;

// End of bitmap, aligned to 8 pages.
pub const FRAME_START   : *mut u16  =
    (ALLOC + align_as(PAGE_SIZE + MAP_SIZE * WORD_BITS / 8, 8 * PAGE_SIZE)) as _;

// The page table physical address.
pub const PAGE_TABLE : usize = ALLOC - PAGE_SIZE * 2;

// The lowest memory reserved for memory management.
pub const MEMORY_START : usize = PAGE_TABLE;
