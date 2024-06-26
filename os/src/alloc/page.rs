use bitflags::bitflags;

use crate::alloc::print_separator;
use crate::alloc::get_mem_end;
use crate::get_zero_page;
use super::{buddy::BuddyAllocator, constant::*};

#[derive(Clone, Copy)]
pub struct PageAddress(usize);
#[derive(Clone, Copy)]
pub struct PageTableEntry(usize);
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PTEFlag(usize);

pub(super) enum PTEOwner {
    Kernel   = 0,   // Kernel owned, no need to destruct.
    Process  = 1,   // Process owned, destructed when process exits.
    Shared   = 2,   // Shared by multiple processes, need reference counting.
    Reserved = 3,   // Reserved for future use.
}

/**
 * These flags are made private to avoid any misuse.
 * ---------------------------------------------------
 * Plan of the 2 reserved bits:
 * - 00: Default, a PTE that doesn't own any permission.
 * - 01: A PTE owned by current process, destructed when process exits.
 * - 10: A PTE shared by multiple processes, need reference counting.
 * - 11: Not used yet.
 */
pub(super) const V : PTEFlag = PTEFlag(1 << 0);
pub(super) const R : PTEFlag = PTEFlag(1 << 1);
pub(super) const W : PTEFlag = PTEFlag(1 << 2);
pub(super) const X : PTEFlag = PTEFlag(1 << 3);
pub(super) const U : PTEFlag = PTEFlag(1 << 4);
pub(super) const G : PTEFlag = PTEFlag(1 << 5);
pub(super) const A : PTEFlag = PTEFlag(1 << 6);
pub(super) const D : PTEFlag = PTEFlag(1 << 7);
pub(super) const RSV : PTEFlag = PTEFlag(3 << 8);

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
        const EMPTY     = 0;        // Empty page table entry
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
    for i in 2..mem_end {
        set_medium_identity(page, 2, i, PTEFlag::RW);
    }
    // Set the rest as invalid, of course.
    for i in mem_end..512 {
        set_medium_identity(page, 2, i, PTEFlag::INVALID);
    }

    // Set the kernel code in details.

    let size = get_kernel_size();
    const MIDDLE_PAGE_SIZE : usize = PAGE_SIZE << 9;
    let mids = (size + MIDDLE_PAGE_SIZE - 1) / MIDDLE_PAGE_SIZE;
    message!("Middle pages {}", mids);
    for i in 0..mids {
        page[i].set_entry(PageAddress::new_pagetable(), PTEFlag::NEXT);
    }

    init_kernel_page(page, mids);

    logging!("Page table initialized.");
    print_separator();
}

unsafe fn get_kernel_size() -> usize {
    extern "C" {
        fn stext();
        fn ekernel();
    }
    let size = ekernel as usize - stext as usize;
    message!("Kernel size: {}", size);
    size
}

unsafe fn init_kernel_page(leaf : PageAddress, middle_count : usize) {
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
        set_special_identity(leaf, 2, 0, i, PTEFlag::RX);
    }

    let rodata_start  = get_relative_page_num(srodata as usize);
    let rodata_finish = get_relative_page_num(erodata as usize);
    message!("rodata_start: {}, rodata_finish: {}", rodata_start, rodata_finish);
    for i in rodata_start..rodata_finish {
        set_special_identity(leaf, 2, 0, i, PTEFlag::RO);
    }

    let data_start  = get_relative_page_num(sdata as usize);
    let data_finish = get_relative_page_num(edata as usize);
    message!("data_start: {}, data_finish: {}", data_start, data_finish);
    for i in data_start..data_finish {
        set_special_identity(leaf, 2, 0, i, PTEFlag::RW);
    }

    let bss_start  = get_relative_page_num(sbss_real as usize);
    let bss_finish = get_relative_page_num(ebss as usize);
    message!("bss_start: {}, bss_finish: {}", bss_start, bss_finish);
    for i in bss_start..bss_finish {
        set_special_identity(leaf, 2, 0, i, PTEFlag::RW);
    }

    // The rest is reserved for buddy allocator.
    let finish = get_relative_page_num(ekernel as usize);
    message!("Kernel page finish at {}", finish);
    for i in finish..(middle_count << 9) {
        set_special_identity(leaf, 2, 0, i, PTEFlag::RW);
    }

    // Set the address of zero page as read-only
    // Before we drop to the supervisor mode, we need to
    // fill the zero page with zero.
    // After that, we can set it as read-only.
    init_zero_page();
    set_special_identity(leaf, 2, 0, 1, PTEFlag::RO);

    // Set the address of root page table as read/write-able
    // This is because our pagetable is placed at a special
    // position, within the text section (which will be marked as RX).
    // So, we need to change it to RW.
    set_special_identity(leaf, 2, 0, 2, PTEFlag::RW);

    extern "C" { fn boot_stack_low(); }
    let stack = get_relative_page_num(boot_stack_low as usize);
    message!("Boot stack at {}", stack);

    set_special_identity(leaf, 2, 0, stack, PTEFlag::INVALID);
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
    pub fn add_flag(&mut self, flag : PTEFlag) {
        self.0 |= flag.bits();
    }
    pub fn reset(&mut self) {
        self.0 = 0;
    }
}

impl PTEFlag {
    pub(super) const fn get_owner(&self) -> PTEOwner {
        match self.bits() >> 8 {
            0b00 => PTEOwner::Kernel,
            0b01 => PTEOwner::Process,
            0b10 => PTEOwner::Shared,
            0b11 => PTEOwner::Reserved,
            _    => unreachable!(),
        }
    }
}

impl PTEOwner {
    #[inline(always)]
    pub(super) const fn to_flag(&self) -> PTEFlag {
        match self {
            PTEOwner::Kernel   => PTEFlag(0b00 << 8),
            PTEOwner::Process  => PTEFlag(0b01 << 8),
            PTEOwner::Shared   => PTEFlag(0b10 << 8),
            PTEOwner::Reserved => panic!("Reserved is not allowed."),
        }
    }
}

impl PageAddress {
    /** Return a zero-filled page for page table. */
    pub fn new_pagetable() -> Self { unsafe { allocate_zero() } }
    /** Return an uninitialized page with random bits. */
    pub fn new_rand_page() -> Self { unsafe { allocate_page() } }
    /** Return a page with given physical address entry. */
    pub const fn new_usize(num : usize) -> Self { PageAddress(num >> 12) }
    /** Free only this page. */
    pub unsafe fn free_this(self) {
        BuddyAllocator::deallocate_page(self.address());
    }
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
    pub(super) unsafe fn get_entry(&self, x : usize) -> &mut PageTableEntry {
        &mut *((self.0 << 12) as *mut PageTableEntry).wrapping_add(x)
    }
    /** Return the index of a physical page. */
    pub fn bits(self) -> usize { self.0 }
    /** Return the physical address. */
    pub fn address(self) -> *mut u8 { (self.0 << 12) as *mut u8 }
    /** Copy at given offset from some slice */
    pub fn copy_at(self, offset : usize, slice : &[u8]) {
        assert!(offset + slice.len() <= PAGE_SIZE, "Copy out of bound.");
        let dst = self.address().wrapping_add(offset);
        let src = slice.as_ptr();
        let len = slice.len();
        unsafe { dst.copy_from_nonoverlapping(src, len); }
    }
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

unsafe fn set_special_identity(page : PageAddress, i : usize, j : usize, k : usize, flag : PTEFlag) {
    let (mut page, leaf) = page[k >> 9].get_entry();
    assert!(leaf == PTEFlag::NEXT);
    // let addr = PageAddress::new_normal(i as _, j as _, k as _);
    // warning!("Special identity: {:#?}", addr.address());
    page[k & 0x1FF].set_entry(PageAddress::new_normal(i as _, j as _, k as _), flag);
}

unsafe fn allocate_zero() -> PageAddress {
    let addr = BuddyAllocator::allocate_page();

    // warning!("Zero-filled page allocated at {:p}", addr);

    /* Reset the page to zero. */
    let temp = addr as *mut usize;
    for i in 0..512 { *temp.wrapping_add(i) = 0; }

    return PageAddress::new_ptr(addr);
}

unsafe fn allocate_page() -> PageAddress {
    let addr = BuddyAllocator::allocate_page();
    // warning!("Uninitialized page allocated at {:p}", addr);
    return PageAddress::new_ptr(addr);
}

extern "C" { fn end_entry(); }

unsafe fn init_zero_page() {
    let entry = end_entry as usize;
    let start = get_zero_page().as_ptr() as *mut u8;
    message!("End of entry: {:x}", entry);
    message!("Zero page start: {:x}", start as usize);
    assert!(entry < start as usize, "Invalid end address");
    start.write_bytes(0, PAGE_SIZE);
}
