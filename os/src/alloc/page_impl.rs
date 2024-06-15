extern crate alloc;

use crate::alloc::page::{A, D, G, R, W, X} ;
use super::{page::{PTEFlag, PTEOwner, PageAddress, PageTableEntry, U}, PAGE_BITS};

impl core::ops::Index<usize> for PageAddress {
    type Output = PageTableEntry;
    fn index(&self, x : usize) -> &PageTableEntry {
        unsafe { self.get_entry(x) }
    }
}

impl core::ops::IndexMut<usize> for PageAddress {
    fn index_mut(&mut self, x : usize) -> &mut PageTableEntry {
        unsafe { self.get_entry(x) }
    }
}

impl PageAddress {
    /** Debug method. */
    pub fn debug(self) {
        warning!("Root address = {:#x}", self.address() as usize);
        for i in 0..512 {
            let base = i << 18;
            let (addr, flag) = self[i].get_entry();
            if flag == PTEFlag::INVALID { continue; }
            if flag != PTEFlag::NEXT {
                message_inline!("Mapping 1GiB {:<12p} -> {:<10p} Flag = ",
                    to_virtual(base) , addr.address());
                flag.debug(); 
                continue;
            }
            warning!("- Huge page {:p}", addr.address());
            for j in 0..512 {
                let base = base | j << 9;
                let (addr, flag) = addr[j].get_entry();
                if flag == PTEFlag::INVALID { continue; }
                if flag != PTEFlag::NEXT {
                    message_inline!("  - Mapping 2MiB {:<12p} -> {:<10p} Flag = ",
                        to_virtual(base), addr.address());
                    flag.debug();
                    continue;
                }
                warning!("  - Large page {:p}", addr.address());
                for k in 0..512 {
                    let base = base | k;
                    let (addr, flag) = addr[k].get_entry();
                    if flag == PTEFlag::INVALID { continue; }
                    assert!(flag != PTEFlag::NEXT, "Invalid page table mapping!");
                    message_inline!("    - Mapping 4KiB {:<12p} -> {:<10p} Flag = ",
                        to_virtual(base), addr.address());
                    flag.debug();
                }
            }
        }
    }
}

impl PTEFlag {
    /** Debug output. */
    fn debug(self) {
        print_if(self.contains(D), 'D');
        print_if(self.contains(A), 'A');
        print_if(self.contains(G), 'G');
        print_if(self.contains(U), 'U');
        print_if(self.contains(X), 'X');
        print_if(self.contains(W), 'W');
        print_if(self.contains(R), 'R');
        match self.get_owner() {
            PTEOwner::Kernel    => { uart_print!(" | Kernel"); }
            PTEOwner::Process   => { uart_print!(" | Process"); }
            PTEOwner::Shared    => { uart_print!(" | Shared"); }
            PTEOwner::Reserved  => { uart_print!(" | Reserved"); }
        }
        uart_print!("\n");
    }
}

#[inline(always)]
fn print_if(cond : bool, mut x : char) { if !cond { x = '-'; } uart_print!("{}", x); }

#[inline(always)]
fn to_virtual(x : usize) -> *mut u8 { return (x << PAGE_BITS) as _; }
