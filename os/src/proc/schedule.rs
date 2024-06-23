use core::ptr::null_mut;

use super::Process;

extern crate alloc;
use alloc::collections::BTreeSet;



struct Wrapper {
    process : *mut Process,
}

fn cmp_wrapper(a : &Wrapper, b : &Wrapper) -> core::cmp::Ordering {
    let a = unsafe { &*a.process };
    let b = unsafe { &*b.process };
    let tmp = a.get_timing().cmp(&b.get_timing());
    if tmp == core::cmp::Ordering::Equal {
        let a = a as *const Process;
        let b = b as *const Process;
        return a.cmp(&b);
    } else {
        return tmp;
    }
}

impl PartialEq for Wrapper {
    fn eq(&self, other: &Self) -> bool {
        return self.process == other.process;
    }
}

impl Eq for Wrapper {}

impl PartialOrd for Wrapper {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        return Some(cmp_wrapper(self, other));
    }
}

impl Ord for Wrapper {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        return cmp_wrapper(self, other);
    }
}

pub struct Schuduler {
    run_set : BTreeSet <Wrapper>,
    current : *mut Process,
}

impl Schuduler {
    pub const fn new() -> Self {
        return Self { run_set : BTreeSet::new(), current : null_mut() };
    }

    fn max_timing(&self) -> usize {
        let pair = self.run_set.last();
        if let Some(pair) = pair {
            let process = unsafe { &*pair.process };
            return process.get_timing();
        } else {
            return 0;
        }
    }

    fn min_timing(&self) -> usize {
        let pair = self.run_set.first();
        if let Some(pair) = pair {
            let process = unsafe { &*pair.process };
            return process.get_timing();
        } else {
            return 0;
        }
    }

    fn get_step(process : &Process) -> usize {
        const MAX : usize = Process::max_priority() as usize + 1;
        return MAX / (process.get_priority() + 1);
    }

    pub fn register(&mut self, process : &mut Process) {
        let min = self.min_timing();
        process.set_timing(min + Self::get_step(process));
        let result = self.run_set.insert(Wrapper { process });
        if !result {
            for value in self.run_set.iter() {
                let process = unsafe { &*value.process };
                warning!("Process: {}", process.get_pid().bits());
            }
            panic!("Process already registered");
        }
    }

    pub fn unregister(&mut self, process : *mut Process) {
        self.run_set.remove(&Wrapper { process });
    }

    fn end_last_process(&mut self) {
        if !self.current.is_null() {
            let process = unsafe { &mut *self.current };
            let old = process.get_timing();
            process.set_timing(old + Self::get_step(process));
            if process.has_status(super::ProcessStatus::RUNNABLE) {
                self.run_set.insert(Wrapper { process });
            }
        }
    }

    fn find_first_process(&mut self) -> *mut Process {
        while let Some(value) = self.run_set.first() {
            let process = value.process;
            self.run_set.pop_first();
            let process = unsafe { &mut *process };
            if process.has_status(super::ProcessStatus::RUNNABLE) {
                self.current = process;
                return process;
            }
        }
        self.current = null_mut();
        return null_mut();
    }

    pub fn next_process(&mut self) -> *mut Process {
        // End the last process
        self.end_last_process();
        return self.find_first_process();
    }
}
