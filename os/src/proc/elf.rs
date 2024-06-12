use xmas_elf::{program::{Flags, ProgramHeader}, ElfFile};

use crate::alloc::{PTEFlag, PageAddress, PAGE_SIZE};

unsafe fn get_header_range(ph : ProgramHeader) -> (usize, usize) {
    let start_va    = ph.virtual_addr() as usize;
    let end_va      = (ph.virtual_addr() + ph.mem_size()) as usize;
    let ph_size     = end_va - start_va;
    assert!(ph_size == ph.file_size() as usize, "Invalid file size!");
    return (start_va, ph_size);
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
    /** Load a program header from an ELF file. */
    pub unsafe fn load_from_elf(&mut self, ph : ProgramHeader, elf : &ElfFile) {
        let (start_va, ph_size) = get_header_range(ph);
        if ph_size == 0 { return; }

        let permission  = get_header_permission(ph.flags());
        let page        = self.new_umap(start_va, permission);
        let offset      = start_va % PAGE_SIZE;
        let remain      = PAGE_SIZE - offset;
        let data        = get_header_data(ph, elf);

        if data.len() <= remain {
            page.copy_at(offset, data);
        } else {
            let (mut prefix, mut data) = data.split_at(remain);
            page.copy_at(offset, prefix);
            let mut now_va = start_va + remain;

            while data.len() >= PAGE_SIZE {
                let page = self.new_umap(now_va, permission);
                (prefix, data) = data.split_at(PAGE_SIZE);
                page.copy_at(0, prefix);
                now_va += PAGE_SIZE;
            }

            if data.len() > 0 {
                let page = self.new_umap(now_va, permission);
                page.copy_at(0, data);
            }
        }
    }
}