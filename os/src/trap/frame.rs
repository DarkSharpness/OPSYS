use crate::alloc::{PageAddress, PAGE_TABLE};
pub struct TrapFrame {
    ra  : u64,
    gp  : u64,
    tp  : u64,
    t0  : u64,
    t1  : u64,
    t2  : u64,
    s0  : u64,
    s1  : u64,

    a0  : u64,
    a1  : u64,
    a2  : u64,
    a3  : u64,
    a4  : u64,
    a5  : u64,
    a6  : u64,
    a7  : u64,

    s2  : u64,
    s3  : u64,
    s4  : u64,
    s5  : u64,
    s6  : u64,
    s7  : u64,
    s8  : u64,
    s9  : u64,

    s10 : u64,
    s11 : u64,
    t3  : u64,
    t4  : u64,
    t5  : u64,
    t6  : u64,
    sp  : u64,
    pc  : u64,

    thread_number   : u64,          // real thread number
    kernel_stack    : u64,          // kernel stack pointer
    kerner_satp     : PageAddress,  // kernel page table
}

impl TrapFrame {
    pub fn new() -> Self {
        TrapFrame {
            ra  : 0,
            gp  : 0,
            tp  : 0,
            t0  : 0,
            t1  : 0,
            t2  : 0,
            s0  : 0,
            s1  : 0,

            a0  : 0,
            a1  : 0,
            a2  : 0,
            a3  : 0,
            a4  : 0,
            a5  : 0,
            a6  : 0,
            a7  : 0,

            s2  : 0,
            s3  : 0,
            s4  : 0,
            s5  : 0,
            s6  : 0,
            s7  : 0,
            s8  : 0,
            s9  : 0,

            s10 : 0,
            s11 : 0,
            t3  : 0,
            t4  : 0,
            t5  : 0,
            t6  : 0,
            sp  : 0,
            pc  : 0,

            thread_number   : 0,
            kernel_stack    : 0,
            kerner_satp     : PAGE_TABLE
        }
    }
}