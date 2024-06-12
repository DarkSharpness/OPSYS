#![no_std]
#![no_main]

use user_lib::*;
extern crate user_lib;

unsafe fn to_str(mut num : isize, buf : *mut u8) -> isize {
    let mut i = 0;
    let mut neg = false;
    if num < 0 {
        neg = true;
        num = -num;
    }

    if num == 0 {
        *buf.offset(i) = '0' as u8;
        i += 1;
    } else {
        loop {
            *buf.offset(i) = (num % 10) as u8 + '0' as u8;
            num /= 10;
            i += 1;
            if num == 0 { break; }
        }
    }
    if neg {
        *buf.offset(i) = '-' as u8;
        i += 1;
    }

    *buf.offset(i) = 0;
    let mut j = 0;
    while j < i / 2 {
        let tmp = *buf.offset(j);
        *buf.offset(j) = *buf.offset(i - j - 1);
        *buf.offset(i - j - 1) = tmp;
        j += 1;
    }

    return i;
}


#[no_mangle]
fn main() -> i32 {
    let mut arr : [u8; 16] = [0; 16];
    unsafe {
        // let stdin   = FileDescriptor::new(0);
        let stdout  = FileDescriptor::new(1);
        // let buf = arr.as_mut_ptr() as *mut u8;
        // sys_read(stdin, buf, 16);
        let child = sys_fork();
        // Format to buffer
        let len = to_str(child, arr.as_mut_ptr());
        arr[len as usize] = '\n' as u8;
        sys_write(stdout, arr.as_mut_ptr(), (len + 1) as _);
 
        // sys_write(stdout, buf, 16);
        sys_yield();
    }
    return 0;
}
