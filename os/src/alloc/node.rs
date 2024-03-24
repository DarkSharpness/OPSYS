pub struct Node {
    pub prev : *mut Node,
    pub next : *mut Node,
}

pub struct List {
    pub head : Node
}

#[inline(always)]
pub unsafe fn link(prev : *mut Node, next : *mut Node) {
    (*prev).next = next; (*next).prev = prev;
}

#[inline(always)]
pub unsafe fn unlink(node : *mut Node) {
    let prev = (*node).prev;
    let next = (*node).next;
    (*prev).next = next; (*next).prev = prev;
}

impl List {
    pub unsafe fn init(&mut self) {
        let addr = &mut self.head as *mut Node;
        (*addr).prev = addr;
        (*addr).next = addr;
    }

    pub unsafe fn push(&mut self, node : *mut Node) {
        let head = &mut self.head as *mut Node;
        link(node, (*head).next);
        link(head, node);
    }

    pub unsafe fn pop(&mut self) -> *mut Node {
        let head = &mut self.head as *mut Node;
        let node = (*head).next;
        link(head, (*node).next);
        return node;
    }

    pub unsafe fn empty(&self) -> bool {
        let head = &self.head as *const Node;
        let next = (*head).next as *const Node;
        return head == next;
    }
}
