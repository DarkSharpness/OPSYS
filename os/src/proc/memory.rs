use core::cmp::max;

use crate::alloc::{PTEFlag, PageAddress, PAGE_SIZE};

pub struct MemoryArea {
    root            : PageAddress,  // root page table
    program_start   : usize,        // Start of program data
    program_finish  : usize,        // End of program data, start of heap
    break_finish    : usize,        // End of heap
    stack_bottom    : usize,        // Bottom of stack
}

const USER_STACK : usize = 1 << 38;

impl MemoryArea {
    pub fn new() -> MemoryArea {
        MemoryArea {
            root            : PageAddress::new_pagetable(),
            program_start   : 0,
            program_finish  : 0,
            break_finish    : 0,
            stack_bottom    : USER_STACK,
        }
    }

    pub(super) fn get_satp(&self) -> PageAddress {
        return self.root.clone();
    }

    pub(super) fn set_program_end(&mut self, end: usize) -> usize {
        let end = (end + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        self.program_finish = end;
        self.break_finish = end;
        return end;
    }

    pub unsafe fn sbrk(&mut self, increment: isize) -> usize {
        let old_break = self.break_finish;
        let new_break = old_break + increment as usize;
        if increment > 0 {
            self.break_finish = new_break;

            let root = self.get_satp();
            let old_page = (old_break - 1) / PAGE_SIZE;
            let new_page = (new_break - 1) / PAGE_SIZE;
            for page in (old_page + 1)..=new_page {
                root.try_umap(page * PAGE_SIZE, PTEFlag::RW);
            }
        } else if increment < 0 {
            let new_break = max(new_break, self.program_finish);
            self.break_finish = new_break;

            let root = self.get_satp();
            let old_page = (old_break - 1) / PAGE_SIZE;
            let new_page = (new_break - 1) / PAGE_SIZE;
            for page in (new_page + 1)..=old_page {
                root.try_unumap(page * PAGE_SIZE);
            }
        }
        return old_break;
    }

    pub const fn get_user_stack_top() -> usize {
        return USER_STACK;
    }

    pub unsafe fn add_stack(&mut self, size: usize) {
        let stack_top = self.stack_bottom;
        let stack_low = stack_top - size * PAGE_SIZE;

        let root = self.get_satp();
        for page in (stack_low / PAGE_SIZE)..(stack_top / PAGE_SIZE) {
            root.try_umap(page * PAGE_SIZE, PTEFlag::RW);
        }
    }

    pub unsafe fn free(&self) {
        let root = self.get_satp();
        root.free();
    }
}

impl PageAddress {
    unsafe fn map_user_stack(self, cnt : usize) {
        for i in 0..cnt {
            let user_stack = USER_STACK - (i + 1) * PAGE_SIZE as usize;
            self.new_umap(user_stack, PTEFlag::RW);
        }
    }
}
