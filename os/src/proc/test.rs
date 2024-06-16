use core::ptr::addr_of;

use super::{memory::MemoryArea, Process};

#[repr(C)]
struct Property {
    start   : *const u8,    // inclusive
    length  : usize,        // exclusive
    name    : *const u8,    // name of the test
    strlen  : usize,        // length of name
}

unsafe fn load_file(which : usize) -> &'static [u8] {
    extern "C" {
        static _num_app : usize;
        static _app_meta: Property;
    }

    let num : usize = _num_app;
    assert!(which < num, "Invalid test number!");
    let meta = &*addr_of!(_app_meta).wrapping_add(which);
    let name = meta.get_name();
    warning!("Initing program: {}", name);

    return meta.get_data();
}


impl Process {
    pub(super) unsafe fn new_test(which : usize) -> Process {
        let data = load_file(which);

        let mut process = Process::init();

        // Initialize the text and data segment.
        process.init_from_elf(data);

        // Initialize the user stack.
        let trap_frame = process.get_trap_frame();
        trap_frame.sp = MemoryArea::get_user_stack_top();

        process.get_memory_area().add_stack(1);

        return process;
    }

/* 
    pub(super) unsafe fn old_test(which : bool) -> Process {
        let process = Process::init();
        let text = PageAddress::new_zero_page();
        process.root.umap(0, text, PTEFlag::RX | PTEFlag::OWNED);

        // Identical mapping for MMIO.
        let mmio = PageAddress::new_usize(0x10000000);
        process.root.umap(0x10000000, mmio, PTEFlag::RW);

        let addr = text.address() as *mut u32;
        let program = if which {[
            0x140413,       // addi s0, s0 1
            0x140413,       // addi s0, s0 1
            0x140413,       // addi s0, s0 1
            0x0000bfd5,     // j 0
        ] } else { [
            0x10000537,     // lui a0,0x10000
            0x0320059b,     // addiw a1,zero,0x32
            0x00b50023,     // sb a1,0(a0)
            0x0000bfd5      // j 0
        ] };

        for i in 0..program.len() {
            addr.wrapping_add(i).write_volatile(program[i]);
        }

        return process;
    }
*/
}

impl Property {
    pub unsafe fn get_name (&self) -> &str {
        core::str::from_utf8(core::slice::from_raw_parts(self.name, self.strlen)).unwrap()
    }
    pub unsafe fn get_data (&self) -> &[u8] {
        core::slice::from_raw_parts(self.start, self.length)
    }
}
