use super::page::PageAddress;

/**
 * Physical memory frame:
 * -----------------------------------
 * Alloc = 0x80200000
 * 
 * Alloc + 0 page:
 *  Buddy allocator free list here.
 * 
 * Alloc + 2 page:
 *  Kernel page table here.
 *  The reason why it is aligned by 2 page is that
 *  8KB aligned immediate can be loaded in 2 instructions,
 *  while 4KB aligned immediate needs 3 instructions.
 * 
 * Alloc + 4 page:
 *  Buddy allocator bitmap here.
 * 
 * Alloc + 8 page:
 *  Frame allocator data here.
 *
 *  -----------------------------------
 * Base = Alloc + 0x00200000
 *       = 0x80400000
 *
 * Base + 0 page:
 *  Buddy allocator base address here.
 * 
 * -----------------------------------
 * Summary:
 * 
 * Allocator only needs R/W permission!
 * 
 * Frame allocator:
 *  [0x80208000, 0x80400000)
 * 
 * Buddy allocator:
 *  [0x80200000, 0x80202000) free list
 *  [0x80204000, 0x80206000) bitmap
 *  [0x80400000, end of mem) base address
 * 
 * Page table:
 *  [0x80202000, 0x80204000)
 * 
 * Allocator begin at 0x80200000.
 *  [0x80200000, end of mem) 
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
pub const BITMAP : *mut u8      = (ALLOC + 4 * PAGE_SIZE) as _;

// The word that the bitmap takes.
pub const MAP_SIZE  : usize = (2 << MAX_RANK) / WORD_BITS;

// Buddy allocator base address.
pub const BUDDY_START_ADDR : usize  = ALLOC + 0x00E00000;
pub const BUDDY_START   : *mut u8   = BUDDY_START_ADDR as _;

// End of bitmap, aligned to 8 pages.
// Start of the fram allocator.
pub const FRAME_START_ADDR : usize  =
    ALLOC + align_as(PAGE_SIZE + MAP_SIZE * WORD_BITS / 8, 8 * PAGE_SIZE);
pub const FRAME_START   : *mut u16  = FRAME_START_ADDR as _;

// The page table physical address.
pub const PAGE_TABLE : PageAddress = PageAddress::new_u64((ALLOC + PAGE_SIZE * 2) as _);

// The lowest memory reserved for memory management.
pub const MEMORY_START : usize = ALLOC;
