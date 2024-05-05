pub trait File {
    fn read(&mut self, buf: *mut u8, x : usize) -> usize;
    fn write(&mut self, buf: *const u8, x : usize) -> usize;
}
