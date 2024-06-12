use crate::alloc::page::{R, U, V, W};
use crate::alloc::{page::PTEOwner, PTEFlag, PAGE_SIZE};
use crate::utility::*;

use super::{page::PageTableEntry, PageAddress};

struct PageIterator {
    huge    : *mut PageTableEntry,
    next    : *mut PageTableEntry,
    leaf    : *mut PageTableEntry,
}

impl PageAddress {
        /**
     * Copy from a kernel pointer to a user pointer.
     * This will check the permission of the user pointer.
     * It should be at least U + W + V.
     */
    pub unsafe fn core_to_user <T : CanCopy> (self, dst : usize, len : usize, src : T) {
        if len == 0 { return; }
        return copy_to_user_impl(self, dst, len, src);
    }

    /**
     * Copy from a user pointer to a kernel pointer.
     * This will check the permission of the user pointer.
     * It should be at least U + R + V.
     */
    pub unsafe fn user_to_core <T : CanPush> (self, dst : T, src : usize, len : usize) {
        if len == 0 { return; }
        return user_to_core_impl(self, dst, src, len);
    }

    /** Validate the input pointer. */
    pub unsafe fn check_ptr(self, dst : usize, len : usize, flag : PTEFlag) -> bool {
        if len == 0 { return true; }
        let end = dst + len;
        let page_beg = dst >> 12;
        let page_end = (end - 1) >> 12;
        return validate_pointer(self, dst, page_end - page_beg, flag);
    }

    pub unsafe fn copy_from(self, root : PageAddress) {
        return copy_impl(self, root);
    }

    /** Return the iterator at given address. */
    unsafe fn get_iterator(mut self, src : usize) -> Option<PageIterator> {
        let page = src >> 12;
        let ppn0 = (page >> 18) & 0x1FF;
        let ppn1 = (page >> 9 ) & 0x1FF;
        let ppn2 = (page >> 0 ) & 0x1FF;

        let huge = &mut self[ppn0];
        let (mut addr, flag) = huge.get_entry();
        if flag != PTEFlag::NEXT { return None; }

        let next = &mut addr[ppn1];
        let (mut addr, flag) = next.get_entry();
        if flag != PTEFlag::NEXT { return None; }

        let leaf = &mut addr[ppn2];
        return Some(PageIterator{ huge, next, leaf });
    }
}

impl PageIterator {
    /** Get the next page. */
    unsafe fn inc(&mut self) {
        if inc_test(&mut self.leaf) {
            if inc_test(&mut self.next) {
                inc_test(&mut self.huge);
                let (mut addr, _) = (*self.huge).get_entry();
                self.next = &mut addr[0];
            }
            let (mut addr, _) = (*self.next).get_entry();
            self.leaf = &mut addr[0];
        }
    }

    unsafe fn inc_check(&mut self) -> bool {
        if inc_test(&mut self.leaf) {
            if inc_test(&mut self.next) {
                if inc_test(&mut self.huge) { return false; }

                let (mut addr, flag) = (*self.huge).get_entry();
                if flag != PTEFlag::NEXT { return false; }
                self.next = &mut addr[0];
            }

            let (mut addr, flag) = (*self.next).get_entry();
            if flag != PTEFlag::NEXT { return false; }
            self.leaf = &mut addr[0];
        }
        return true;
    }

    unsafe fn get_address_flag(&self) -> (*mut u8, PTEFlag) {
        let leaf = &mut *self.leaf;
        let (addr, flag) = leaf.get_entry();
        return (addr.address(), flag);
    }
}

/** The in-block offset of a pointer. */
#[inline(always)]
fn block_offset(p : usize) -> usize { return p & 0xFFF; }

/** Increment a entry and return whether it reach an end. */
unsafe fn inc_test(addr : *mut *mut PageTableEntry) -> bool {
    *addr = (*addr).wrapping_add(1);
    let addr = *addr as usize;
    return block_offset(addr) == 0;
}

unsafe fn copy_to_user_impl <T : CanCopy> (
    root : PageAddress, dst : usize, mut len : usize, src : T) {
    let offset      = block_offset(dst);
    let mut iter    = root.get_iterator(dst).unwrap();
    let (addr,flag) = iter.get_address_flag();
    let addr        = addr.wrapping_add(offset);
    let mut src     = src;

    assert!(flag.contains(U | W | V), "Invalid page table mapping!");

    let remain = PAGE_SIZE - offset; // Remaining bytes in the first page.
    if len <= remain {
        return src.copy_n(core::slice::from_raw_parts_mut(addr, len));
    } else {
        len -= remain;
        src.copy_n(core::slice::from_raw_parts_mut(addr, remain));
        while len >= PAGE_SIZE {
            iter.inc();
            len -= PAGE_SIZE;
            let (addr, flag) = iter.get_address_flag();
            assert!(flag.contains(U | W | V), "Invalid page table mapping!");
            src.copy_n(core::slice::from_raw_parts_mut(addr, PAGE_SIZE));
        }
        src.copy_n(core::slice::from_raw_parts_mut(addr, len));
    }
}

unsafe fn user_to_core_impl <T : CanPush> (
    root : PageAddress, dst : T, src : usize, mut len : usize) {
    let offset      = block_offset(src);
    let mut iter    = root.get_iterator(src).unwrap();
    let (addr,flag) = iter.get_address_flag();
    let addr        = addr.wrapping_add(offset);
    let mut dst     = dst;
    assert!(flag.contains(U | R | V), "Invalid page table mapping!");

    let remain = PAGE_SIZE - offset; // Remaining bytes in the first page.
    if len <= remain {
        return dst.push_n(core::slice::from_raw_parts(addr, len));
    } else {
        len -= remain;
        dst.push_n(core::slice::from_raw_parts(addr, remain));
        while len >= PAGE_SIZE {
            iter.inc();
            len -= PAGE_SIZE;
            let (addr, flag) = iter.get_address_flag();
            assert!(flag.contains(U | R | V), "Invalid page table mapping!");
            dst.push_n(core::slice::from_raw_parts(addr, PAGE_SIZE));
        }
        dst.push_n(core::slice::from_raw_parts(addr, len));
    }
}

/**
 * Validate the pointer in a range of pages.
 */
unsafe fn validate_pointer(
    addr : PageAddress, beg : usize, mut cnt : usize, test : PTEFlag) -> bool {
    let mut iter = match addr.get_iterator(beg) {
        Some(x) => x,
        None => return false,
    };
    loop {
        let (_, flag) = iter.get_address_flag();
        if !flag.contains(U | V | test) { return false; }

        if cnt == 0 { return true; }
        cnt -= 1;
        if !iter.inc_check() { return false; }
    }
}

unsafe fn copy_impl(dst : PageAddress, src : PageAddress) {
    for i in 0..512 {
        let (addr, flag) = src[i].get_entry();
        if flag == PTEFlag::INVALID { continue; }
        assert!(flag == PTEFlag::NEXT, "Invalid page table mapping!");
        for j in 0..512 {
            let (addr, flag) = addr[j].get_entry();
            if flag == PTEFlag::INVALID { continue; }
            assert!(flag == PTEFlag::NEXT, "Invalid page table mapping!");
            for k in 0..512 {
                let (addr, flag) = addr[k].get_entry();
                if flag == PTEFlag::INVALID { continue; }
                match flag.get_owner() {
                    PTEOwner::Kernel => {
                        /* Kernel should have done it previously  */
                    },
                    PTEOwner::Process => {
                        use core::slice::from_raw_parts;
                        let page = dst.try_umap((i << 30) | (j << 21) | (k << 12), flag);
                        page.copy_at(0, from_raw_parts(addr.address(), PAGE_SIZE));
                    },
                    _ => todo!("Implement support for other owners."),
                }
            }
        }
    }
}
