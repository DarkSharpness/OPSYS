use crate::alloc::page::{A, D, G, R, V, W, X};

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

    /** Copy from a kernel pointer to a user pointer */
    pub unsafe fn core_to_user(self, dst : *mut u8 , src : *const u8, len : usize) {
        if len == 0 { return; }
        let end = dst.wrapping_add(len);
        let page_beg = (dst as usize) >> 12;
        let page_end = (end as usize - 1) >> 12;
        if page_beg == page_end {
            return core_to_user_0(self, dst, len, src);
        } else {
            todo!("Implement copy_to_user for multiple pages.");
            // return self.copy_to_user_1(dst, len, src);
        }
    }

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
    let virt = (virt >> 12) as usize;
    let ppn0 = (virt >> 18) & 0x1FF;
    let ppn1 = (virt >> 9)  & 0x1FF;
    let ppn2 = (virt >> 0)  & 0x1FF;

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
unsafe fn core_to_user_0(addr : PageAddress, beg : *mut u8, len : usize, src : *const u8) {
    let page    = beg as usize >> 12;
    let offset  = beg as usize & 0xFFF; 
    let ppn0 = (page as usize) >> 18;
    let ppn1 = (page as usize) >> 9 & 0x1FF;
    let ppn2 = (page as usize) & 0x1FF;

    let (addr, flag) = addr[ppn0].get_entry();        
    assert!(flag == PTEFlag::NEXT, "Invalid page table mapping!");
    let (addr, flag) = addr[ppn1].get_entry();
    assert!(flag == PTEFlag::NEXT, "Invalid page table mapping!");
    let (addr, flag) = addr[ppn2].get_entry();
    assert!(flag.contains(U | W | V), "Invalid page table mapping!");

    let addr = addr.address().wrapping_add(offset);
    addr.copy_from(src, len);
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
