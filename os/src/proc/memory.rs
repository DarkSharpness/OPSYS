use core::cmp::max;

use crate::alloc::{PTEFlag, PageAddress, PAGE_SIZE};

pub struct MemoryArea {
    root        : PageAddress,  // root page table
    program_end : usize,    // End of program data
    break_start : usize,    // Start of heap
}

impl MemoryArea {
    pub fn new() -> MemoryArea {
        MemoryArea {
            root        : PageAddress::new_pagetable(),
            program_end : 0,
            break_start : 0,
        }
    }

    pub(super) fn get_satp(&self) -> PageAddress {
        return self.root.clone();
    }

    pub(super) fn set_program_end(&mut self, end: usize) -> usize {
        let end = (end + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        self.program_end = end;
        self.break_start = end;
        return end;
    }

    pub unsafe fn sbrk(&mut self, increment: isize) -> usize {
        let old_break = self.break_start;
        let new_break = old_break + increment as usize;
        if increment > 0 {
            self.break_start = new_break;

            let root = self.get_satp();
            let old_page = (old_break - 1) / PAGE_SIZE;
            let new_page = (new_break - 1) / PAGE_SIZE;
            for page in (old_page + 1)..=new_page {
                root.try_umap(page * PAGE_SIZE, PTEFlag::RW);
            }

            todo!("Map between {:#x} and {:#x}", old_break, new_break);            
        } else if increment < 0 {
            let new_break = max(new_break, self.program_end);
            self.break_start = new_break;

            let root = self.get_satp();
            let old_page = (old_break - 1) / PAGE_SIZE;
            let new_page = (new_break - 1) / PAGE_SIZE;
            for page in (new_page + 1)..=old_page {
                root.try_unumap(page * PAGE_SIZE);
            }

            todo!("Unmap between {:#x} and {:#x}", new_break, old_break);
        }
        return old_break;
    }

    pub unsafe fn free(&self) {
        let root = self.get_satp();
        root.free();
    }
}
