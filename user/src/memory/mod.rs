mod list;
mod heap;
mod memory;

use core::alloc::{GlobalAlloc, Layout};

pub use memory::*;

// I'm the author.
struct DarkSharpness;

unsafe impl GlobalAlloc for DarkSharpness {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        assert!(layout.align() <= 8, "For now, we only support align <= 8");
        let ptr = malloc(layout.size());
        return ptr;
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let _ = layout;
        return free(ptr);
    }
}

#[global_allocator]
static _DARKSHARPNESS_ : DarkSharpness = DarkSharpness;
