use core::{ptr::{null, null_mut}, usize};

use crate::{sys_respond, Argument, IPCHandle};

extern crate alloc;
use alloc::{collections::BTreeMap, vec::Vec, boxed::Box};
use user_lib::{print, println};

pub struct Node {
    pid     : usize,
    parent  : *mut Node,
    child   : Vec<*mut Node>,
    killed  : bool, // false: alive, true: dead
    handle  : Option<IPCHandle>,
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
        }
    }

    pub fn insert_child(&mut self, child : &mut Node) {
        assert!(!self.is_dead());
        self.child.push(child);
        child.set_parent(self);
    }

    fn force_remove(&mut self, target : &mut Node) {
        for i in 0..self.child.len() {
            if self.child[i] == target {
                self.child.swap_remove(i);
                return;
            }
        }
        println!("{:?}", self.child);
        panic!("::which_child: target is not a child of self.");
    }

    fn remove_child(&mut self, target : &mut Node) {
        self.force_remove(target);
        target.set_orphan();

        // Some updates are needed here.

        let handle = match self.handle.take() {
            Some(handle)    => handle,
            None            => return
        };

        let pid = unsafe { target.get_pid() };
        return respond_wait(pid, target.get_exit_code(), handle);
    }

    pub fn exit(&mut self, exit_code : i32) {
        assert!(!self.is_dead());

        for i in 0..self.child.len() {
            let child = self.get_child(i);
            assert!(!child.is_orphan());
            if !child.is_dead() {
                child.set_orphan();
            } else {
                child.destroy();
            }
        }

        self.child.clear();
        self.child.shrink_to_fit();

        self.set_exit_code(exit_code);
        self.set_dead();

        if self.is_orphan() {
            self.destroy();
        } else {
            let this = self as *mut Node; // Just to skip borrow checker.
            let this = unsafe {&mut *this};
            self.get_parent().remove_child(this);
        }
    }

    pub fn wait(&mut self, handle: IPCHandle) {
        assert!(!self.is_dead());
        for i in 0..self.child.len() {
            let child = self.get_child(i);
            if child.is_dead() {
                let pid = unsafe { child.get_pid() };
                return respond_wait(pid, child.get_exit_code(), handle);
            }
        }
        self.set_waiting(handle);
    }

    fn dump(&self, indent : usize, parent : *const Node) {
        assert!((self.parent as *const Node) == parent);
        print!("- ");
        for _ in 0..indent { print!("    "); }
        if self.is_dead() {
            assert!(self.child.capacity() == 0);
            println!("[x] pid: {}, exit_code: {}", self.pid, self.exit_code);
        } else {
            println!("[ ] pid: {}", self.pid);
            let this = self as *const Node;
            for i in 0..self.child.len() {
                self.get_child(i).dump(indent + 1, this);
            }
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

fn respond_wait(pid : usize, exit_code : i32, handle : IPCHandle) {
    sys_respond(Argument::Register(pid, exit_code as _), handle);
}

pub fn pm_dump() {
    let mut noroot : Vec<*mut Node> = Vec::new();
    for (_, node) in unsafe { POOL.iter_mut() } {
        let node = &mut **node;
        if node.is_orphan() {
            noroot.push(node);
        }
    }
    println!("== Dumping all nodes: ==");
    for node in noroot {
        unsafe { (*node).dump(0, null()); }
    }
}
