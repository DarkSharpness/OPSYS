use super::constant::*;

const TABLE : *mut u64 = PAGE_TABLE as *mut u64;

unsafe fn set_huge(x : u64, y : u64, flag : u64) {
    for i in x..y {
        let mut pte : u64 = i;
        pte <<= 28;
        pte |= flag;
        TABLE.wrapping_add(i as usize).write(pte as u64);
    }
}

// Initialize the huge page table.
pub unsafe fn init_huge_page() {
    // EXECUTE, WRITE, READ, VALID
    set_huge(  0  , 256 , 0b0000 ); // Reset to invalid first.
    set_huge(  0  ,  2  , 0b0111 ); // MMIO part.
    set_huge(  2  ,  4  , 0b1111 ); // Kernel part.

    // Kernel part should be separated.


}
