mod list;
mod heap;
mod memory;

use core::{alloc::{GlobalAlloc, Layout}, cmp::max};

pub use memory::*;

// I'm the author.
struct DarkSharpness;

unsafe impl GlobalAlloc for DarkSharpness {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = max(layout.size(), layout.align());
        return malloc(size);
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let _ = max(layout.size(), layout.align());
        return free(ptr);
    }
}

#[global_allocator]
static _DARKSHARPNESS_ : DarkSharpness = DarkSharpness;