extern crate alloc;

use alloc::{boxed::Box, vec::Vec};

use crate::alloc::page::{A, D, G, R, V, W, X};

struct PageIterator {
    huge    : *mut PageTableEntry,
    next    : *mut PageTableEntry,
    leaf    : *mut PageTableEntry,
}

use super::{page::{PTEFlag, PageAddress, PageTableEntry, U}, PAGE_BITS};

impl core::ops::Index<usize> for PageAddress {
    type Output = PageTableEntry;
    fn index(&self, x : usize) -> &PageTableEntry {
        unsafe { self.get_entry(x) }
    }
}

impl core::ops::IndexMut<usize> for PageAddress {
    fn index_mut(&mut self, x : usize) -> &mut PageTableEntry {
        unsafe { self.get_entry(x) }
    }
}

impl PageAddress {
    /** Add a supervisor mapping. */
    pub unsafe fn smap(self, virt : usize, phys : PageAddress, flag : PTEFlag) {
        return vmmap(self, virt, phys, flag);
    }
    /** Add a user mapping. */
    pub unsafe fn umap(self, virt : usize, phys : PageAddress, flag : PTEFlag) {
        return vmmap(self, virt, phys, flag | U);
    }

    /**
     * Copy from a user pointer to a kernel pointer.
     * This will check the permission of the user pointer.
     * It should be at least U + R + V.
     */
    pub unsafe fn user_to_core(self, src : usize, len : usize) -> Box<[u8]> {
        if len == 0 { return Box::new([]); }
        let vec : Vec <u8> = Vec::with_capacity(len);
        let end = src + len;
        let page_beg = src >> 12;
        let page_end = (end - 1) >> 12;
        if page_beg == page_end {
            return user_to_core_0(self, src, vec);
        } else {
            todo!("Implement user_to_core for multiple pages.");
            // return copy_from_user_1(self, src, len, vec);
        }
    }

    /**
     * Copy from a kernel pointer to a user pointer.
     * This will check the permission of the user pointer.
     * It should be at least U + W + V.
     */
    pub unsafe fn core_to_user(self, dst : usize, src : &[u8]) {
        if src.len() == 0 { return; }
        let end = dst + src.len();
        let page_beg = dst >> 12;
        let page_end = (end - 1) >> 12;
        if page_beg == page_end {
            return core_to_user_0(self, dst, src);
        } else {
            todo!("Implement core_to_user for multiple pages.");
            // return copy_to_user_1(self, dst, len, src);
        }
    }

    /** Validate the input pointer. */
    pub unsafe fn validate_ptr(self, dst : usize, len : usize, flag : PTEFlag) {
        if len == 0 { return; }
        let end = dst + len;
        let page_beg = dst >> 12;
        let page_end = (end - 1) >> 12;
        return validate_pointer(self, dst, page_end - page_beg, flag);
    }

    /** Return the iterator at given address. */
    unsafe fn get_iterator(mut self, src : usize) -> PageIterator {
        let page = src >> 12;
        let ppn0 = (page >> 18) & 0x1FF;
        let ppn1 = (page >> 9 ) & 0x1FF;
        let ppn2 = (page >> 0 ) & 0x1FF;

        let huge = &mut self[ppn0];
        let (mut addr, flag) = huge.get_entry();
        assert!(flag == PTEFlag::NEXT, "Invalid page table mapping!");

        let next = &mut addr[ppn1];
        let (mut addr, flag) = next.get_entry();
        assert!(flag == PTEFlag::NEXT, "Invalid page table mapping!");

        let leaf = &mut addr[ppn2];
        return PageIterator { huge, next, leaf };
    }

    /** Debug method. */
    pub fn debug(self) {
        warning!("Root address = {:#x}", self.address() as usize);
        for i in 0..512 {
            let base = i << 18;
            let (addr, flag) = self[i].get_entry();
            if flag == PTEFlag::INVALID { continue; }
            if flag != PTEFlag::NEXT {
                message_inline!("Mapping 1GiB {:<12p} -> {:<10p} Flag = ",
                    to_virtual(base) , addr.address());
                flag.debug(); 
                continue;
            }
            for j in 0..512 {
                let base = base | j << 9;
                let (addr, flag) = addr[j].get_entry();
                if flag == PTEFlag::INVALID { continue; }
                if flag != PTEFlag::NEXT {
                    message_inline!("Mapping 2MiB {:<12p} -> {:<10p} Flag = ",
                        to_virtual(base), addr.address());
                    flag.debug();
                    continue;
                }
                warning!("Here {:p}", addr.address());
                for k in 0..512 {
                    let base = base | k;
                    let (addr, flag) = addr[k].get_entry();
                    if flag == PTEFlag::INVALID { continue; }
                    assert!(flag != PTEFlag::NEXT, "Invalid page table mapping!");
                    message_inline!("Mapping 4KiB {:<12p} -> {:<10p} Flag = ",
                        to_virtual(base), addr.address());
                    flag.debug();
                }
            }
        }
    }
}


/**
 * Build up a mapping from a virtual address to a physical address at given page table.
 */
unsafe fn vmmap(mut root : PageAddress, virt : usize, phys : PageAddress, __flag : PTEFlag) {
    let virt =  virt >> 12;
    let ppn0 = (virt >> 18) & 0x1FF;
    let ppn1 = (virt >> 9 ) & 0x1FF;
    let ppn2 = (virt >> 0 ) & 0x1FF;

    let page = &mut root[ppn0];
    let (addr, flag) = page.get_entry();
    if flag == PTEFlag::INVALID {
        let temp = PageAddress::new_pagetable();
        page.set_entry(temp, PTEFlag::NEXT);
        root = temp;
    } else {
        assert!(flag == PTEFlag::NEXT, "Mapping existed!");
        root = addr;
    }

    let page = &mut root[ppn1];
    let (addr, flag) = page.get_entry();
    if flag == PTEFlag::INVALID {
        let temp = PageAddress::new_pagetable();
        page.set_entry(temp, PTEFlag::NEXT);
        root = temp;
    } else {
        assert!(flag == PTEFlag::NEXT, "Mapping existed!");
        root = addr;
    }

    let page = &mut root[ppn2];
    let (_, flag) = page.get_entry();
    assert!(flag == PTEFlag::INVALID, "Mapping existed!");
    page.set_entry(phys, __flag);
}

/**
 * Copy from kernel to user in one page.
*/
unsafe fn core_to_user_0(
    root : PageAddress, beg : usize, src : &[u8]) {
    let offset  = block_offset(beg);
    let iter    = root.get_iterator(beg);

    let (addr, flag) = (*iter.leaf).get_entry();
    assert!(flag.contains(U | W | V), "Invalid page table mapping!");

    let addr = addr.address().wrapping_add(offset);
    return addr.copy_from(src.as_ptr(), src.len());
}

unsafe fn user_to_core_0(
    root : PageAddress, beg : usize, mut vec : Vec<u8>) -> Box<[u8]> {
    let offset  = block_offset(beg);
    let iter    = root.get_iterator(beg);

    let (addr, flag) = (*iter.leaf).get_entry();
    assert!(flag.contains(U | R | V), "Invalid page table mapping!");

    let addr = addr.address().wrapping_add(offset);
    let size = vec.capacity() - vec.len();
    vec.extend(core::slice::from_raw_parts(addr, size));

    return vec.into_boxed_slice();
}

/**
 * Validate the pointer in a range of pages.
 */
unsafe fn validate_pointer(
    addr : PageAddress, beg : usize, mut cnt : usize, test : PTEFlag) {
    let mut iter = addr.get_iterator(beg);
    loop {
        let (_, flag) = (*iter.leaf).get_entry();
        assert!(flag.contains(U | V | test), "Invalid page table mapping!");

        if cnt == 0 { break; }
        cnt -= 1; iter.inc(); // Increment the iterator and check the next page.
    }
}

impl PTEFlag {
    /** Debug output. */
    fn debug(self) {
        print_if(self.contains(D), 'D');
        print_if(self.contains(A), 'A');
        print_if(self.contains(G), 'G');
        print_if(self.contains(U), 'U');
        print_if(self.contains(X), 'X');
        print_if(self.contains(W), 'W');
        print_if(self.contains(R), 'R');
        uart_print!("\n");
    }
}

#[inline(always)]
fn print_if(cond : bool, mut x : char) { if !cond { x = '-'; } uart_print!("{}", x); }

#[inline(always)]
fn to_virtual(x : usize) -> *mut u8 { return (x << PAGE_BITS) as _; }

/** The in-block offset of a pointer. */
#[inline(always)]
fn block_offset(p : usize) -> usize { return p & 0xFFF; }

/** Increment a entry and return whether it reach an end. */
unsafe fn inc_test(addr : *mut *mut PageTableEntry) -> bool {
    *addr = (*addr).wrapping_add(1);
    let addr = *addr as usize;
    return block_offset(addr) == 0;
}

impl PageIterator {
    /** Get the next page. */
    pub unsafe fn inc(&mut self) {
        if inc_test(&mut self.leaf) {
            if inc_test(&mut self.next) {
                assert!(!inc_test(&mut self.huge), "God Damn it!");
                let (mut addr, _) = (*self.huge).get_entry();
                self.next = &mut addr[0];
            }
            let (mut addr, _) = (*self.next).get_entry();
            self.leaf = &mut addr[0];
        }
    }
}
