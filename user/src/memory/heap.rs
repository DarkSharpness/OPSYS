use core::ptr::null_mut;

use crate::memory::list::Header;

use super::list::Node;

pub(super) struct Heap {
    heap_end : usize,
    freelist : Node,
}

#[allow(static_mut_refs)]
pub(super) fn get_heap() -> &'static mut Heap {
    static mut HEAP : Heap = Heap {
        heap_end : 0,
        freelist : Node::new_unchecked(),
    };
    unsafe { &mut HEAP }
}

impl Heap {
    pub unsafe fn init(&mut self, heap_start : usize, heap_end : usize) {
        self.freelist.init();
        self.heap_end = heap_end;

        // Use the first 8 bytes as the boundary
        let header = heap_start as *mut Header;
        (*header).set_size_with(0, true);   // Busy
        (*header).set_prev_size(0);         // No previous block

        let header      = header.add(1);
        let real_end    = heap_end - size_of::<Header>();
        let size        = real_end - (*header).get_data() as usize;

        assert_eq!(size, 4096 - 3 * size_of::<Header>(), "Heap size mismatch");

        (*header).set_size_with(size as _, false);  // Free
        (*header).set_prev_size(0);                 // Dummy

        self.freelist.insert((*header).get_node());

        let header = real_end as *mut Header;
        (*header).set_size_with(0, true);   // Busy
        (*header).set_prev_size(size as _); // Previous block size
    }
}

impl Heap {
    pub fn last_header(&self) -> *mut Header {
        let heap_end = self.heap_end;
        let header = heap_end as *mut Header;
        let header = header.wrapping_sub(1);
        return header;
    }

    pub unsafe fn enlarge(&mut self, sbrk : *mut u8, size : usize) {
        assert_eq!(sbrk as usize, self.heap_end, "Invalid sbrk");

        let real_size = size - size_of::<Header>();
        let header = self.last_header();

        (*header).set_size_with(real_size as _, false);
        self.freelist.insert((*header).get_node());

        self.heap_end += size;
        let header = self.last_header();

        (*header).set_size_with(0, true);
        (*header).set_prev_size(real_size as _);
    }

    pub unsafe fn first_fit(&mut self, size : usize) -> *mut Header {
        let head = &mut self.freelist;
        let mut node = (*head).get_next();
        while node != head {
            let this = &mut *node;
            let header = this.get_header();
            if (*header).get_size() as usize >= size {
                this.remove();
                assert!((*header).is_free(), "Corrupted freelist");
                (*header).set_busy();
                return header;
            }
            node = this.get_next();
        }
        return null_mut();
    }

    pub unsafe fn recycle(&mut self, header : *mut Header) {
        assert!(!(*header).is_free(), "Recycling a free block");
        (*header).set_free();
        self.freelist.insert((*header).get_node());
    }
}
