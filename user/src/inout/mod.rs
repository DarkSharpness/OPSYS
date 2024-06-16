mod input;
mod error;
mod output;
mod buffer;

use crate::FileDescriptor;

const STDIN  : FileDescriptor   = FileDescriptor::new(0);
const STDOUT : FileDescriptor   = FileDescriptor::new(1);
const STDERR : FileDescriptor   = FileDescriptor::new(2);

pub use output::*;
pub use error::*;
pub use input::*;
