use core::ptr::null_mut;

use crate::println;

pub struct Node {
    prev : *mut Node,
    next : *mut Node,
}

pub struct Header {
    last_size : u32,
    this_size : u32,
}

pub unsafe fn link(a : *mut Node, b : *mut Node) {
    (*a).next = b; (*b).prev = a;
}

impl Node {
    pub const fn new_unchecked() -> Self {
        Node {
            prev : null_mut(),
            next : null_mut(),
        }
    }

    pub fn init(&mut self) {
        self.prev = self as *mut Node;
        self.next = self as *mut Node;
    }

    pub unsafe fn insert(&mut self, temp : *mut Node) {
        let next = self.next;
        link(self, temp);
        link(temp, next);
    }

    pub unsafe fn is_empty(&self) -> bool {
        self.next as *const Node == self
    }

    pub unsafe fn remove(&mut self) {
        let prev = self.prev;
        let next = self.next;
        link(prev, next);
    }

    pub unsafe fn get_header(&mut self) -> *mut Header {
        let ptr = self as *mut Node as usize;
        let ptr = ptr - core::mem::size_of::<Header>();
        return &mut *(ptr as *mut Header)
    }

    pub unsafe fn get_next(&self) -> *mut Node {
        self.next
    }
}

impl Header {
    pub fn set_free(&mut self) {
        self.this_size = self.this_size & !0x1;
    }
    pub fn set_busy(&mut self) {
        self.this_size = self.this_size | 0x1;
    }
    pub fn is_free(&self) -> bool {
        self.this_size & 0x1 == 0
    }
    pub fn set_size(&mut self, size : u32) {
        self.this_size = size | (self.this_size & 0x7);
    }
    pub fn set_size_with(&mut self, size : u32, busy : bool) {
        self.this_size = size | (busy as u32);
    }
    pub fn set_prev_size(&mut self, size : u32) {
        self.last_size = size;
    }
    pub fn get_prev_size(&self) -> u32 {
        self.last_size
    }
    pub fn get_size(&self) -> u32 {
        self.this_size & !0x7
    }
    pub fn get_node(&mut self) -> *mut Node {
        return unsafe {&mut *(self.get_data() as *mut Node) };
    }
    pub fn get_data(&mut self) -> *mut u8 {
        let ptr = self as *mut Header as usize;
        let ptr = ptr + core::mem::size_of::<Header>();
        return ptr as *mut u8
    }
    pub fn get_next(&mut self) -> *mut Header {
        let address = self as *mut Header as usize;
        return (address + (self.get_size() as usize)) as *mut Header
    }
    pub fn get_prev(&mut self) -> *mut Header {
        let address = self as *mut Header as usize;
        let prev_size = self.get_prev_size() as usize;
        return (address - prev_size) as *mut Header
    }
    pub unsafe fn try_split(&mut self, size : usize) -> (*mut u8, Option<&mut Header>) {
        return self.try_split_impl(size);
    }

    #[allow(unused)]
    unsafe fn try_split_impl(&mut self, size : usize) -> (*mut u8, Option<&mut Header>) {
        let size     = size as u32;
        let capacity = self.get_size();
        let rest     = capacity - size;
        if rest <= size || rest <= 32 {
            return (self.get_data(), None);
        }

        println!("Splitting: {} -> {} + {}", capacity, size, rest);

        // From [capacity] to [size] to [rest]
        let next_header = self.get_next();
        (*next_header).set_prev_size(rest);

        self.set_size(size as _);
        let next_header = self.get_next();
        (*next_header).set_prev_size(size);
        (*next_header).set_size_with(rest, true);

        return (self.get_data(), Some(&mut *next_header));
    }
}
