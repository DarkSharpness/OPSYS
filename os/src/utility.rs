extern crate alloc;
use alloc::{boxed::Box, vec::Vec, collections::VecDeque};

pub struct DequeIter {
    deque : * mut VecDeque<u8>,
}
pub struct SliceIter {
    beg   : * mut u8,
    end   : * mut u8,
}

impl DequeIter {
    pub fn new(deque : & mut VecDeque<u8>) -> Self {
        Self { deque }
    }
}

impl SliceIter {
    pub fn new(slice : & mut Box<[u8]>) -> Self {
        let beg = slice.as_mut_ptr();
        let end = unsafe { beg.add(slice.len()) };
        Self { beg, end }
    }
    pub fn new_vec(slice : & mut Vec<u8>) -> Self {
        let beg = slice.as_mut_ptr();
        let end = unsafe { beg.add(slice.len()) };
        Self { beg, end }
    }
}

pub trait CanPush { fn push_n(&mut self, src : &[u8]); }
pub trait CanCopy { fn copy_n(&mut self, dst : &mut [u8]); }

impl CanPush for DequeIter {
    fn push_n(&mut self, src : &[u8]) {
        for i in src.iter() {
            unsafe { (*self.deque).push_back(*i); }
        }
    }
}

impl CanCopy for DequeIter  {
    fn copy_n(&mut self, dst : &mut [u8]) {
        let deque = unsafe { &mut *self.deque };
        let (front, back) = deque.as_slices();
        let dst_len     = dst.len();
        let front_len   = front.len();
        if dst_len <= front_len {
            dst.copy_from_slice(&front[..dst.len()]);
        } else {
            dst[..front_len].copy_from_slice(front);
            let rest = dst_len - front_len;
            dst[front_len..].copy_from_slice(&back[..rest]);
        }
        deque.drain(..dst_len);
    }
}

impl CanPush for SliceIter {
    fn push_n(&mut self, src : &[u8]) {
        let slice = unsafe { core::slice::from_raw_parts_mut(self.beg, src.len()) };
        slice.copy_from_slice(src);
        self.beg = unsafe { self.beg.add(src.len()) };
        assert!(self.beg <= self.end);
    }
}

impl CanCopy for SliceIter {
    fn copy_n(&mut self, dst : &mut [u8]) {
        let slice = unsafe { core::slice::from_raw_parts(self.beg, dst.len()) };
        dst.copy_from_slice(slice);
        self.beg = unsafe { self.beg.add(dst.len()) };
        assert!(self.beg <= self.end);
    }
}
