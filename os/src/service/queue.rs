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
        self.tail = new_iter();
        self.head = self.tail.clone();
    }

    pub unsafe fn push(&mut self, process : *mut Process) {
        *self.tail = process;
        (*process).service = self.tail.clone();
        next_tail(&mut self.tail);
    }

    pub unsafe fn front(&mut self) -> Option<Iterator> {
        while self.head != self.tail {
            // The service has been closed by the process.
            if (*self.head).is_null() {
                next_head(&mut self.head);
            } else {
                return Some(self.head.clone());
            }
        }
        return None;
    }
}
