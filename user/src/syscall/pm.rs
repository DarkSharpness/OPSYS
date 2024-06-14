use crate::{sys_respond, Argument, IPCHandle};

pub struct Node {
    parent  : *mut Node,
    child   : [*mut Node; 8],
    killed  : bool, // false: alive, true: dead
    handle  : Option<IPCHandle>,
    exit_code   : i32,
}

const REPEAT : Node = Node::new(core::ptr::null_mut());

pub unsafe fn get_node(pid : usize) -> &'static mut Node {
    static mut NODE_POOL : [Node; 1024] = [REPEAT; 1024];
    &mut NODE_POOL[pid - 1]
}

impl Node {
    const fn new(parent : *mut Node) -> Node {
        Node {
            parent,
            child   : [core::ptr::null_mut(); 8],
            killed  : false,
            handle  : None,
            exit_code : 0,
        }
    }

    pub fn add_child(&mut self, child : *mut Node) {
        assert!(!self.is_dead());
        for i in 0..self.child.len() {
            if self.child[i] == core::ptr::null_mut() {
                self.child[i] = child;
                unsafe { (*child).parent = self as *mut Node };
                return;
            }
        }
        panic!("No more space for child node!");
    }

    pub fn exit(&mut self, exit_code : i32) {
        assert!(!self.is_dead());
        for i in 0..self.child.len() {
            let child = match self.get_child(i) {
                Some(child) => child,
                None => continue
            };
            assert!(!child.is_orphan());
            if !child.is_dead() {
                child.set_orphan();
            } else {
                child.destroy();
            }
        }

        let this = self as *mut Node;
        self.set_exit_code(exit_code);
        self.set_dead();
        match self.get_parent() {
            Some(parent)    => parent.set_child_die(this),
            None            => self.destroy()
        }
    }

    pub fn wait(&mut self, handle: IPCHandle) {
        assert!(!self.is_dead());
        for i in 0..self.child.len() {
            let child = match self.get_child(i) {
                Some(child) => child,
                None => continue
            };
            if child.is_dead() {
                let pid = unsafe { child.get_pid() };
                return respond_wait(pid, child.get_exit_code(), handle);
            }
        }
        self.set_waiting(handle);
    }
}

impl Node {
    fn is_dead(&self) -> bool {
        self.killed == true
    }
    fn is_orphan(&self) -> bool {
        self.parent == core::ptr::null_mut()
    }
    fn is_waiting(&self) -> bool {
        self.handle.is_some()
    }
    fn get_exit_code(&self) -> i32 {
        self.exit_code
    }
    unsafe fn get_pid(&self) -> usize {
        let begin = get_node(0) as *const Node;
        let end = self as *const Node;
        return end.offset_from(begin) as usize + 1;
    }
    fn get_child(&self, index : usize) -> Option<&mut Node> {
        let child = self.child[index];
        if child == core::ptr::null_mut() {
            None
        } else {
            Some(unsafe { &mut *child })
        }
    }
    fn get_parent(&mut self) -> Option<&mut Node> {
        if self.parent == core::ptr::null_mut() {
            None
        } else {
            Some(unsafe { &mut *self.parent })
        }
    }
}

impl Node {
    fn set_dead(&mut self) {
        self.killed = true;
    }
    fn set_child_die(&mut self, target : *mut Node) {
        for i in 0..self.child.len() {
            if self.child[i] == target {
                self.child[i] = core::ptr::null_mut();
                unsafe { return self.try_handle_wait(&mut *target) };
            }
        }
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
    fn try_handle_wait(&mut self, child : &mut Node) {
        let handle = match self.handle.take() {
            Some(handle)    => handle,
            None            => return
        };
        let pid = unsafe { child.get_pid() };
        return respond_wait(pid, child.get_exit_code(), handle);
    }
    fn destroy(&mut self) {
        /* Do nothing now. */
    }
}

fn respond_wait(pid : usize, exit_code : i32, handle : IPCHandle) {
    sys_respond(Argument::Register(pid, exit_code as _), handle);
}
