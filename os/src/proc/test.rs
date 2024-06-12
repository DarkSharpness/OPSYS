use core::ptr::addr_of;

use crate::alloc::{PTEFlag, PageAddress, PAGE_SIZE};

use super::Process;

const USER_STACK : usize = 1 << 38;

impl PageAddress {
    unsafe fn map_stack(self, cnt : usize) {
        for i in 0..cnt {
            let stack_page = PageAddress::new_rand_page();
            let user_stack = USER_STACK - (i + 1) * PAGE_SIZE as usize;
            self.umap(user_stack, stack_page, PTEFlag::RW);
        }
    }
}

impl Process {
    pub(super) unsafe fn new_test(which : usize) -> Process {
        let process = Process::init();

        extern "C" { static _num_app : usize; }

        let num : usize = _num_app;
        assert!(which < num, "Invalid test number!");

        let addr = addr_of!(_num_app).wrapping_add(1);

        let program_start   = *addr.wrapping_add(which * 2);
        let program_finish  = *addr.wrapping_add(which * 2 + 1);
        let program_length  = program_finish - program_start;

        let data : &[u8] = core::slice::from_raw_parts(
            program_start as *const u8, program_length);

        let elf = xmas_elf::ElfFile::new(&data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                message!("{}", ph);
                process.get_satp().load_from_elf(ph, &elf);
            }
        }

        let trap_frame = process.get_trap_frame();
        trap_frame.pc = elf.header.pt2.entry_point() as usize;
        trap_frame.sp = USER_STACK;
        process.get_satp().map_stack(1);

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
