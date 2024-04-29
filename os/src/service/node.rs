use core::{mem::size_of, ops::{Deref, DerefMut}, ptr::null_mut};
use crate::{alloc::PageAddress, proc::Process};

/** Block-style node. */
const NODE_SIZE : usize = 256;
const DATA_SIZE : usize = (NODE_SIZE - size_of::<*mut QueueNode>()) / size_of::<*mut Process>();

#[repr(C)]
pub struct QueueNode {
    next : *mut QueueNode,
    data : [*mut Process; DATA_SIZE],
}

#[derive(PartialEq, Clone, Copy)]
pub struct Iterator {
    node : *mut QueueNode,
    index : usize,
}

static mut FREE : *mut QueueNode = null_mut();
static mut SIZE : usize = 0;

impl QueueNode {
    unsafe fn grow() -> *mut QueueNode {
        const SIZE : usize = 4096 / size_of::<QueueNode>() - 1;

        let page = PageAddress::new_rand_page();
        let addr = page.address();
        let node = addr as *mut QueueNode;

        let mut iter = node.offset(1);
        FREE = iter;

        for _ in 0..SIZE - 1 {
            let next = iter.offset(1);
            (*iter).next = next;
            iter = next;
        }
        (*iter).next = null_mut();

        return node;
    }

    unsafe fn new() -> *mut QueueNode {
        SIZE += 1;
        if FREE.is_null() {
            return QueueNode::grow();
        } else {
            let node = FREE;
            FREE = (*node).next;
            return node;
        }
    }

    unsafe fn drop(&mut self) {
        self.next = FREE;
        FREE = self as *mut QueueNode;
        SIZE -= 1;
    }
}

impl Deref for Iterator {
    type Target = *mut Process;
    fn deref(&self) -> &Self::Target {
        unsafe { return &(*self.node).data[self.index]; }
    }
}

impl DerefMut for Iterator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { return &mut (*self.node).data[self.index]; }
    }
}

impl Iterator {
    pub fn new() -> Self { return Iterator { node : null_mut(), index : 0 }; }

    /* Initialize the iterator with a new page. */
    pub unsafe fn init(&mut self) {
        self.node = QueueNode::new();
        self.index = 0;
    }

    pub unsafe fn next_head(&mut self) {
        self.index += 1;
        if self.index == DATA_SIZE {
            let next = (*self.node).next;
            (*self.node).drop();
            self.node = next;
            self.index = 0;
        }
    }

    pub unsafe fn next_tail(&mut self) {
        self.index += 1;
        if self.index == DATA_SIZE {
            let next = QueueNode::new();
            (*self.node).next = next;
            self.node = next;
            self.index = 0;
        }
    }
}

