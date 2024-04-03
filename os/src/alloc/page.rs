use super::constant::*;

const TABLE : *mut u64 = PAGE_TABLE as *mut u64;

// Initialize the huge page table.
pub unsafe fn init_huge_page() {
    for i in 0..512 {
        let mut pte : u64 = i;
        pte <<= 28;
        pte |= 0b1111;  // EXECUTE, WRITE, READ, VALID
        TABLE.wrapping_add(i as usize).write(pte as u64);
    }
}
