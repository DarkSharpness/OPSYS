use core::ptr::null_mut;

use crate::{proc::{Process, ProcessStatus}, syscall::sys_yield};
use super::node::*;

/** A block list style queue. */
pub struct ServiceQueue {
    head    : Iterator,
    tail    : Iterator,
    service : *mut Process,
}

impl ServiceQueue {
    pub const fn new() -> Self {
        let head = Iterator::new();
        let tail = Iterator::new();
        let service = null_mut();
        return ServiceQueue { head, tail , service };
    }

    pub unsafe fn init(&mut self) {
        self.tail = new_iter();
        self.head = self.tail.clone();
    }

    pub unsafe fn accept(&mut self) -> Iterator {
        loop {
            while self.head != self.tail {
                if (*self.head).is_null() {
                    next_head(&mut self.head);
                } else {
                    return self.head.clone();
                }
            }
            sys_yield(); // Wait for new service.
        }
    }

    /** Append one more process. */
    pub unsafe fn register(&mut self, process : *mut Process) {
        *self.tail = process;
        (*process).service  = self.tail.clone();
        (*process).status   = ProcessStatus::INSERVICE;
        next_tail(&mut self.tail);
        // Tries to wake up the acceptor.
    }

    pub unsafe fn get_handler(&self) -> *mut Process {
        return self.service;
    }
}
