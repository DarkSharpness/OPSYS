mod node;
mod buddy;

const QEMU_ADD  : usize     = 0x80000000;
// 128 MB memory for QEMU.
const QEMU_END  : usize     = QEMU_ADD.wrapping_add(128 << 20);

// Huge page size, 2 MB each.
const HUGE_SIZE : usize     = 4096 * 512;

const PAGE_BITS : usize = 12;               // Page bits
const PAGE_SIZE : usize = 1 << PAGE_BITS;   // Page size
const WORD_BITS : usize = 8;                // byte level bitmap

const MAX_BITS  : usize = 7 + 10 + 10;      // Maximum buddy rank
const MAX_SIZE  : usize = 1 << MAX_BITS;    // Maximum buddy byte (128MB)
const MAX_RANK  : usize = MAX_BITS - PAGE_BITS; // Maximum buddy rank

const MAP_SIZE  : usize = (2 << MAX_RANK) / WORD_BITS; // Bitmap size

use core::alloc::{GlobalAlloc, Layout};
use buddy::BuddyAllocator;
use alloc::vec::Vec;
extern crate alloc;

struct Buddy;

unsafe impl GlobalAlloc for Buddy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        return BuddyAllocator::allocate(_layout.size())
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        return BuddyAllocator::deallocate(_ptr, _layout.size())
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR : Buddy = Buddy;

/* Call this function to initialize the fucking buddy system. */
pub fn init_buddy() {
    extern "C" { fn ekernel(); }

    let mut rank = 12;
    let pool = align_huge_page(ekernel as usize);
    let diff = QEMU_END - pool;

    while (1 << rank) <= diff { rank += 1; }
    rank -= 1 + PAGE_BITS;

    unsafe {
        BuddyAllocator::first_init(pool as _, rank);
        // play();
    }
}

#[inline(always)]
fn align_huge_page(num : usize) -> usize {
    return (num + HUGE_SIZE - 1) / HUGE_SIZE * HUGE_SIZE;
}

unsafe fn play() {
    let p1 = BuddyAllocator::allocate(1);
    let mut t : Vec<[i32; PAGE_SIZE * PAGE_SIZE]> = Vec::new();
    t.reserve(2);

    BuddyAllocator::debug();

    let p2 = BuddyAllocator::allocate(1);

    BuddyAllocator::debug();

    BuddyAllocator::deallocate(p1, 1);

    BuddyAllocator::debug();

    BuddyAllocator::deallocate(p2, 1);

    BuddyAllocator::debug();
}
