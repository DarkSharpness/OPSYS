use super::constant::*;
pub struct FrameAllocator;


impl FrameAllocator {
    pub unsafe fn first_init() -> usize {
        let mut beg = MAP_LAST as *mut u8;  // End of bitmap
        let     end = BASE;                 // Start of buddy
        (*FRLIST).init();
        let mut cnt = 0;
        while beg != end {
            (*FRLIST).push(beg as _);
            beg = beg.wrapping_add(PAGE_SIZE);
            cnt += 1;
        }
        return cnt;
    }

    pub unsafe fn allocate_page() -> * mut u8  {
        if (*FRLIST).empty() { panic!("Out of pages!"); }
        (*FRLIST).pop() as _
    }

    pub unsafe fn deallocate_page(page: * mut u8) {
        (*FRLIST).push(page as _);
    }
}
