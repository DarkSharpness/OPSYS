use sys::syscall::*;

use crate::sys_request;

pub struct Mutex {
    dummy: usize,
}


impl Mutex {
    pub fn new() -> Mutex {
        let args = crate::Argument::Register(0, 0);
        let ret = sys_request(args, PM_MUTEX_CREATE, PM_PORT);
        return Mutex { dummy: ret as usize };
    }

    pub fn lock(&self) {
        let args = crate::Argument::Register(self.dummy, 0);
        sys_request(args, PM_MUTEX_LOCK, PM_PORT);
    }

    pub fn unlock(&self) {
        let args = crate::Argument::Register(0, self.dummy);
        sys_request(args, PM_MUTEX_UNLOCK, PM_PORT);
    }

    pub fn destroy(&self) {
        let args = crate::Argument::Register(0, self.dummy);
        sys_request(args, PM_MUTEX_DESTROY, PM_PORT);
    }
}

pub struct Condvar {
    dummy: usize,
}

impl Condvar {
    pub fn new() -> Condvar {
        let args = crate::Argument::Register(0, 0);
        let ret = sys_request(args, PM_COND_CREATE, PM_PORT);
        return Condvar { dummy: ret as usize };
    }

    pub fn wait(&self, mutex: &Mutex) {
        let args = crate::Argument::Register(mutex.dummy, self.dummy);
        sys_request(args, PM_COND_WAIT, PM_PORT);
    }

    pub fn signal(&self) {
        let args = crate::Argument::Register(0, self.dummy);
        sys_request(args, PM_COND_SIGNAL, PM_PORT);
    }

    pub fn broadcast(&self) {
        let args = crate::Argument::Register(0, self.dummy);
        sys_request(args, PM_COND_BROADCAST, PM_PORT);
    }

    pub fn destroy(&self) {
        let args = crate::Argument::Register(0, self.dummy);
        sys_request(args, PM_COND_DESTROY, PM_PORT);
    }
}
