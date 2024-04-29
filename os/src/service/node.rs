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

#[derive(PartialEq, Clone)]
pub struct Iterator {
    data : *mut *mut Process,
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
        unsafe { return &*self.data; }
    }
}

impl DerefMut for Iterator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { return &mut *self.data; }
    }
}

impl Iterator {
    pub fn new() -> Self { return Iterator { data : null_mut() }; }
    pub unsafe fn set_done(&mut self) {
        *self.data = null_mut();
        self.data = null_mut();
    }
    pub unsafe fn in_service(&self) -> bool {
        return !self.data.is_null();
    }
}

/** Create a new iterator using the allocator. */
pub unsafe fn new_iter() -> Iterator {
    let node = QueueNode::new();
    let data = (*node).data.as_mut_ptr();
    return Iterator { data };
}

/** Find the next head. */
pub unsafe fn next_head(iter : &mut Iterator) {
    let addr = iter.data.wrapping_add(1) as usize;
    if addr % NODE_SIZE == 0 {
        let prev = (addr as *mut QueueNode).wrapping_sub(1);
        let next = (*prev).next;
        iter.data = next as _;
        (*prev).drop();
    } else {
        iter.data = addr as _;
    }
}

/** Find the next tail. */
pub unsafe fn next_tail(iter : &mut Iterator) {
    let addr = iter.data.wrapping_add(1) as usize;
    if addr % NODE_SIZE == 0 {
        let prev = (addr as *mut QueueNode).wrapping_sub(1);
        let next = QueueNode::new();
        let data = (*next).data.as_mut_ptr();
        (*prev).next = data as _;
        iter.data = data as _;
    } else {
        iter.data = addr as _;
    }
}
