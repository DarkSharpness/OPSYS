use super::{constant::*, frame::FrameAllocator};

#[derive(Clone, Copy)]
pub struct PageAddress(pub u64);
#[derive(Clone, Copy)]
pub struct PageTableEntry(u64);
#[derive(Clone, Copy)]
pub struct PTEFlag(u64);

unsafe fn set_huge(mut page : PageAddress, i : usize, flag : PTEFlag) {
    page[i].set_entry(PageAddress::new_huge(i as _), flag);
}

unsafe fn set_medium(mut page : PageAddress, i : usize, j : usize, flag : PTEFlag) {
    page[j].set_entry(PageAddress::new_medium(i as _, j as _), flag);
}

// Initialize the huge page table.
#[allow(const_item_mutation)]
pub unsafe fn init_huge_page() {
    // Reset as invalid.
    for i in 3..256 { set_huge(PAGE_TABLE, i, PageTableEntry::INVALID); }

    // Set MMIO as read/write only.
    set_huge(PAGE_TABLE, 0, PageTableEntry::RW);
    set_huge(PAGE_TABLE, 1, PageTableEntry::RW);

    // Kernel part below.
    // Set kernel part using middle size page.

    let page = PageAddress::new_ptr(FrameAllocator::allocate_page());
    PAGE_TABLE[2].set_entry(page, PageTableEntry::NXT);

    // Set the kernel code in details.
    set_medium(page, 2, 0, PageTableEntry::RWX);
    // Set the kernel memory as read/write only.
    for i in 1..256 { set_medium(page, 2, i, PageTableEntry::RW); }
}

/* Implementation part below. */

impl PTEFlag {
    const VALID    : u64   = 1 << 0;
    const READ     : u64   = 1 << 1;
    const WRITE    : u64   = 1 << 2;
    const EXECUTE  : u64   = 1 << 3;
    const USER     : u64   = 1 << 4;
    const GLOBAL   : u64   = 1 << 5;
    const ACCESSED : u64   = 1 << 6;
    const DIRTY    : u64   = 1 << 7;
    const HUGE     : u64   = 1 << 8;
    const RESERVED : u64   = 1 << 9;
    pub fn bits(&self) -> u64 { self.0 }
}

impl PageTableEntry {
    // Invalid page table entry.
    const INVALID   : PTEFlag = PTEFlag(0);

    // Normal page table entry setting.
    const EX    : PTEFlag = PTEFlag(PTEFlag::EXECUTE | PTEFlag::VALID);
    const RW    : PTEFlag = PTEFlag(PTEFlag::READ | PTEFlag::WRITE | PTEFlag::VALID);
    const RX    : PTEFlag = PTEFlag(PTEFlag::READ | PTEFlag::EXECUTE | PTEFlag::VALID);
    const RWX   : PTEFlag = PTEFlag(PTEFlag::READ | PTEFlag::WRITE | PTEFlag::EXECUTE | PTEFlag::VALID);
    const NXT   : PTEFlag = PTEFlag(PTEFlag::VALID);
    
    pub fn set_entry(&mut self, addr : PageAddress, flag : PTEFlag) {
        self.0 = (addr.bits()) << 10 | flag.bits();
    }
}

impl PageAddress {
    pub const fn new_u64(num : u64) -> Self {
        PageAddress(num >> 12)
    }
    pub fn new_ptr(num : *mut u8) -> Self {
        PageAddress((num as u64) >> 12)
    }
    fn new_huge(ppn0 : u64) -> Self {
        PageAddress(ppn0 << 18)
    }
    fn new_medium(ppn0 : u64, ppn1 : u64) -> Self {
        PageAddress(ppn0 << 18 | ppn1 << 9)
    }
    fn new_normal(ppn0 : u64, ppn1 : u64, ppn2 : u64) -> Self {
        PageAddress(ppn0 << 18 | ppn1 << 9 | ppn2)
    }
    unsafe fn get_entry(&self, x : usize) -> &mut PageTableEntry {
        &mut *((self.0 << 12) as *mut PageTableEntry).wrapping_add(x)
    }
    /** Return Inner bits. */
    pub fn bits(&self) -> u64 { self.0 }
}

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
