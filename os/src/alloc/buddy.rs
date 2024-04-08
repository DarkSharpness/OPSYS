
use crate::{alloc::node::*, console::print_separator};

use super::constant::*;
pub struct BuddyAllocator;

unsafe fn get_rank(mut size : usize) -> usize {
    let mut rank = 0;
    while PAGE_SIZE < size { size >>= 1; rank += 1; }
    return rank;
}

#[inline(always)]
unsafe fn rklist(idx : usize) -> *mut List {
    return RKLIST.add(idx);
}

#[inline(always)]
unsafe fn mask(rank : usize) -> usize {
    return 1 << (MAX_RANK - rank - 1);
}

/* Divide and mod operation to get word and offset. */
#[inline(always)]
unsafe fn div_mod(num : usize) -> (* mut u8, usize) {
    return (BITMAP.add(num / WORD_BITS), num % WORD_BITS);
}

/* Remove the buddy for the free list. */
#[inline(always)]
unsafe fn remove_buddy(num : usize, rank : usize) {
    unlink(set_index(num ^ 1, rank) as _);
}

/* Find the first non-empty list. */
#[inline(always)]
unsafe fn find_first(rank : usize) -> usize{
    let mut ret = rank;
    loop {
        if rank >= MAX_RANK { panic!("Out of memory!"); }
        if !(*rklist(ret)).empty() { break; }
        ret += 1;
    } return ret;
}

/* Return the given number of start address and rank.  */
unsafe fn get_index(ptr : *mut u8, rank : usize) -> usize {
    let addr = BUDDY_START;
    let bias = (ptr as usize - addr as usize) / PAGE_SIZE;
    return bias >> rank | mask(rank);
}

/* Return the start address of given number and rank. */
unsafe fn set_index(num : usize, rank : usize) -> *mut u8 {
    let addr = BUDDY_START;
    let bias = ((num & (mask(rank) - 1)) << rank) * PAGE_SIZE;
    return addr.wrapping_add(bias);
}

/* Set a bit as busy. */
unsafe fn set_busy(num : usize) {
    let (ptr, bit) = div_mod(num);
    ptr.write(ptr.read() & !(1 << bit));
}

/* Set a bit as busy. */
unsafe fn set_free(num : usize) {
    let (ptr, bit) = div_mod(num);
    ptr.write(ptr.read() | 1 << bit);
}

/* Test and set this bit and buddy bit accordingly.  */
unsafe fn test_and_set(num : usize, rank : usize) -> bool {
    let (ptr, bit) = div_mod(num);
    let bud = bit ^ 1;
    let val = ptr.read();
    if (val & (1 << bud)) != 0 {
        // If buddy free, merge into larger block.
        ptr.write(val ^ (1 << bud));
        remove_buddy(num, rank);
        return true;
    } else {
        // If buddy busy, just set this bit only.
        ptr.write(val | (1 << bit));
        return false;
    }
}

/* Try to allocate memory for buddy allocator. */
unsafe fn try_alloc(rank : usize) -> *mut u8 {
    let mut top = find_first(rank);
    let ptr = (*rklist(top)).pop() as *mut u8;

    let mut num = get_index(ptr, top);
    set_busy(num);

    while rank < top {
        top -= 1;   // Split the buddy
        num <<= 1;  // Left child

        let old = ptr.add(PAGE_SIZE << top);
        (*rklist(top)).push(old as _);
        set_free(num | 1);  // Right child as free.
    }

    return ptr;
}

/* Try to deallocate a memory piece. */
unsafe fn try_dealloc(mut num : usize, mut rank : usize) {
    // Go to parent...
    while test_and_set(num, rank) { num >>= 1; rank += 1; }

    (*rklist(rank)).push(set_index(num, rank) as _);
}

/* Init the rank list as empty.  */
unsafe fn init_rklist() {
    for i in 0..MAX_RANK { (*rklist(i)).init(); }
}

/* Init the bit map with 0. */
unsafe fn init_bitmap() {
    for i in 0..MAP_SIZE { BITMAP.add(i).write(0); }
}

/* Functions of buddy allocator. */
impl BuddyAllocator {
    pub unsafe fn first_init(rank : usize) {
        init_rklist(); init_bitmap();

        (*rklist(rank)).push(BUDDY_START as _);
        set_free(get_index(BUDDY_START, rank));
    }

    pub unsafe fn allocate(size : usize) -> *mut u8 {
        return try_alloc(get_rank(size));
    }

    pub unsafe fn deallocate(ptr : *mut u8, size : usize) {
        let rank = get_rank(size);
        return try_dealloc(get_index(ptr, rank), rank);
    }

    pub unsafe fn debug() {
        warning!("Base address: {:p}", BUDDY_START);
        for i in 0..MAX_RANK {
            let list = rklist(i);
            message!("  Rank {}: ", i);
            (*list).debug(i, BUDDY_START);
        }
        warning!("End of buddy debug!");
        print_separator();
    }
}
