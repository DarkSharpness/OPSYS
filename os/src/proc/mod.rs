mod proc;
mod schedule;

pub unsafe fn init_proc() {
    proc::init_process();
}