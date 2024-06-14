use xmas_elf::{program::{Flags, ProgramHeader}, ElfFile};

use crate::{alloc::{PTEFlag, PageAddress, PAGE_SIZE}, get_zero_page};

use super::Process;

unsafe fn get_header_range(ph : ProgramHeader) -> (usize, usize) {
    let start_va    = ph.virtual_addr() as usize;
    let end_va      = (ph.virtual_addr() + ph.mem_size()) as usize;
    let mem_size    = end_va - start_va;
    assert!(mem_size >= ph.file_size() as usize, "Invalid file size!");
    return (start_va, mem_size);
}

unsafe fn get_header_permission(ph_flags : Flags) -> PTEFlag {
    let mut permission = PTEFlag::EMPTY;
    if ph_flags.is_read() { permission |= PTEFlag::RO; }
    if ph_flags.is_write() { permission |= PTEFlag::WO; }
    if ph_flags.is_execute() { permission |= PTEFlag::RX; }
    return permission;
}

unsafe fn get_header_data<'a>(ph : ProgramHeader<'a>, elf : &'a ElfFile<'a>) -> &'a [u8] {
    let ph_offset = ph.offset() as usize;
    let ph_size = ph.file_size() as usize;
    return &elf.input[ph_offset..ph_offset + ph_size];
}

impl PageAddress {
    unsafe fn add_range(self, start_va : usize , data : &[u8], permission : PTEFlag) {
        let offset      = start_va % PAGE_SIZE;
        let padding     = PAGE_SIZE - offset;
        let page        = self.try_umap(start_va, permission);
        if data.len() <= padding {
            page.copy_at(offset, data);
        } else {
            let (mut prefix, mut data) = data.split_at(padding);
            page.copy_at(offset, prefix);

            let mut now_va = start_va + padding;

            while data.len() >= PAGE_SIZE {
                let page = self.try_umap(now_va, permission);
                (prefix, data) = data.split_at(PAGE_SIZE);
                page.copy_at(0, prefix);
                now_va += PAGE_SIZE;
            }

            if data.len() > 0 {
                let page = self.try_umap(now_va, permission);
                page.copy_at(0, data);
            }
        }
    }

    unsafe fn add_range_zero(self, start_va : usize, mem_size : usize, permission : PTEFlag) {
        let offset      = start_va % PAGE_SIZE;
        let remain      = PAGE_SIZE - offset;
        let page        = self.try_umap(start_va, permission);
        let zero_page   = get_zero_page();

        if mem_size <= remain {
            page.copy_at(offset, &zero_page[..mem_size]);
        } else {
            page.copy_at(offset, &zero_page[..remain]);
            let mut now_va = start_va + remain;

            let mut rest = mem_size - remain;
            while rest >= PAGE_SIZE {
                let page = self.new_umap(now_va, permission);
                page.copy_at(0, zero_page);
                now_va += PAGE_SIZE;
                rest -= PAGE_SIZE;
            }

            if rest > 0 {
                let page = self.try_umap(now_va, permission);
                page.copy_at(0, &zero_page[..rest]);
            }
        }
    }

    /** Load a program header from an ELF file. */
    pub unsafe fn load_from_elf(&mut self, ph : ProgramHeader, elf : &ElfFile) -> usize {
        let (start_va, mem_size) = get_header_range(ph);
        if mem_size == 0 { return start_va; }

        let permission  = get_header_permission(ph.flags());
        let data        = get_header_data(ph, elf);

        self.add_range(start_va, data, permission);

        // The rest bits are set to zero.
        if mem_size != data.len() {
            let remain = mem_size - data.len();
            self.add_range_zero(start_va + data.len(), remain, permission);
        }
        return start_va + mem_size;
    }
}

impl Process {
    pub unsafe fn init_from_elf(&mut self, data : &[u8]) {
        let elf = xmas_elf::ElfFile::new(&data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_vaddr = 0;
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                // message!("{}", ph);
                let vaddr_end = self.get_satp().load_from_elf(ph, &elf);
                if vaddr_end > max_vaddr {
                    max_vaddr = vaddr_end;
                }
            }
        }

        let memory = self.get_memory_area();

        message!("Program end: {:#x}", memory.set_program_end(max_vaddr));

        let trap_frame = self.get_trap_frame();
        trap_frame.pc = elf.header.pt2.entry_point() as usize;
    }
}