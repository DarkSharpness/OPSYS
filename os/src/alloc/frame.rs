use crate::{console::print_separator, warning};

use super::constant::*;
pub struct FrameAllocator;

impl FrameAllocator {
    pub unsafe fn first_init() {
        let beg = FRAME_START as *mut u8;   // End of bitmap
        let end = BUDDY_START;              // Start of buddy
        // Count of available pages
        let cnt = end.offset_from(beg) as usize / PAGE_SIZE - 1;

        // Use the first page to store recycled pages.
        *FRAME_START = cnt as u16;

        let mut ptr = FRAME_START;
        for i in 0..cnt {
            ptr = ptr.offset(1);
            *ptr = (i + 1) as u16;
        }
    }

    pub unsafe fn allocate_page() -> * mut u8  {
        let cnt = *FRAME_START;
        if cnt == 0 { panic!("Out of pages!"); }
        *FRAME_START = cnt - 1;

        let num = FRAME_START.offset(cnt as _).read() as usize;
        let beg = FRAME_START as *mut u8;
        return beg.wrapping_add(num * PAGE_SIZE)
    }

    pub unsafe fn deallocate_page(page: * mut u8) {
        let num = (page as usize - FRAME_START as usize) / PAGE_SIZE;
        let cnt = (*FRAME_START) + 1;
        *FRAME_START = cnt;
        FRAME_START.offset(cnt as _).write(num as _);
    }

    pub unsafe fn size() -> usize {
        return FRAME_START.read() as _;
    }

    pub unsafe fn debug() {
        let cnt = FrameAllocator::size();
        warning!("Available frame page count: {}", cnt);
        if cnt != 0 {
            warning!("- Last page: {}", FRAME_START.offset(cnt as _).read());
        }
        print_separator();
    }
}
