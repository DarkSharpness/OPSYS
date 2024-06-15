use super::{page::{PTEOwner, PageTableEntry, U}, PTEFlag, PageAddress};

impl PageAddress {
    /** Add a supervisor mapping. */
    pub unsafe fn smap(self, virt : usize, phys : PageAddress, flag : PTEFlag) {
        return mmap(self, virt, phys, flag | PTEOwner::Kernel.to_flag());
    }
    /** Try to add a supervisor mapping. If existed, throw. */
    pub unsafe fn new_smap(self, virt : usize, flag : PTEFlag) -> PageAddress {
        return new_mmap(self, virt, flag | PTEOwner::Kernel.to_flag());
    }
    /** Add a user-defined mapping. */
    pub unsafe fn umap(self, virt : usize, phys : PageAddress, flag : PTEFlag) {
        return mmap(self, virt, phys, flag | PTEOwner::Process.to_flag() | U);
    }
    /** Try to add a mapping. If existed, throw. */
    pub unsafe fn new_umap(self, virt : usize, flag : PTEFlag) -> PageAddress {
        return new_mmap(self, virt, flag | PTEOwner::Process.to_flag() | U);
    }
    /** Try to add a mapping. If existed, just add to the flags. */
    pub unsafe fn try_umap(self, virt : usize, flag : PTEFlag) -> PageAddress {
        return try_mmap(self, virt, flag | PTEOwner::Process.to_flag() | U);
    }
    pub unsafe fn try_unumap(self, virt : usize) -> bool {
        return try_unumap(self, virt);
    }
}

#[inline(never)]
unsafe fn mmap_until_leaf(
    mut root : PageAddress, virt : usize, __flag : PTEFlag) -> *mut PageTableEntry {
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

    return &mut root[ppn2];
}

unsafe fn mmap(root : PageAddress, virt : usize, phys : PageAddress, __flag : PTEFlag) {
    let page = &mut *mmap_until_leaf(root, virt, __flag);
    let (_, flag) = page.get_entry();
    assert!(flag == PTEFlag::INVALID, "Mapping existed!");
    page.set_entry(phys, __flag);
}

unsafe fn new_mmap(root : PageAddress, virt : usize, __flag : PTEFlag) -> PageAddress {
    let page = &mut *mmap_until_leaf(root, virt, __flag);
    let (_, flag) = page.get_entry();
    assert!(flag == PTEFlag::INVALID, "Mapping existed!");
    let new = PageAddress::new_pagetable();
    page.set_entry(new, __flag);
    return new;
}

unsafe fn try_mmap(root : PageAddress, virt : usize, __flag : PTEFlag) -> PageAddress {
    let page = &mut *mmap_until_leaf(root, virt, __flag);
    let (old, flag) = page.get_entry();
    if flag == PTEFlag::INVALID {
        let phys = PageAddress::new_pagetable();
        page.set_entry(phys, __flag);
        return phys;
    } else {
        assert!(flag != PTEFlag::NEXT, "Invalid mapping!");
        page.add_flag(__flag);
        return old;
    }
}

unsafe fn try_unumap(mut root : PageAddress, virt : usize) -> bool {
    let virt = virt >> 12;
    let ppn0 = (virt >> 18) & 0x1FF;
    let ppn1 = (virt >> 9 ) & 0x1FF;
    let ppn2 = (virt >> 0 ) & 0x1FF;

    let page = &mut root[ppn0];
    let (addr, flag) = page.get_entry();
    if flag != PTEFlag::NEXT {
        return false;
    } 

    root = addr;
    let page = &mut root[ppn1];
    let (addr, flag) = page.get_entry();
    if flag != PTEFlag::NEXT {
        return false;
    }

    root = addr;
    let page = &mut root[ppn2];
    let (addr, flag) = page.get_entry();
    if flag == PTEFlag::INVALID || flag == PTEFlag::NEXT {
        return false;
    }

    addr.free_this();
    page.reset();

    return true;
}
