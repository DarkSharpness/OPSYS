mod input;
mod output;

use crate::FileDescriptor;

const STDIN : FileDescriptor = unsafe { FileDescriptor::new(0) };
const STDOUT : FileDescriptor = unsafe { FileDescriptor::new(1) };
// const STDERR : FileDescriptor = unsafe { FileDescriptor::new(2) };

pub use output::print_fmt;
pub use input::read_int;