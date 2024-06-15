#![no_std]
#![no_main]

use user_lib::*;

#[no_mangle]fn main() -> i32 {
    unsafe { demo(); }
    return 0;
}

unsafe fn demo() {
    let size = 1024;
    let ptr0 = malloc(size);
    println!("malloc ptr: {:p} {}", ptr0, malloc_usable_size(ptr0));
    let ptr1 = malloc(size);
    println!("malloc ptr: {:p} {}", ptr1, malloc_usable_size(ptr1));
    malloc_dump();

    free(ptr0);
    free(ptr1);
    let ptr2 = malloc(size * 8);
    println!("malloc ptr: {:p} {}", ptr2, malloc_usable_size(ptr2));

}
