mod node;
mod buddy;
mod frame;
mod constant;

use constant::*;
use core::alloc::{GlobalAlloc, Layout};
use buddy::BuddyAllocator;
use alloc::vec::Vec;

use crate::{alloc::frame::FrameAllocator, debug::print_separator, uart_println};
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
    assert!((ekernel as usize) <= ALLOC, "Invalid setting");

    let mut rank = 12;
    let diff = mem_end - (BASE as usize);

    while (1 << rank) <= diff { rank += 1; }
    rank -= 1 + PAGE_BITS;

    BuddyAllocator::first_init(rank);
    uart_println!("Buddy allocator initialized! {} MiB in all!", (PAGE_SIZE << rank) >> 20);
    BuddyAllocator::debug();

    print_separator();

    FrameAllocator::first_init();
    uart_println!("Frame allocator initialized! {} Pages available!", FrameAllocator::size());
    FrameAllocator::debug();
    print_separator();
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
}
