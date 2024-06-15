use core::ptr::null_mut;

use crate::{memory::list::Header, println};

use super::list::Node;

pub(super) struct Heap {
    heap_beg : usize, // Debug only
    heap_end : usize,
    freelist : Node,
}

#[allow(static_mut_refs)]
pub(super) fn get_heap() -> &'static mut Heap {
    static mut HEAP : Heap = Heap {
        heap_beg : 0,
        heap_end : 0,
        freelist : Node::new_unchecked(),
    };
    unsafe { &mut HEAP }
}

impl Heap {
    const INIT_SIZE  : u32  = (4096 - 2 * size_of::<Header>()) as _;
    const DUMMY_SIZE : u32  = (size_of::<Header>() as _);
    const MAGIC_SIZE : u32  = 191981000; // Magic number

    pub unsafe fn init(&mut self, heap_start : usize, heap_end : usize) {
        self.freelist.init();
        self.heap_beg = heap_start;
        self.heap_end = heap_end;

        // Use the first 8 bytes as the boundary
        let header = heap_start as *mut Header;
        (*header).set_prev_size(Self::MAGIC_SIZE);  // No previous block
        (*header).set_size_with(Self::DUMMY_SIZE, true);

        let header      = header.add(1);
        let real_end    = heap_end - size_of::<Header>();
        let size        = real_end - (header as usize);

        assert_eq!(size, Self::INIT_SIZE as usize, "Heap size mismatch");

        (*header).set_size_with(size as _, false);      // Free
        (*header).set_prev_size(Self::DUMMY_SIZE as _); // Dummy

        self.freelist.insert((*header).get_node());

        let header = real_end as *mut Header;
        (*header).set_size_with(Self::MAGIC_SIZE, true);// Busy
        (*header).set_prev_size(size as _); // Previous block size
    }

    pub fn last_header(&self) -> *mut Header {
        let heap_end = self.heap_end;
        let header = heap_end as *mut Header;
        let header = header.wrapping_sub(1);
        return header;
    }

    pub unsafe fn enlarge(&mut self, sbrk : *mut u8, size : usize) {
        assert_eq!(sbrk as usize, self.heap_end, "Invalid sbrk");

        let header = self.last_header();
        (*header).set_size_with(size as _, false);

        self.freelist.insert((*header).get_node());
        self.heap_end += size;

        let header = self.last_header();
        (*header).set_size_with(Self::MAGIC_SIZE, true);
        (*header).set_prev_size(size as _);
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

    unsafe fn dump_0(&self) {
        let mut header      = self.heap_beg as *mut Header;
        let mut last_size   = Self::MAGIC_SIZE;
        let last_header     = self.last_header();
        loop {
            let prev = (*header).get_prev_size();
            assert!(prev == last_size, "Corrupted prev_size");
            let size = (*header).get_size();
            let free = (*header).is_free();
            println!("== Header: {:p}, size: {},\tfree: {} ==", header, size, free);
            last_size = size;
            header = (*header).get_next();
            if header == last_header { break; }
        }

        assert!((*last_header).get_size() == Self::MAGIC_SIZE, "Corrupted last header");
    }

    unsafe fn dump_1(&self) {
        let head = &self.freelist as *const Node as usize as * mut Node;
        let mut node = (*head).get_next() as * mut Node;
        while node != head {
            let this = &mut *node;
            let header = this.get_header();
            assert!((*header).is_free(), "Corrupted freelist");
            println!("== Free block: {:p}, size: {} ==", header, (*header).get_size());
            node = this.get_next();
        }
    }

    pub unsafe fn dump_and_check(&self) {
        println!("== Heap dump ==");
        self.dump_0();
        self.dump_1();
        println!("== End of dump ==");
    }
}
