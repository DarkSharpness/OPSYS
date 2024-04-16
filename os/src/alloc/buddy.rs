use crate::{alloc::node::*, console::print_separator};
use super::constant::*;

pub struct BuddyAllocator;

#[inline(always)]
const fn get_rank(mut size : usize) -> usize {
    let mut rank = 0;
    while PAGE_SIZE < size {
        size >>= 1;
        rank += 1;
    }
    return rank;
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
        assert!(rank < TOP_RANK, "Out of memory!");
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

/* Set a bit as free. */
unsafe fn set_free(num : usize) {
    let (ptr, bit) = div_mod(num);
    ptr.write(ptr.read() | (1 << bit));
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

/** A segement-tree style build-up for buddy allocator. */
unsafe fn build(index : usize, l : usize, r : usize, beg : usize , end : usize, rank : usize) {
    /* Exactly the node. */
    if l == beg && r == end {
        set_free(index);
        let index = set_index(index, rank);
        return rklist(rank).push(index as _);
    } else {
        let mid = (l + r) >> 1;
        if end <= mid {
            // [begin, end) \in [l, mid)
            return build(index << 1 | 0, l, mid, beg, end, rank - 1);
        } else if mid <= beg {
            // [begin, end) \in [mid, r)
            return build(index << 1 | 1, mid, r, beg, end, rank - 1);
        } else {
            // Divide into [begin, mid) and [mid, end)
            build(index << 1 | 0, l, mid, beg, mid, rank - 1);
            build(index << 1 | 1, mid, r, mid, end, rank - 1);
        }
    }
}

/* Functions of buddy allocator. */
impl BuddyAllocator {
    const PAGE_RANK : usize = get_rank(PAGE_SIZE);
    const MAX_INDEX : usize = MAX_SIZE / PAGE_SIZE;

    /** Call once on init. */
    pub unsafe fn first_init(begin : usize, end : usize) {
        init_rklist();
        let begin = (begin - BASE_ADDRESS) / PAGE_SIZE;
        let end   = (end   - BASE_ADDRESS) / PAGE_SIZE;
        build(1, 0, BuddyAllocator::MAX_INDEX, begin, end, TOP_RANK - 1);
        BuddyAllocator::debug();
    }

    /** Allocate an arbitary size of memory (aligned to page). */
    pub unsafe fn allocate(size : usize) -> *mut u8 {
        return try_alloc(get_rank(size));
    }
    /** Deallocate an arbitary size of memory (aligned to page). */
    pub unsafe fn deallocate(ptr : *mut u8, size : usize) {
        let rank = get_rank(size);
        return try_dealloc(get_index(ptr, rank), rank);
    }

    /** Allocate exactly one page. */
    pub unsafe fn allocate_page() -> *mut u8 {
        return try_alloc(BuddyAllocator::PAGE_RANK);
    }
    /** Deallocate exactly one page.  */
    pub unsafe fn deallocate_page(ptr : *mut u8) {
        return try_dealloc(get_index(ptr, BuddyAllocator::PAGE_RANK), BuddyAllocator::PAGE_RANK);
    }

    /** An inner debug interface. */
    pub unsafe fn debug() {
        warning!("Base address: {:p}", BUDDY_START);
        warning!("Bitmap address: {:p}", BITMAP.as_ptr());
        warning!("Rank list address: {:p}", RKLIST.as_ptr());
        for i in 0..TOP_RANK {
            message!("  Rank {}: ", i);
            rklist(i).debug(i, BUDDY_START);
        }
        warning!("End of buddy debug!");
        print_separator();
    }
}

#[inline(always)]
unsafe fn rklist(idx : usize) -> &'static mut List { return RKLIST.get_unchecked_mut(idx); }
#[inline(always)]
unsafe fn mask(rank : usize) -> usize { return 1 << (TOP_RANK - 1 - rank); }
/** Divide and mod operation to get word and offset. */
#[inline(always)]
unsafe fn div_mod(num : usize) -> (* mut u8, usize) {
    return (BITMAP.get_unchecked_mut(num / WORD_BITS), num % WORD_BITS);
}
/* Init the rank list as empty.  */
#[inline(always)]
unsafe fn init_rklist() { for i in 0..TOP_RANK { rklist(i).init(); } }
