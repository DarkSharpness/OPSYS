use bitflags::bitflags;

use crate::alloc::print_separator;
use crate::alloc::get_mem_end;
use super::{buddy::BuddyAllocator, constant::*};

#[derive(Clone, Copy)]
pub struct PageAddress(usize);
#[derive(Clone, Copy)]
pub struct PageTableEntry(usize);
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PTEFlag(usize);

/**
 * These flags are made private to avoid any misuse.
 * ---------------------------------------------------
 * Plan of the 2 reserved bits:
 * - 00: Default, a PTE that doesn't own any permission.
 * - 01: A PTE owned by current process, destructed when process exits.
 * - 10: A PTE shared by multiple processes, need reference counting.
 * - 11: Not used yet.
 */
pub(crate) const V : PTEFlag = PTEFlag(1 << 0);
pub(crate) const R : PTEFlag = PTEFlag(1 << 1);
pub(crate) const W : PTEFlag = PTEFlag(1 << 2);
pub(crate) const X : PTEFlag = PTEFlag(1 << 3);
pub(crate) const U : PTEFlag = PTEFlag(1 << 4);
pub(crate) const G : PTEFlag = PTEFlag(1 << 5);
pub(crate) const A : PTEFlag = PTEFlag(1 << 6);
pub(crate) const D : PTEFlag = PTEFlag(1 << 7);
pub(crate) const RSV : PTEFlag = PTEFlag(3 << 8);

bitflags! {
    // Only make those useful flags public.
    impl PTEFlag : usize {
        const XO = V.0 | X.0;       // Execute-only
        const RO = V.0 | R.0;       // Read-only
        const WO = V.0 | W.0;       // Write-only
        const RX = V.0 | R.0 | X.0; // Read-execute
        const RW = V.0 | R.0 | W.0; // Read-write

        const NEXT      = 1;        // Next level of page table
        const INVALID   = 0;        // Invalid page table entry

        const OTHER     = 0b00 << 8; // Default, no permission.
        const OWNED     = 0b01 << 8; // Exclusive mapping
        const SHARED    = 0b10 << 8; // Shared mapping, need ref count.
    }
}

/**
 * Init the page table with 3 size of pages.
 */
pub unsafe fn init_page_table() {
    logging!("Initialize the page table.");
    extern "C" { fn get_pagetable() -> usize; }

    /* Page table should be located at 0x80002000.  */
    assert!(get_pagetable() == PAGE_TABLE_ADDR, "Page table at wrong address!");

    let mut root = KERNEL_SATP;

    // Reset as invalid.
    for i in 3..512 {
        set_huge_identity(root, i, PTEFlag::INVALID);
    }

    // Set MMIO as read/write only.
    set_huge_identity(root, 0, PTEFlag::RW);
    set_huge_identity(root, 1, PTEFlag::INVALID);

    // Set kernel part using middle/normal size page.
    // Set the second level page table.
    let mut page = PageAddress::new_pagetable();
    root[2].set_entry(page, PTEFlag::NEXT);

    // Set the kernel memory as read/write only.
    let mem_end = get_relative_page_num(get_mem_end()) >> 9;
    for i in 1..mem_end {
        set_medium_identity(page, 2, i, PTEFlag::RW);
    }
    // Set the rest as invalid, of course.
    for i in mem_end..512 {
        set_medium_identity(page, 2, i, PTEFlag::INVALID);
    }

    // Set the kernel code in details.
    let leaf = PageAddress::new_pagetable();
    page[0].set_entry(leaf, PTEFlag::NEXT);
    init_kernel_page(leaf);

    logging!("Page table initialized.");
    print_separator();
}

unsafe fn init_kernel_page(leaf : PageAddress) {
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss_real(); // This is because kernel stack is below sbss, but still in bss.
        fn ebss();
        fn ekernel();
    }

    let text_start  = get_relative_page_num(stext as usize);
    let text_finish = get_relative_page_num(etext as usize);
    message!("text_start: {}, text_finish: {}", text_start, text_finish);
    for i in text_start..text_finish {
        set_normal_identity(leaf, 2, 0, i, PTEFlag::RX);
    }

    let rodata_start  = get_relative_page_num(srodata as usize);
    let rodata_finish = get_relative_page_num(erodata as usize);
    message!("rodata_start: {}, rodata_finish: {}", rodata_start, rodata_finish);
    for i in rodata_start..rodata_finish {
        set_normal_identity(leaf, 2, 0, i, PTEFlag::RO);
    }

    let data_start  = get_relative_page_num(sdata as usize);
    let data_finish = get_relative_page_num(edata as usize);
    message!("data_start: {}, data_finish: {}", data_start, data_finish);
    for i in data_start..data_finish {
        set_normal_identity(leaf, 2, 0, i, PTEFlag::RW);
    }

    let bss_start  = get_relative_page_num(sbss_real as usize);
    let bss_finish = get_relative_page_num(ebss as usize);
    message!("bss_start: {}, bss_finish: {}", bss_start, bss_finish);
    for i in bss_start..bss_finish {
        set_normal_identity(leaf, 2, 0, i, PTEFlag::RW);
    }

    // The rest is reserved for buddy allocator.
    let finish = get_relative_page_num(ekernel as usize);
    message!("Kernel page finish at {}", finish);
    for i in finish..512 {
        set_normal_identity(leaf, 2, 0, i, PTEFlag::RW);
    }

    // Set the address of root page table as read/write-able
    // This is because our pagetable is placed at a special
    // position, within the text section (which will be marked as RX).
    // So, we need to change it to RW.
    set_normal_identity(leaf, 2, 0, 2, PTEFlag::RW);
} 

impl PageTableEntry {
    const MASK : usize = 0x3FF;
    pub fn set_entry(&mut self, addr : PageAddress, flag : PTEFlag) {
        self.0 = (addr.bits()) << 10 | flag.bits();
    }
    pub fn get_entry(self) -> (PageAddress, PTEFlag) {
        (PageAddress(self.0 >> 10), PTEFlag(self.0 & Self::MASK))
    }
    pub fn set_flag(&mut self, flag : PTEFlag) {
        self.0 = (self.0 & !Self::MASK) | flag.bits();
    }
}

impl PageAddress {
    /** Return a zero-filled page for page table. */
    pub fn new_pagetable() -> Self { unsafe { allocate_zero() } }
    /** Return a zero-filled page  */
    pub fn new_zero_page() -> Self { unsafe { allocate_zero() } }
    /** Return an uninitialized page with random bits. */
    pub fn new_rand_page() -> Self { unsafe { allocate_page() } }
    /** Return a page with given physical address entry. */
    pub const fn new_usize(num : usize) -> Self { PageAddress(num >> 12) }

    fn new_ptr(num : *mut u8) -> Self {
        PageAddress((num as usize) >> 12)
    }
    fn new_huge(ppn0 : usize) -> Self {
        PageAddress(ppn0 << 18)
    }
    fn new_medium(ppn0 : usize, ppn1 : usize) -> Self {
        PageAddress(ppn0 << 18 | ppn1 << 9)
    }
    fn new_normal(ppn0 : usize, ppn1 : usize, ppn2 : usize) -> Self {
        PageAddress(ppn0 << 18 | ppn1 << 9 | ppn2)
    }
    pub(crate) unsafe fn get_entry(&self, x : usize) -> &mut PageTableEntry {
        &mut *((self.0 << 12) as *mut PageTableEntry).wrapping_add(x)
    }

    /** Return the index of a physical page. */
    pub fn bits(self) -> usize { self.0 }
    /** Return the physical address. */
    pub fn address(self) -> *mut u8 { (self.0 << 12) as *mut u8 }
}

/** Return the relative page number to the front of kernel (0x80000000).  */
unsafe fn get_relative_page_num(x : usize) -> usize {
    extern "C" { fn skernel(); }
    return (x - skernel as usize) / PAGE_SIZE;
}

/* Some identity mapping based-on the level size of the page (huge/medium/normal). */

unsafe fn set_huge_identity(mut page : PageAddress, i : usize, flag : PTEFlag) {
    page[i].set_entry(PageAddress::new_huge(i as _), flag);
}

unsafe fn set_medium_identity(mut page : PageAddress, i : usize, j : usize, flag : PTEFlag) {
    page[j].set_entry(PageAddress::new_medium(i as _, j as _), flag);
}

unsafe fn set_normal_identity(mut page : PageAddress, i : usize, j : usize, k : usize, flag : PTEFlag) {
    page[k].set_entry(PageAddress::new_normal(i as _, j as _, k as _), flag);
}

unsafe fn allocate_zero() -> PageAddress {
    let addr = BuddyAllocator::allocate_page();

    warning!("Zero-filled page allocated at {:p}", addr);

    /* Reset the page to zero. */
    let temp = addr as *mut usize;
    for i in 0..512 { *temp.wrapping_add(i) = 0; }

    return PageAddress::new_ptr(addr);
}

unsafe fn allocate_page() -> PageAddress {
    let addr = BuddyAllocator::allocate_page();
    warning!("Uninitialized page allocated at {:p}", addr);
    return PageAddress::new_ptr(addr);
}
