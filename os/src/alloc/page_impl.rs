extern crate alloc;
use xmas_elf::{program::ProgramHeader, ElfFile};
use crate::{alloc::page::{A, D, G, R, V, W, X}, utility::*};

struct PageIterator {
    huge    : *mut PageTableEntry,
    next    : *mut PageTableEntry,
    leaf    : *mut PageTableEntry,
}

use super::{page::{PTEFlag, PageAddress, PageTableEntry, U}, PAGE_BITS, PAGE_SIZE};

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
    #[inline(always)]
    pub unsafe fn smap(self, virt : usize, phys : PageAddress, flag : PTEFlag, owner : PTEFlag) {
        return vmmap(self, virt, phys, flag | owner);
    }
    /** Add a user mapping. */
    #[inline(always)]
    pub unsafe fn umap(self, virt : usize, phys : PageAddress, flag : PTEFlag, owner : PTEFlag) {
        return vmmap(self, virt, phys, flag | owner | U);
    }

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

    /** Load a program header from an ELF file. */
    pub unsafe fn load_from_elf(&mut self, ph : ProgramHeader, elf : &ElfFile) {
        let start_va    = ph.virtual_addr() as usize;
        let end_va      = (ph.virtual_addr() + ph.mem_size()) as usize;
        if start_va == end_va { return; }

        let mut permission = PTEFlag::OWNED;
        let ph_flags = ph.flags();
        if ph_flags.is_read() { permission |= PTEFlag::RO; }
        if ph_flags.is_write() { permission |= PTEFlag::WO; }
        if ph_flags.is_execute() { permission |= PTEFlag::RX; }

        let start_page  = start_va      / PAGE_SIZE;
        let end_page    = (end_va - 1)  / PAGE_SIZE;
        let offset      = ph.offset() as usize;
        let data = &elf.input[offset..offset + ph.file_size() as usize];
        if start_page == end_page {
            let page = PageAddress::new_rand_page();
            self.umap(start_va, page, permission, PTEFlag::OWNED);
            let address = page.address().wrapping_add(start_va % PAGE_SIZE);
            for i in 0..data.len() {
                address.wrapping_add(i).write(data[i]);
            }
        } else {
            todo!("Implement load_from_elf for multiple pages.");
        }
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
                if flag.contains(PTEFlag::OTHER) {
                    /* Do nothing */
                } else if flag.contains(PTEFlag::OWNED) {
                    let temp = PageAddress::new_rand_page();
                    // The ownership has been contained in the flag.
                    dst.umap((i << 30) | (j << 21) | (k << 12), temp, flag, PTEFlag::EMPTY);
                    temp.address().copy_from(addr.address(), PAGE_SIZE);
                } else {
                    // If shared, add up the reference count.
                    todo!("Implement duplicate_impl for other flags.");
                }
            }
        }
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
