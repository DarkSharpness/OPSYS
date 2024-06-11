use crate::trap::user_trap_return;

#[repr(C)]
pub struct Context { stack_bottom : usize, }

impl Context {
    pub const fn new() -> Self {
        return Self { stack_bottom : 0, };
    }

    /** Create a with given ra and sp. */
    pub(super) fn new_with(sp : usize) -> Self {
        let ra = user_trap_return as usize;
        let ptr = sp as *mut usize;
        unsafe { ptr.wrapping_sub(1).write_volatile(ra); }
        return Self { stack_bottom : sp, };
    }
}
