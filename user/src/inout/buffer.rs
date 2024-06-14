#[derive(PartialEq, Eq)]
pub enum BufState {
    Normal      = 0,
    EndOfFile   = 1,
    Error       = 2,
}

pub type Bufsize = u32;
const BUFLEN : usize = 500;

#[repr(align(512))]
pub struct StdBuf {
    buffer : [u8; BUFLEN],
    begin  : Bufsize,
    finish : Bufsize,
    state  : BufState,
}

impl StdBuf {
    pub const fn new() -> Self {
        Self {
            buffer  : [0; BUFLEN],
            begin   : 0,
            finish  : 0,
            state   : BufState::Normal,
        }
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer[self.begin as usize..self.finish as usize]
    }
    pub fn set_error(&mut self) {
        self.state = BufState::Error;
    }
    pub fn set_eof(&mut self) {
        self.state = BufState::EndOfFile;
    }
    pub fn is_good(&self) -> bool {
        self.state == BufState::Normal
    }
    pub fn set_range(&mut self, begin: Bufsize, finish: Bufsize) {
        self.begin = begin;
        self.finish = finish;
    }
    pub fn set_clear(&mut self) {
        self.begin = 0;
        self.finish = 0;
    }
    pub fn is_empty(&self) -> bool {
        self.begin == self.finish
    }
    pub fn is_full(&self) -> bool {
        self.finish == (BUFLEN as Bufsize)
    }
    pub fn get_buffer(&self) -> &[u8] {
        &self.buffer
    }
    pub fn get_mut_buffer(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
    pub fn pop_n(&mut self, n: Bufsize) {
        self.begin += n;
    }
    pub fn push_str(&mut self, bytes : &[u8]) {
        let len = bytes.len();
        let buf = &mut self.buffer[self.finish as usize..];
        buf[..len].copy_from_slice(bytes);
        self.finish += len as Bufsize;
    }
    pub fn remain(&mut self) -> &mut [u8] {
        &mut self.buffer[self.finish as usize..]
    }
}
