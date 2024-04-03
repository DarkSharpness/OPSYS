use crate::uart_println;

use super::constant::*;
pub struct FrameAllocator;

impl FrameAllocator {
    pub unsafe fn first_init() {
        let beg = MAP_LAST as *mut u8;  // End of bitmap
        let end = BASE;                 // Start of buddy

        // Count of available pages
        let cnt = end.offset_from(beg) as usize / PAGE_SIZE - 1;

        // Use the first page to store recycled pages.
        let mut ptr = MAP_LAST as *mut u16;
        *ptr = cnt as u16;

        for i in 0..cnt {
            ptr = ptr.offset(1);
            *ptr = (i + 1) as u16;
        }
    }

    pub unsafe fn allocate_page() -> * mut u8  {
        let ptr = MAP_LAST as *mut u16;
        let cnt = *ptr;
        if cnt == 0 { panic!("Out of pages!"); }
        *ptr = cnt - 1;

        let num = ptr.offset(cnt as _).read() as usize;
        let beg = MAP_LAST as *mut u8;

        return beg.wrapping_add(num * PAGE_SIZE)
    }

    pub unsafe fn deallocate_page(page: * mut u8) {
        let num = (page as usize - MAP_LAST) / PAGE_SIZE;
        let ptr = MAP_LAST as *mut u16;
        let cnt = *ptr;
        *ptr = cnt + 1;

        ptr.offset(1).offset(cnt as _).write(num as _);
    }

    pub unsafe fn size() -> usize {
        return (MAP_LAST as *mut u16).read() as _;
    }

    pub unsafe fn debug() {
        let cnt = FrameAllocator::size();
        uart_println!("Available page count: {}", cnt);
        if cnt != 0 {
            uart_println!("- Last page: {}",
                (MAP_LAST as *mut u16).offset(cnt as _).read());
        }
    }
}
