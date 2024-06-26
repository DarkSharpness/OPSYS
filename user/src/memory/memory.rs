use core::{cmp::max, sync::atomic::AtomicBool};
use crate::{memory::{heap::get_heap, list::Header}, sys_sbrk};
use super::{heap::Heap, list::Node};

unsafe fn malloc_init() {
    let heap_beg = sys_sbrk(4096) as usize;
    let heap_end = heap_beg + 4096;
    assert!(heap_beg % 4096 == 0, "Misaligned heap");
    let heap = get_heap();
    return heap.init(heap_beg, heap_end);
}

static mut INIT : AtomicBool = AtomicBool::new(false);

unsafe fn enlarge(size : usize) {
    let size = (size + 4095) & !4095;
    let heap = get_heap();
    return heap.enlarge(sys_sbrk(size as isize), size);
}

unsafe fn try_find(heap : &mut Heap, size : usize) -> *mut Header {
    let header = heap.first_fit(size);
    if header.is_null() {
        enlarge(size);
        let header = heap.first_fit(size);
        assert!(!header.is_null(), "Out of memory");
        return header;
    } else {
        return header;
    }
}

pub unsafe fn malloc(size : usize) -> *mut u8 {
    if !INIT.fetch_or(true, core::sync::atomic::Ordering::Relaxed) {
        malloc_init();
    }

    const MIN_SIZE : usize = size_of::<Header>() + size_of::<Node>();
    let size = max((size + size_of::<Header>() + 7) & !7, MIN_SIZE);
    let heap = get_heap();
    let header = try_find(heap, size);

    let (data, rest) = (*header).try_split(size);
    match rest {
        Some(rest)  => heap.recycle(rest),
        None        => {},
    }

    return data;
}

unsafe fn check_initialized() {
    assert!(INIT.load(core::sync::atomic::Ordering::Relaxed), "Malloc not initialized");
}

pub unsafe fn free(ptr : *mut u8) {
    if ptr.is_null() { return; }
    check_initialized();
    assert!(ptr as usize % 8 == 0, "Misaligned pointer");
    let heap = get_heap();
    let header = ptr as *mut Header;
    heap.recycle(header.sub(1));
}

pub unsafe fn malloc_usable_size(ptr : *mut u8) -> usize {
    if ptr.is_null() { return 0; }
    check_initialized();
    assert!(ptr as usize % 8 == 0, "Misaligned pointer");
    let header = ptr.sub(8) as *mut Header;
    return (*header).get_size() as usize - size_of::<Header>();
}

pub unsafe fn malloc_dump() {
    check_initialized();
    let heap = get_heap();
    heap.dump_and_check();
}
