use crate::proc::Process;
use super::node::*;

/** A block list style queue. */
pub struct ServiceQueue {
    head : Iterator,
    tail : Iterator,
}

impl ServiceQueue {
    pub fn new() -> Self {
        let head = Iterator::new();
        let tail = Iterator::new();
        return ServiceQueue { head, tail };
    }

    pub unsafe fn init(&mut self) {
        self.tail.init();
        self.head = self.tail;
    }

    pub unsafe fn push(&mut self, process : *mut Process) {
        *self.tail = process;
        (*process).service = *self.tail as _;
        self.tail.next_tail();
    }

    pub unsafe fn front(&mut self) -> Option<*mut *mut Process> {
        while self.head != self.tail {
            // The service has been closed by the process.
            if (*self.head).is_null() {
                self.head.next_head();
            } else {
                return Some(*self.head as _);
            }
        }
        return None;
    }
}
