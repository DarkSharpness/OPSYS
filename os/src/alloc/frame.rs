use core::mem::size_of;
use crate::console::print_separator;
use super::constant::*;
pub struct FrameAllocator;

struct VecPage(*mut u16, *mut u8);

const FRAME : VecPage = VecPage::new();

impl VecPage {
    const fn get_avail_size() -> usize {
        let beg = FRAME_START_ADDR;
        let end = BUDDY_START_ADDR;
        let cnt = (end - beg) / PAGE_SIZE;
        let max = u16::MAX as usize;
        if cnt > max { return max; }
        else         { return cnt; }
    }

    const fn get_vec_info() -> (usize, usize) {
        // Available size in all.
        let avail = VecPage::get_avail_size();
        // Manager needs entries * sizeof(u16) storage bytes.
        let storage = (avail - 1) * size_of::<u16>();
        // Header front offset, at least 1
        let headers = storage / PAGE_SIZE + 1;
        // Count of remaining pages, which can be used as memory.
        let remains = avail - headers;
        // Return the desired data pair.
        return (headers, remains);
    }

    const HEADERS   : usize = VecPage::get_vec_info().0;
    const MAXSIZE   : usize = VecPage::get_vec_info().1;
    const ENDSIZE   : usize = VecPage::get_avail_size();

    const fn new() -> Self {
        return VecPage(FRAME_START, FRAME_START as _);
    }

    unsafe fn first_init(&self) {
        let mut data = self.0;
        data.write(VecPage::MAXSIZE as _);
        for i in (VecPage::HEADERS..VecPage::ENDSIZE).rev() {
            data = data.wrapping_add(1);
            data.write(i as _);
        }
    }

    unsafe fn pop_back(&self) -> *mut u8 {
        let data = self.0;
        let size = data.read();
        assert!(size != 0, "Out of pages!");
        data.write(size - 1);
        let page = data.wrapping_add(size as _).read() as usize;
        return self.1.wrapping_add(page * PAGE_SIZE);
    }

    unsafe fn push_back(&self, page : *mut u8) {
        let data = self.0;
        let size = data.read() + 1;
        data.write(size);
        let page = (page as usize - self.1 as usize) / PAGE_SIZE;

        data.wrapping_add(size as _).write(page as _);
    }

    unsafe fn size(&self) -> usize {
        return self.0.read() as _;
    }
}

impl core::ops::Index<usize> for VecPage {
    type Output = u16;
    fn index(&self, x : usize) -> &u16 {
        return unsafe {&*self.0.wrapping_add(1 + x)};
    }
}

impl FrameAllocator {
    pub unsafe fn first_init() {
        return FRAME.first_init();
    }

    pub unsafe fn allocate_page() -> * mut u8  {
        return FRAME.pop_back();    
    }

    pub unsafe fn deallocate_page(page: * mut u8) {
        return FRAME.push_back(page);
    }

    pub unsafe fn size() -> usize {
        return FRAME.size();
    }

    pub unsafe fn debug() {
        let size = FRAME.size();
        warning!("Available frame page count: {}", size);
        if size != 0 {
            warning!("- First page: {}", FRAME[0]);
            warning!("- Last  page: {}", FRAME[size - 1]);
        }
        print_separator();
    }
}
