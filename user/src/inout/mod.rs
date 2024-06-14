mod input;
mod error;
mod output;
mod buffer;

use crate::FileDescriptor;

const STDIN : FileDescriptor = unsafe { FileDescriptor::new(0) };
const STDOUT : FileDescriptor = unsafe { FileDescriptor::new(1) };
const STDERR : FileDescriptor = unsafe { FileDescriptor::new(2) };

pub use output::print_fmt;
pub use error::error_fmt;
pub use input::read_int;
