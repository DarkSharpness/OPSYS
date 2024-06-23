mod basic;
mod thread;
use core::ptr::null_mut;
use crate::IPCHandle;
pub use basic::pm_dump;

extern crate alloc;
use alloc::{collections::BTreeMap, vec::Vec, boxed::Box};
use thread::{CondVQueue, MutexQueue};

pub struct Node {
    pid     : usize,
    parent  : *mut Node,
    child   : Vec<*mut Node>,
    killed  : bool, // false: alive, true: dead
    handle  : Option<IPCHandle>,

    mutex_map   : BTreeMap<usize, MutexQueue>,   // Mutex queue
    condv_map   : BTreeMap<usize, CondVQueue>,   // Condition variable queue

    mutex_cnt   : usize,
    condv_cnt   : usize,

    exit_code   : i32,
}

static mut POOL : BTreeMap <usize, Box<Node>> = BTreeMap::new();

pub unsafe fn get_node(pid : usize) -> &'static mut Node {
    let node = POOL.entry(pid).or_insert(Box::new(Node::new(pid)));
    return &mut *node;
}

unsafe fn remove_node(pid : usize) {
    assert!(POOL.remove(&pid).is_some(), "::remove_node: node not found.");
}

impl Node {
    fn new(pid : usize) -> Node {
        Node {
            pid,
            parent  : null_mut(),
            child   : Vec::new(),
            killed  : false,
            handle  : None,
            exit_code : 0,
            mutex_map : BTreeMap::new(),
            condv_map : BTreeMap::new(),
            mutex_cnt : 0,
            condv_cnt : 0,
        }
    }
}

impl Node {
    fn is_dead(&self) -> bool {
        self.killed == true
    }
    fn is_orphan(&self) -> bool {
        self.parent == core::ptr::null_mut()
    }
    fn get_exit_code(&self) -> i32 {
        self.exit_code
    }
    unsafe fn get_pid(&self) -> usize {
        return self.pid;
    }
    fn get_child(&self, index : usize) -> &mut Node {
        let child = self.child[index];
        return unsafe { child.as_mut().unwrap() };
    }
    fn get_parent(&self) ->&mut Node {
        return unsafe { self.parent.as_mut().unwrap() };
    }
}

impl Node {
    fn set_dead(&mut self) {
        self.killed = true;
    }
    fn set_parent(&mut self, parent : &mut Node) {
        self.parent = parent;
    }
    fn set_orphan(&mut self) {
        self.parent = core::ptr::null_mut();
    }
    fn set_exit_code(&mut self, exit_code : i32) {
        self.exit_code = exit_code;
    }
    fn set_waiting(&mut self, handle: IPCHandle) {
        self.handle = Some(handle);
    }
    fn destroy(&mut self) {
        assert!(self.is_dead());
        unsafe { remove_node(self.pid); }
    }
}
