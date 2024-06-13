use core::cmp::max;

use crate::alloc::{PageAddress, PAGE_SIZE};

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

    pub fn sbrk(&mut self, increment: isize) -> usize {
        let old_break = self.break_start;
        if increment > 0 {
            let new_break = old_break + increment as usize;
            self.break_start = new_break;
            todo!("Map between {:#x} and {:#x}", old_break, new_break);            
        } else if increment < 0 {
            let new_break = max(old_break as isize + increment, self.program_end as isize) as usize;
            self.break_start = new_break;
            todo!("Unmap between {:#x} and {:#x}", new_break, old_break);
        } else {
            return old_break;
        }
    }

}
