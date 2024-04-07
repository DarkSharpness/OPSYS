mod node;
mod page;
mod buddy;
mod frame;
mod constant;

pub use constant::PAGE_TABLE;

use constant::*;
use core::alloc::{GlobalAlloc, Layout};
use buddy::BuddyAllocator;
use alloc::vec::Vec;


use crate::{alloc::frame::FrameAllocator, logging};
extern crate alloc;

struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        return BuddyAllocator::allocate(_layout.size())
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        return BuddyAllocator::deallocate(_ptr, _layout.size())
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR : Dummy = Dummy;

/* Call this function to initialize the fucking buddy system. */
pub unsafe fn init_alloc(mem_end : usize)  {
    extern "C" { fn ekernel(); }
    assert!((ekernel as usize) <= MEMORY_START, "Kernel too big...");

    let mut rank = 12;
    let diff = mem_end - (BUDDY_START as usize);

    while (1 << rank) <= diff { rank += 1; }
    rank -= 1 + PAGE_BITS;

    init_buddy(rank);
    init_frame();
    page::init_huge_page();
    play();
}

unsafe fn play() {
    let p1 = BuddyAllocator::allocate(1);
    let mut t : Vec<[i32; PAGE_SIZE * PAGE_BITS]> = Vec::new();
    t.reserve(2);

    BuddyAllocator::debug();

    let p2 = BuddyAllocator::allocate(1);

    BuddyAllocator::debug();

    BuddyAllocator::deallocate(p1, 1);

    BuddyAllocator::debug();

    BuddyAllocator::deallocate(p2, 1);

    BuddyAllocator::debug();

    t.reserve(1 << 10);
}

unsafe fn init_frame() {
    FrameAllocator::first_init();
    logging!("Frame allocator initialized! {} Pages available!", FrameAllocator::size());
    FrameAllocator::debug();
}

unsafe fn init_buddy(rank : usize) {
    BuddyAllocator::first_init(rank);
    logging!("Buddy allocator initialized! {} MiB in all!", (PAGE_SIZE << rank) >> 20);
    BuddyAllocator::debug();
}