use crate::driver::uart;

use super::get_tid;

#[derive(Clone, Copy)]
enum Mode {
    Machine     = 0,
    Supervisor  = 1,
}

#[derive(Clone, Copy)]
enum IRQ {
    UART0   = 10,
    VIRTIO  = 1,
}

const fn supported_irq() -> [IRQ; 2] {
    return [IRQ::UART0, IRQ::VIRTIO];
}

const fn supported_mode() -> [Mode; 2] {
    return [Mode::Machine, Mode::Supervisor];
}

struct Plic { id : usize }

/** Resolve a plic. */
pub unsafe fn resolve() {
    let supervisor = Plic::new(Mode::Supervisor);
    let irq = supervisor.get_claim();
    match irq {
        10 => uart::uart_trap(),
        1 => todo!("VIRTIO IRQ"),
        _ => panic!("Unknown IRQ: {}", irq)
    }
    supervisor.set_claim(irq);
}

pub unsafe fn init() {
    // Disable all interrupts from machine mode.
    let machine = Plic::new(Mode::Machine);
    machine.set_threshold(1);

    // Enable all interrupts from supervisor mode.
    let supervisor = Plic::new(Mode::Supervisor);
    supervisor.set_threshold(0);

    // Enable all supported interrupts.
    for irq in supported_irq() {
        supervisor.enable(irq);
        supervisor.set_priority(irq, 1);
    }
}

impl Plic {
    const BASE : usize = 0xC_000_000;   // PLIC Base Address
    unsafe fn new(mode : Mode) -> Self {
        let tid = get_tid();
        let idx = tid * supported_mode().len() + (mode as usize);
        return Plic { id : idx };
    }
    unsafe fn ptr_threshold(&self) -> *mut u32 {
        let addr = Plic::BASE + 0x200000 + self.id * 0x1000;
        return addr as *mut u32;
    }
    unsafe fn ptr_claim(&self) -> *mut u32 {
        let addr = Plic::BASE + 0x200004 + self.id * 0x1000;
        return addr as *mut u32;
    }
    unsafe fn ptr_enable(&self) -> *mut u32 {
        let addr = Plic::BASE + 0x2000 + self.id * 0x80;
        return addr as *mut u32;
    }
    unsafe fn ptr_priority(&self) -> *mut u32 {
        let addr = Plic::BASE;
        return addr as *mut u32;
    }
}

impl Plic {
    unsafe fn set_threshold(&self, threshold: u32) {
        let ptr = self.ptr_threshold();
        ptr.write_volatile(threshold);
    }
    unsafe fn set_claim(&self, irq : u32) {
        let ptr = self.ptr_claim();
        ptr.write_volatile(irq);
    }
    unsafe fn set_priority(&self, irq : IRQ, priority : u32) {
        let ptr = self.ptr_priority();
        ptr.wrapping_add(irq as _).write_volatile(priority);
    }
    unsafe fn enable(&self, irq : IRQ) {
        let irq     = irq as usize;
        let ptr     = self.ptr_enable();
        let mask    = 1 << (irq % 32);
        let offset  = irq / 32;
        let old     = ptr.wrapping_add(offset as _).read_volatile();
        let new     = old | mask;
        ptr.wrapping_add(offset as _).write_volatile(new);
    }
}

impl Plic {
    unsafe fn get_threshold(&self) -> u32 {
        let ptr = self.ptr_threshold();
        return ptr.read_volatile();
    }
    unsafe fn get_claim(&self) -> u32 {
        let ptr = self.ptr_claim();
        return ptr.read_volatile();
    }
}
