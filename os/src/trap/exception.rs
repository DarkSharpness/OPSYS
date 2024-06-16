use crate::proc::Process;

#[derive(Debug, PartialEq)]
pub enum PageFaultType {
    Load,
    Store,
    Instruction,
}

impl Process {
    pub unsafe fn handle_page_fault(&mut self, addr: usize, tp : PageFaultType) {
        if !self.get_memory_area().handle_page_fault(addr, tp) {
            warning!("Page fault at 0x{:x}", addr);
            self.handle_fatal_error("");
        }
    }
}
