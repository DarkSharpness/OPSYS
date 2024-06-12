mod node;
mod page;
mod buddy;
mod constant;
mod page_impl;
mod page_mmap;
mod page_copy;

pub use constant::KERNEL_SATP;
pub use constant::PAGE_SIZE;
pub use page::PageAddress;
pub use page::PTEFlag;

use constant::*;
use core::alloc::{GlobalAlloc, Layout};
use core::arch::global_asm;
use core::cmp::max;
use buddy::BuddyAllocator;
use alloc::vec::Vec;

use crate::{console::print_separator, driver::get_mem_end};
extern crate alloc;

global_asm!(include_str!("alloc.asm"));

struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = max(layout.size(), layout.align());
        return BuddyAllocator::allocate(size);
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = max(layout.size(), layout.align());
        return BuddyAllocator::deallocate(ptr, size);
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR : Dummy = Dummy;

/**
 * Call this function to initialize the buddy system.
 * It will automatically set up the page table.
 */
pub unsafe fn init(mem_end : usize)  {
    extern "C" { fn ekernel(); }
    BuddyAllocator::first_init(ekernel as _, mem_end);
    // logging!("Buddy allocator initialized! {} MiB in all!", (PAGE_SIZE << rank) >> 20);
    page::init_page_table();
}

/** A demo play function after the initialization. */
pub unsafe fn demo() {
    let p1 = BuddyAllocator::allocate(1);
    let mut t : Vec<[i32; PAGE_SIZE * PAGE_BITS]> = Vec::new();
    t.reserve(2);

    BuddyAllocator::debug();

    // let p2 = BuddyAllocator::allocate(1);

    // BuddyAllocator::debug();

    BuddyAllocator::deallocate(p1, 1);

    // BuddyAllocator::debug();

    // BuddyAllocator::deallocate(p2, 1);

    drop(t);

    BuddyAllocator::debug();

    // t.reserve(1 << 10); // This function will panic

    warning!("End of allocator demo!");

    print_separator();
}

/** Display the memory usage of the allocator. */
pub unsafe fn display() { BuddyAllocator::debug(); }
