pub trait File {
    fn read(buf: *mut u8, x : usize) -> usize;
    fn write(buf: *mut u8, x : usize) -> usize;
}
