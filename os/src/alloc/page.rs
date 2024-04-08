/**
 * Page table settings.
 * 
 *  We use huge page, middle page and normal page to manage the memory.
 * 
 * -------------------------------------------------------------------
 * 
 * [0x00000000, 0x80000000): MMIO.
 * We use just 2 huge pages to manage them.
 * 
 * -------------------------------------------------------------------
 * 
 * [0x80000000, ekernel): Kernel code part.
 * We use detailed normal page to manage them.
 * For example, the rodata part is read only, while the text part is
 * execute only.
 * It will takes no more than 1 leaf page table.
 * 
 * -------------------------------------------------------------------
 * 
 * [ekernel, 0x80200000): Padding. (unused, so unmaped)
 * It is in the same leaf page table as kernel code, so we choose
 * to set those entries as invalid.
 * 
 * -------------------------------------------------------------------
 * 
 * [0x80200000, mem_end): Kernel memory management.
 * This part should take no more than 1 middle page table.
 * We do not use leaf page table since that's too costly.
 * We set their entries as read/write only.
 * 
 * -------------------------------------------------------------------
 */

#[allow(unused_imports)]
use crate::message;
use crate::{console::print_separator, driver::get_mem_end};

use super::{constant::*, frame::FrameAllocator};

#[derive(Clone, Copy)]
pub struct PageAddress(pub u64);
#[derive(Clone, Copy)]
pub struct PageTableEntry(u64);
#[derive(Clone, Copy)]
pub struct PTEFlag(u64);

unsafe fn get_kernel_page_num(x : usize) -> usize {
    extern "C" { fn skernel(); }
    return (x - skernel as usize) / PAGE_SIZE;
}

unsafe fn set_huge(mut page : PageAddress, i : usize, flag : PTEFlag) {
    page[i].set_entry(PageAddress::new_huge(i as _), flag);
}

unsafe fn set_medium(mut page : PageAddress, i : usize, j : usize, flag : PTEFlag) {
    page[j].set_entry(PageAddress::new_medium(i as _, j as _), flag);
}

unsafe fn set_normal(mut page : PageAddress, i : usize, j : usize, k : usize, flag : PTEFlag) {
    page[k].set_entry(PageAddress::new_normal(i as _, j as _, k as _), flag);
}

// Initialize the huge page table.
pub unsafe fn init_huge_page() {
    logging!("Initialize the page table.");
    let mut root = PAGE_TABLE;

    // Reset as invalid.
    for i in 3..256 {
        set_huge(root, i, PageTableEntry::INVALID);
    }

    // Set MMIO as read/write only.
    set_huge(root, 0, PageTableEntry::RW);
    set_huge(root, 1, PageTableEntry::RW);

    // Kernel part below.
    // Set kernel part using middle size page.

    // Set the second level page table.
    let mut page = PageAddress::new_ptr(FrameAllocator::allocate_page());
    root[2].set_entry(page, PageTableEntry::NEXT);

    // Set the kernel code in details.
    let leaf = PageAddress::new_ptr(FrameAllocator::allocate_page());
    page[0].set_entry(leaf, PageTableEntry::NEXT);
    init_kernel_page(leaf);

    let mem_end = get_kernel_page_num(get_mem_end()) >> 9;

    // Set the kernel memory as read/write only.
    for i in 1..mem_end {
        set_medium(page, 2, i, PageTableEntry::RW);
    }
    for i in mem_end..512 {
        set_medium(page, 2, i, PageTableEntry::INVALID);
    }

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

    let text_start  = get_kernel_page_num(stext as usize);
    let text_finish = get_kernel_page_num(etext as usize);
    for i in text_start..text_finish {
        set_normal(leaf, 2, 0, i, PageTableEntry::X);
    }
    // message!("text_start: {}, text_finish: {}", text_start, text_finish);

    let rodata_start  = get_kernel_page_num(srodata as usize);
    let rodata_finish = get_kernel_page_num(erodata as usize);
    for i in rodata_start..rodata_finish {
        set_normal(leaf, 2, 0, i, PageTableEntry::R);
    }
    // message!("rodata_start: {}, rodata_finish: {}", rodata_start, rodata_finish);

    let data_start  = get_kernel_page_num(sdata as usize);
    let data_finish = get_kernel_page_num(edata as usize);
    for i in data_start..data_finish {
        set_normal(leaf, 2, 0, i, PageTableEntry::RW);
    }
    // message!("data_start: {}, data_finish: {}", data_start, data_finish);

    let bss_start  = get_kernel_page_num(sbss_real as usize);
    let bss_finish = get_kernel_page_num(ebss as usize);
    for i in bss_start..bss_finish {
        set_normal(leaf, 2, 0, i, PageTableEntry::RW);
    }
    // message!("bss_start: {}, bss_finish: {}", bss_start, bss_finish);

    let finish = get_kernel_page_num(ekernel as usize);
    for i in finish..512 {
        set_normal(leaf, 2, 0, i, PageTableEntry::INVALID);
    }
    // message!("Kernel page finish at {}", finish);
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
    // Invalid page table entry flag.
    pub const INVALID   : PTEFlag = PTEFlag(0);
    // Normal page table entry settings.
    pub const X     : PTEFlag = PTEFlag(PTEFlag::VALID | PTEFlag::EXECUTE);
    pub const R     : PTEFlag = PTEFlag(PTEFlag::VALID | PTEFlag::READ);
    pub const RW    : PTEFlag = PTEFlag(PTEFlag::VALID | PTEFlag::READ | PTEFlag::WRITE);
    pub const RX    : PTEFlag = PTEFlag(PTEFlag::VALID | PTEFlag::READ | PTEFlag::EXECUTE);
    pub const RWX   : PTEFlag = PTEFlag(PTEFlag::VALID | PTEFlag::READ | PTEFlag::WRITE | PTEFlag::EXECUTE);
    pub const NEXT  : PTEFlag = PTEFlag(PTEFlag::VALID);

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
