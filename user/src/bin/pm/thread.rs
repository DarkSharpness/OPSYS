use crate::Node;

extern crate alloc;
use alloc::collections::VecDeque;
use user_lib::{Argument, IPCHandle};

pub struct MutexQueue {
    locked  : bool,
    queue   : VecDeque<IPCHandle>,
}

pub struct CondVQueue {
    queue   : VecDeque<(usize, IPCHandle)>,
}

impl Node {
    pub fn mutex_create(&mut self) -> usize {
        self.mutex_map.insert(self.mutex_cnt, MutexQueue::new());
        self.mutex_cnt += 1;
        return self.mutex_cnt - 1;
    }
    pub fn mutex_destroy(&mut self, mutex_id: usize) {
        self.mutex_map.remove(&mutex_id);
    }
    pub fn mutex_lock(&mut self, mutex_id: usize, handle: IPCHandle) {
        if let Some(queue) = self.mutex_map.get_mut(&mutex_id) {
            queue.lock(handle);
        } else {
            handle.respond(Argument::Register(!0, 0));
        }
    }
    pub fn mutex_unlock(&mut self, mutex_id: usize) -> bool {
        if let Some(queue) = self.mutex_map.get_mut(&mutex_id) {
            return queue.unlock();
        } else {
            return false;
        }
    }

    pub fn condv_create(&mut self) -> usize {
        self.condv_map.insert(self.condv_cnt, CondVQueue::new());
        self.condv_cnt += 1;
        return self.condv_cnt - 1;
    }

    pub fn condv_destroy(&mut self, condv_id: usize) {
        self.condv_map.remove(&condv_id);
    }

    pub fn condv_wait(&mut self, condv_id: usize, mutex_id: usize, handle: IPCHandle) {
        self.mutex_unlock(mutex_id);
        if let Some(queue) = self.condv_map.get_mut(&condv_id) {
            queue.wait(mutex_id, handle);
        } else {
            handle.respond(Argument::Register(1, 0));
        }
    }

    pub fn condv_signal(&mut self, condv_id: usize) -> bool {
        if let Some(queue) = self.condv_map.get_mut(&condv_id) {
            match queue.signal() {
                Some((mutex_id, handle)) => {
                    self.mutex_lock(mutex_id, handle);
                    return true;
                }
                None => {
                    return false;
                }                
            }
        } else {
            return false;
        }
    }

    pub fn condv_broadcast(&mut self, condv_id: usize) -> bool {
        if let Some(queue) = self.condv_map.get_mut(&condv_id) {
            // Move the storage out of the queue
            let handles : VecDeque<(usize, IPCHandle)>
                = queue.queue.drain(..).collect();
            assert!(queue.queue.len() == 0);
            for (mutex_id, handle) in handles {
                self.mutex_lock(mutex_id, handle);
            }
            return true;
        } else {
            return false;
        }
    }
}

impl MutexQueue {
    pub fn new() -> Self {
        MutexQueue {
            locked  : false,
            queue   : VecDeque::new(),
        }
    }

    pub fn lock(&mut self, handle: IPCHandle) {
        if self.locked {
            self.queue.push_back(handle);
        } else {
            self.locked = true;
            handle.respond(Argument::Register(0, 0));
        }
    }

    pub fn unlock(&mut self) -> bool {
        if self.queue.len() > 0 {
            self.locked = true;
            let handle = self.queue.pop_front().expect("?");
            handle.respond(Argument::Register(0, 0));
            return true;
        } else {
            self.locked = false;
            return false;
        }
    }
}

impl CondVQueue {
    pub fn new() -> Self {
        CondVQueue {
            queue   : VecDeque::new(),
        }
    }

    pub fn wait(&mut self, mutex_id : usize, handle: IPCHandle) {
        self.queue.push_back((mutex_id, handle));
    }

    pub fn signal(&mut self) -> Option<(usize, IPCHandle)> {
        if self.queue.len() > 0 {
            return self.queue.pop_front();
        } else {
            return None;
        }
    }
}
